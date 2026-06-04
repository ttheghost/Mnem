pub mod db;
pub mod resp;
pub mod server;
pub mod value;
pub mod command;

use crate::db::new_db;
use crate::server::run;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:6379").await?;
    let db = new_db();

    run(listener, db).await;
    Ok(())
}
