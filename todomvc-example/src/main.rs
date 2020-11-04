mod model;
mod routes;

use pinhole::{Application, Result, Route};

pub fn main() -> Result<()> {
    pinhole::run(TodoApplication, "0.0.0.0:8080".to_string())
}

#[derive(Copy, Clone)]
struct TodoApplication;

impl Application for TodoApplication {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![Box::new(routes::IndexRoute), Box::new(routes::ListRoute)]
    }
}
