mod routes;
mod stylesheet;

use pinhole::{Application, Result, Route, TlsConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Load TLS configuration
    // For development, use: scripts/generate_dev_cert.sh to create cert.pem and key.pem
    let tls_config = TlsConfig::new("cert.pem", "key.pem");

    pinhole::run(ShowcaseApplication, "0.0.0.0:3100", tls_config).await
}

#[derive(Copy, Clone)]
struct ShowcaseApplication;

impl Application for ShowcaseApplication {
    fn routes(&self) -> Vec<Box<dyn Route>> {
        vec![Box::new(routes::ShowcaseRoute)]
    }
}
