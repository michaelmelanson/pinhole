use crate::{Params, Route};

pub type BoxedRoute = Box<dyn Route>;

pub trait Application: Copy + Send + Sync + Sized {
    fn routes(&self) -> Vec<BoxedRoute>;

    fn route(&self, path: &str) -> Option<(BoxedRoute, Params)> {
        for route in self.routes() {
            if let Some(params) = route.pattern().matches(path) {
                return Some((route, params));
            }
        }

        None
    }
}
