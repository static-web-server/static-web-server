use humansize::{file_size_opts, FileSize};
use iron::headers::{
    AcceptRanges, ContentEncoding, ContentLength, Encoding, HttpDate, IfModifiedSince,
    LastModified, Range, RangeUnit,
};
use iron::method::Method;
use iron::middleware::Handler;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use std::fs::{File, Metadata};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{error, io};

use crate::helpers;
use crate::staticfile_middleware::partial_file::PartialFile;

/// Recursively serves files from the specified root and assets directories.
pub struct Staticfile {
    root: PathBuf,
    assets: PathBuf,
    dir_listing: bool,
}

impl Staticfile {
    pub fn new<P: AsRef<Path>>(
        root_dir: P,
        assets_dir: P,
        dir_listing: bool,
    ) -> io::Result<Staticfile>
    where
        PathBuf: From<P>,
    {
        Ok(Staticfile {
            root: root_dir.into(),
            assets: assets_dir.into(),
            dir_listing,
        })
    }

    fn resolve_path(&self, path: &[&str]) -> Result<PathBuf, Box<dyn error::Error>> {
        let current_dirname = path[0];
        let asserts_dirname = self.assets.iter().last().unwrap().to_str().unwrap();
        let mut is_assets = false;

        let path_resolved = if current_dirname == asserts_dirname {
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

        let base_path = if is_assets { &self.assets } else { &self.root };
        let path_resolved = PathBuf::from(helpers::adjust_canonicalization(
            path_resolved.canonicalize()?,
        ));

        // Protect against path/directory traversal
        if !path_resolved.starts_with(&base_path) {
            return Err(From::from(format!(
                "Cannot leave {:?} base path",
                &base_path
            )));
        }

        Ok(path_resolved)
    }
}

impl Handler for Staticfile {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // Accept only HEAD and GET methods
        if !(req.method == Method::Head || req.method == Method::Get) {
            return Ok(Response::with(status::MethodNotAllowed));
        }

        // Resolve path on file system
        let path_resolved = match self.resolve_path(&req.url.path()) {
            Ok(file_path) => file_path,
            Err(e) => {
                trace!("{}", e);
                return Ok(Response::with(status::NotFound));
            }
        };

        // 1. Check if "directory listing" feature is enabled,
        // if current path is a valid directory and
        // if it does not contain an index.html file
        if self.dir_listing && path_resolved.is_dir() && !path_resolved.join("index.html").exists()
        {
            let read_dir = match std::fs::read_dir(path_resolved) {
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
            for entry in read_dir {
                let entry = entry.unwrap();
                let meta = entry.metadata().unwrap();
                let mut filesize = meta.len().file_size(file_size_opts::DECIMAL).unwrap();
                let mut name = entry.file_name().into_string().unwrap();
                if meta.is_dir() {
                    name = format!("{}/", name);
                    filesize = String::from("-")
                }
                let uri = format!("{}{}", current_path, name);
                let modified = parse_last_modified(meta.modified().unwrap()).unwrap();

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

            let page_str = format!(
                "<html><head><title>Index of {}</title></head><body><h1>Index of {}</h1><table style=\"min-width:680px;\"><tr><th colspan=\"3\"><hr></th></tr>{}<tr><th colspan=\"3\"><hr></th></tr></table></body></html>", current_path, current_path, entries_str
            );
            let len = page_str.len() as u64;
            let content_encoding = ContentEncoding(vec![Encoding::Identity]);
            let mut resp = Response::with((status::Ok, Header(content_encoding), page_str));

            // Empty current response body on HEAD requests,
            // just setting up the `content-length` header (size of the file in bytes)
            // https://tools.ietf.org/html/rfc7231#section-4.3.2
            if req.method == Method::Head {
                resp.set_mut(vec![]);
                resp.set_mut(Header(ContentLength(len)));
            }

            return Ok(resp);
        }

        // 2. Otherwise proceed with the normal file-  process

        // Search a file and its metadata by the resolved path
        let static_file = match StaticFileWithMeta::search(path_resolved.clone()) {
            Ok(f) => f,
            Err(e) => {
                trace!("{}", e);
                return Ok(Response::with(status::NotFound));
            }
        };

        // Apply last modified date time
        let client_last_mod = req.headers.get::<IfModifiedSince>();
        let last_mod = static_file.last_modified().ok().map(HttpDate);

        if let (Some(client_last_mod), Some(last_mod)) = (client_last_mod, last_mod) {
            trace!(
                "Comparing {} (file) <= {} (req)",
                last_mod,
                client_last_mod.0
            );

            if last_mod <= client_last_mod.0 {
                return Ok(Response::with(status::NotModified));
            }
        }

        // Prepare response object
        let mut resp = match last_mod {
            Some(last_mod) => {
                Response::with((status::Ok, Header(LastModified(last_mod)), static_file.file))
            }
            None => Response::with((status::Ok, static_file.file)),
        };

        // Empty current response body on HEAD requests,
        // just setting up the `content-length` header (size of the file in bytes)
        // https://tools.ietf.org/html/rfc7231#section-4.3.2
        if req.method == Method::Head {
            resp.set_mut(vec![]);
            resp.set_mut(Header(ContentLength(static_file.meta.len())));
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
                if let Ok(partial_file) = PartialFile::from_path(&path_resolved, v) {
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

/// It represents a regular source file in file system with its metadata.
struct StaticFileWithMeta {
    file: File,
    meta: Metadata,
}

impl StaticFileWithMeta {
    /// Search for a regular source file in file system.
    /// If source file is a directory then it attempts to search for an index.html.
    pub fn search(mut src: PathBuf) -> Result<StaticFileWithMeta, Box<dyn error::Error>> {
        trace!("Opening {}", src.display());

        let mut auto_index = false;
        let meta = std::fs::metadata(&src)?;

        // Look for an `index.html` file inside of a directory
        if meta.is_dir() {
            src.push("index.html");
            auto_index = true;
            trace!("Redirecting to index {}", src.display());
        }

        // Attempt to open source file in read-only mode
        let file = File::open(src)?;
        let meta = if auto_index { file.metadata()? } else { meta };

        if meta.is_file() {
            Ok(StaticFileWithMeta { file, meta })
        } else {
            Err(From::from("Requested path was not a regular file"))
        }
    }

    /// Get the last modification time of current file.
    pub fn last_modified(&self) -> Result<time::Tm, Box<dyn error::Error>> {
        parse_last_modified(self.meta.modified()?)
    }
}

fn parse_last_modified(modified: SystemTime) -> Result<time::Tm, Box<dyn error::Error>> {
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
