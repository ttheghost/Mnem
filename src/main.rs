pub mod db;
pub mod resp;
pub mod server;
pub mod value;

use crate::server::run;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:6379").await?;

    run(listener).await;
    Ok(())
}
