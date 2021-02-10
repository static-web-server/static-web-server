use humansize::{file_size_opts, FileSize};
use iron::headers::{
    AcceptEncoding, AcceptRanges, ContentEncoding, ContentLength, Encoding, HttpDate,
    IfModifiedSince, LastModified, Range, RangeUnit,
};
use iron::method::Method;
use iron::middleware::Handler;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use std::fs::{File, Metadata};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use std::{error, io};
use std::{ffi::OsString, time::SystemTime};

use crate::staticfile_middleware::helpers;
use crate::staticfile_middleware::partial_file::PartialFile;

/// Recursively serves files from the specified root and assets directories.
pub struct Staticfile {
    root: PathBuf,
    assets: PathBuf,
    dir_list: bool,
}

impl Staticfile {
    pub fn new<P>(root: P, assets: P, dir_list: bool) -> io::Result<Staticfile>
    where
        P: AsRef<Path>,
    {
        let root = root.as_ref().canonicalize()?;
        let assets = assets.as_ref().canonicalize()?;

        Ok(Staticfile {
            root,
            assets,
            dir_list,
        })
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

        // 1. Check if directory listing feature is enabled,
        // if current path is a valid directory and
        // if it does not contain an index.html file
        if self.dir_list && file_path.is_dir() && !file_path.join("index.html").exists() {
            let encoding = Encoding::Identity;
            let readir = match std::fs::read_dir(file_path) {
                Ok(dir) => dir,
                Err(err) => {
                    error!("{}", err);
                    return Ok(Response::with(status::InternalServerError));
                }
            };

            let mut current_path = req
                .url
                .path()
                .into_iter()
                .map(|i| format!("/{}", i))
                .collect::<String>();

            // Redirect if current path does not end with slash
            if !current_path.ends_with('/') {
                let mut u: url::Url = req.url.clone().into();
                current_path.push('/');
                u.set_path(&current_path);

                let url = iron::Url::from_generic_url(u).expect("Unable to parse redirect url");

                return Ok(Response::with((
                    status::PermanentRedirect,
                    iron::modifiers::Redirect(url),
                )));
            }

            // Read current directory and create the index page
            let mut entries_str = String::new();
            if current_path != "/" {
                entries_str =
                    String::from("<tr><td colspan=\"3\"><a href=\"../\">../</a></td></tr>");
            }
            for entry in readir {
                let entry = entry.unwrap();
                let meta = entry.metadata().unwrap();
                let mut filesize = meta.len().file_size(file_size_opts::DECIMAL).unwrap();
                let mut name = entry.file_name().into_string().unwrap();
                if meta.is_dir() {
                    name = format!("{}/", name);
                    filesize = String::from("-")
                }
                let uri = format!("{}{}", current_path, name);
                let modified = get_last_modified(meta.modified().unwrap()).unwrap();

                entries_str = format!(
                    "{}<tr><td><a href=\"{}\" title=\"{}\">{}</a></td><td style=\"width: 160px;\">{}</td><td align=\"right\" style=\"width: 140px;\">{}</td></tr>",
                    entries_str,
                    uri,
                    name,
                    name,
                    modified.to_local().strftime("%F %T").unwrap(),
                    filesize
                );
            }

            let page = format!(
                "<html><head><title>Index of {}</title></head><body><h1>Index of {}</h1><table style=\"min-width:680px;\"><tr><th colspan=\"3\"><hr></th></tr>{}<tr><th colspan=\"3\"><hr></th></tr></table></body></html>", current_path, current_path, entries_str
            );
            let len = page.len() as u64;
            let content_encoding = ContentEncoding(vec![encoding]);
            let mut resp = Response::with((status::Ok, Header(content_encoding), page));

            // Empty current response body on HEAD requests,
            // just setting up the `content-length` header (size of the file in bytes)
            // https://tools.ietf.org/html/rfc7231#section-4.3.2
            if req.method == Method::Head {
                resp.set_mut(vec![]);
                resp.set_mut(Header(ContentLength(len)));
            }

            return Ok(resp);
        }

        // 2. Otherwise proceed with the normal file-response process

        // Get current file metadata
        let accept_gz = helpers::accept_gzip(req.headers.get::<AcceptEncoding>());
        let file = match StaticFileWithMetadata::search(&file_path, accept_gz) {
            Ok(f) => f,
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

        // Prepare response
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
        resp = match req.headers.get::<Range>().cloned() {
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
        get_last_modified(self.metadata.modified()?)
    }
}

fn get_last_modified(modified: SystemTime) -> Result<time::Tm, Box<dyn error::Error>> {
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

        let sf = Staticfile::new(fs.path(), fs2.path(), false).unwrap();
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

        let sf = Staticfile::new(fs.path(), fs2.path(), false).unwrap();
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

        let sf = Staticfile::new(dir, dir2, false).unwrap();
        let path = sf.resolve_path(&["..", "naughty.txt"]);
        assert!(path.is_err());
    }

    #[test]
    fn staticfile_disallows_post_requests() {
        let fs = TestFilesystemSetup::new();
        let fs2 = TestFilesystemSetup::new();
        let sf = Staticfile::new(fs.path(), fs2.path(), false).unwrap();

        let response = request::post("http://127.0.0.1/", Headers::new(), "", &sf);

        let response = response.expect("Response was an error");
        assert_eq!(response.status, Some(status::MethodNotAllowed));
    }
}
