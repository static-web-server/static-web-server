use iron::modifier::Modifier;
use iron::prelude::*;
use iron::AfterMiddleware;

/// Applies a modifier to every request that starts with a given path.
pub struct Prefix<M> {
    prefix: Vec<String>,
    modifier: M,
}

impl<M> Prefix<M> {
    pub fn new<P, S>(prefix: P, modifier: M) -> Prefix<M>
    where
        P: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Prefix {
            prefix: prefix.into_iter().map(|s| s.as_ref().into()).collect(),
            modifier,
        }
    }

    fn prefix_matches(&self, path: &[&str]) -> bool {
        if self.prefix.len() > path.len() {
            return false;
        }

        path.iter()
            .zip(self.prefix.iter())
            .all(|(path, prefix)| path == prefix)
    }
}

impl<M> AfterMiddleware for Prefix<M>
where
    M: Clone + Modifier<Response> + Send + Sync + 'static,
{
    fn after(&self, req: &mut Request<'_, '_>, mut res: Response) -> IronResult<Response> {
        if self.prefix_matches(&req.url.path()) {
            self.modifier.clone().modify(&mut res);
        }

        Ok(res)
    }
}
