use iron::modifier::Modifier;
use iron::prelude::*;
use iron::AfterMiddleware;

/// Applies a modifier to every request.
pub struct ModifyWith<M> {
    modifier: M,
}

impl<M> ModifyWith<M> {
    pub fn new(modifier: M) -> ModifyWith<M> {
        ModifyWith { modifier }
    }
}

impl<M> AfterMiddleware for ModifyWith<M>
where
    M: Clone + Modifier<Response> + Send + Sync + 'static,
{
    fn after(&self, _req: &mut Request, mut res: Response) -> IronResult<Response> {
        self.modifier.clone().modify(&mut res);
        Ok(res)
    }
}
