use crate::Route;

pub type BoxedRoute = Box<dyn Route>;

pub trait Application: Copy + Send + Sync + Sized {
  fn routes(&self) -> Vec<BoxedRoute>;

  fn route(&self, path: &str) -> Option<BoxedRoute> {
    for route in self.routes() {
      if route.path() == path {
        return Some(route);
      }
    }

    None
  }
}