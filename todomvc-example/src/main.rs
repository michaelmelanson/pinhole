mod model;
mod routes;
mod stylesheet;

use pinhole::{Application, Result, Route};

pub fn main() -> Result<()> {
    pinhole::run(TodoApplication, "0.0.0.0:8080")
}

#[derive(Copy, Clone)]
struct TodoApplication;

impl Application for TodoApplication {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![Box::new(routes::IndexRoute), Box::new(routes::ListRoute)]
    }
}
