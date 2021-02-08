use std::ffi::OsString;
use std::fs::{File, Metadata};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use std::{error, io};

use iron::headers::{
    AcceptEncoding, AcceptRanges, ContentEncoding, ContentLength, Encoding, HttpDate,
    IfModifiedSince, LastModified, Range, RangeUnit,
};

use iron::method::Method;
use iron::middleware::Handler;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;

use crate::staticfile_middleware::helpers;
use crate::staticfile_middleware::partial_file::PartialFile;

/// Recursively serves files from the specified root and assets directories.
pub struct Staticfile {
    root: PathBuf,
    assets: PathBuf,
}

impl Staticfile {
    pub fn new<P>(root: P, assets: P) -> io::Result<Staticfile>
    where
        P: AsRef<Path>,
    {
        let root = root.as_ref().canonicalize()?;
        let assets = assets.as_ref().canonicalize()?;

        Ok(Staticfile { root, assets })
    }

    fn resolve_path(&self, path: &[&str]) -> Result<PathBuf, Box<dyn error::Error>> {
        let path_dirname = path[0];
        let asserts_dirname = self.assets.iter().last().unwrap().to_str().unwrap();
        let mut is_assets = false;

        let resolved = if path_dirname == asserts_dirname {
            // Assets path validation resolve
            is_assets = true;

            let mut res = self.assets.clone();
            for component in path.iter().skip(1) {
                res.push(component);
            }

            res
        } else {
            // Root path validation resolve
            let mut res = self.root.clone();
            for component in path {
                res.push(component);
            }

            res
        };

        let resolved = resolved.canonicalize()?;
        let path = if is_assets { &self.assets } else { &self.root };

        // Protect against path/directory traversal
        if !resolved.starts_with(&path) {
            return Result::Err(From::from(format!("Cannot leave {:?} path", &path)));
        }

        Ok(resolved)
    }
}

impl Handler for Staticfile {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Accept only HEAD and GET methods
        if !(req.method == Method::Head || req.method == Method::Get) {
            return Ok(Response::with(status::MethodNotAllowed));
        }

        // Resolve path on file system
        let file_path = match self.resolve_path(&req.url.path()) {
            Ok(file_path) => file_path,
            Err(_) => return Ok(Response::with(status::NotFound)),
        };

        // Get current file metadata
        let accept_gz = helpers::accept_gzip(req.headers.get::<AcceptEncoding>());
        let file = match StaticFileWithMetadata::search(&file_path, accept_gz) {
            Ok(file) => file,
            Err(_) => return Ok(Response::with(status::NotFound)),
        };

        // Apply last modified date time
        let client_last_modified = req.headers.get::<IfModifiedSince>();
        let last_modified = file.last_modified().ok().map(HttpDate);

        if let (Some(client_last_modified), Some(last_modified)) =
            (client_last_modified, last_modified)
        {
            trace!(
                "Comparing {} (file) <= {} (req)",
                last_modified,
                client_last_modified.0
            );

            if last_modified <= client_last_modified.0 {
                return Ok(Response::with(status::NotModified));
            }
        }

        // Add Encoding Gzip header
        let encoding = if file.is_gz {
            Encoding::Gzip
        } else {
            Encoding::Identity
        };
        let encoding = ContentEncoding(vec![encoding]);

        let mut resp = match last_modified {
            Some(last_modified) => {
                let last_modified = LastModified(last_modified);
                Response::with((
                    status::Ok,
                    Header(last_modified),
                    Header(encoding),
                    file.file,
                ))
            }
            None => Response::with((status::Ok, Header(encoding), file.file)),
        };

        // Empty current response body on HEAD requests,
        // just setting up the `content-length` header (size of the file in bytes)
        // https://tools.ietf.org/html/rfc7231#section-4.3.2
        if req.method == Method::Head {
            resp.set_mut(vec![]);
            resp.set_mut(Header(ContentLength(file.metadata.len())));
            return Ok(resp);
        }

        // Partial Content Delivery response
        // Enable the "Accept-Ranges" header on all files
        resp.set_mut(Header(AcceptRanges(vec![RangeUnit::Bytes])));

        let resp = match req.headers.get::<Range>().cloned() {
            // Deliver the whole file
            None => resp,
            // Try to deliver partial content
            Some(Range::Bytes(v)) => {
                if let Ok(partial_file) = PartialFile::from_path(&file_path, v) {
                    Response::with((
                        status::Ok,
                        partial_file,
                        Header(AcceptRanges(vec![RangeUnit::Bytes])),
                    ))
                } else {
                    Response::with(status::NotFound)
                }
            }
            Some(_) => Response::with(status::RangeNotSatisfiable),
        };

        Ok(resp)
    }
}

struct StaticFileWithMetadata {
    file: File,
    metadata: Metadata,
    is_gz: bool,
}

impl StaticFileWithMetadata {
    pub fn search<P>(
        path: P,
        allow_gz: bool,
    ) -> Result<StaticFileWithMetadata, Box<dyn error::Error>>
    // TODO: unbox
    where
        P: Into<PathBuf>,
    {
        let mut file_path = path.into();
        trace!("Opening {}", file_path.display());
        let mut file = StaticFileWithMetadata::open(&file_path)?;

        // Look for index.html inside of a directory
        if file.metadata.is_dir() {
            file_path.push("index.html");
            trace!("Redirecting to index {}", file_path.display());
            file = StaticFileWithMetadata::open(&file_path)?;
        }

        if file.metadata.is_file() {
            if allow_gz {
                // Find the foo.gz sibling of foo
                let mut side_by_side_path: OsString = file_path.into();
                side_by_side_path.push(".gz");
                file_path = side_by_side_path.into();
                trace!("Attempting to find side-by-side GZ {}", file_path.display());

                match StaticFileWithMetadata::open(&file_path) {
                    Ok(mut gz_file) => {
                        if gz_file.metadata.is_file() {
                            gz_file.is_gz = true;
                            Ok(gz_file)
                        } else {
                            Ok(file)
                        }
                    }
                    Err(_) => Ok(file),
                }
            } else {
                Ok(file)
            }
        } else {
            Err(From::from("Requested path was not a regular file"))
        }
    }

    fn open<P>(path: P) -> Result<StaticFileWithMetadata, Box<dyn error::Error>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let metadata = file.metadata()?;

        Ok(StaticFileWithMetadata {
            file,
            metadata,
            is_gz: false,
        })
    }

    pub fn last_modified(&self) -> Result<time::Tm, Box<dyn error::Error>> {
        let modified = self.metadata.modified()?;
        let since_epoch = modified.duration_since(UNIX_EPOCH)?;

        // HTTP times don't have nanosecond precision, so we truncate
        // the modification time.
        // Converting to i64 should be safe until we get beyond the
        // planned lifetime of the universe
        //
        // TODO: Investigate how to write a test for this. Changing
        // the modification time of a file with greater than second
        // precision appears to be something that only is possible to
        // do on Linux.
        let ts = time::Timespec::new(since_epoch.as_secs() as i64, 0);
        Ok(time::at_utc(ts))
    }
}

#[cfg(test)]
mod test {
    extern crate hyper;
    extern crate iron_test;
    extern crate tempdir;

    use super::*;

    use std::fs::{DirBuilder, File};
    use std::path::{Path, PathBuf};

    use self::hyper::header::Headers;
    use self::iron_test::request;
    use self::tempdir::TempDir;
    use iron::status;

    struct TestFilesystemSetup(TempDir);

    impl TestFilesystemSetup {
        fn new() -> Self {
            TestFilesystemSetup(TempDir::new("test").expect("Could not create test directory"))
        }

        fn path(&self) -> &Path {
            self.0.path()
        }

        fn dir(&self, name: &str) -> PathBuf {
            let p = self.path().join(name);
            DirBuilder::new()
                .recursive(true)
                .create(&p)
                .expect("Could not create directory");
            p
        }

        fn file(&self, name: &str) -> PathBuf {
            let p = self.path().join(name);
            File::create(&p).expect("Could not create file");
            p
        }
    }

    #[test]
    fn staticfile_resolves_paths() {
        let fs = TestFilesystemSetup::new();
        fs.file("index.html");
        let fs2 = TestFilesystemSetup::new();
        fs2.dir("assets");

        let sf = Staticfile::new(fs.path(), fs2.path()).unwrap();
        let path = sf.resolve_path(&["index.html"]);
        assert!(path.unwrap().ends_with("index.html"));
    }

    #[test]
    fn staticfile_resolves_nested_paths() {
        let fs = TestFilesystemSetup::new();
        fs.dir("dir");
        fs.file("dir/index.html");
        let fs2 = TestFilesystemSetup::new();
        fs2.file("assets");

        let sf = Staticfile::new(fs.path(), fs2.path()).unwrap();
        let path = sf.resolve_path(&["dir", "index.html"]);
        assert!(path.unwrap().ends_with("dir/index.html"));
    }

    #[test]
    fn staticfile_disallows_resolving_out_of_root() {
        let fs = TestFilesystemSetup::new();
        fs.file("naughty.txt");
        let dir = fs.dir("dir");
        let fs2 = TestFilesystemSetup::new();
        let dir2 = fs2.file("assets");

        let sf = Staticfile::new(dir, dir2).unwrap();
        let path = sf.resolve_path(&["..", "naughty.txt"]);
        assert!(path.is_err());
    }

    #[test]
    fn staticfile_disallows_post_requests() {
        let fs = TestFilesystemSetup::new();
        let fs2 = TestFilesystemSetup::new();
        let sf = Staticfile::new(fs.path(), fs2.path()).unwrap();

        let response = request::post("http://127.0.0.1/", Headers::new(), "", &sf);

        let response = response.expect("Response was an error");
        assert_eq!(response.status, Some(status::MethodNotAllowed));
    }
}
