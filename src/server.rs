use tokio::io::AsyncWriteExt;
use crate::db::Db;
use crate::resp::{Resp, RespValue};
use tokio::net::{TcpListener, TcpStream};
use crate::command::Command;

#[derive(Debug)]
struct Server {
    listener: TcpListener,
    db: Db,
}

impl Server {
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let socket = self.accept().await?;

            let mut client = Client::new(socket, self.db.clone());

            tokio::spawn(async move {
                if let Err(e) = client.handle().await {
                    println!("[!] client error = {}", e);
                }
            });
        }
    }

    async fn accept(&mut self) -> Result<TcpStream, Box<dyn std::error::Error>> {
        match self.listener.accept().await {
            Ok((socket, _)) => Ok(socket),
            Err(e) => {
                println!("[!] accept error = {:?}", e);
                Err(e.into())
            }
        }
    }
}

pub struct Client {
    pub socket: TcpStream,
    pub db: Db,
}

impl Client {
    fn new(socket: TcpStream, db: Db) -> Self {
        Self { socket, db }
    }

    async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match Resp::decode(&mut self.socket).await {
                Ok(request) => {
                    println!("request: {:?}", request);
                    let cmd = Command::from_resp(request)?;
                    println!("command: {:?}", cmd);
                    cmd.execute(self).await?;
                    self.socket.flush().await?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    println!("client disconnected cleanly");
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                    println!("client crashed or killed");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("protocol error: {e}");
                    return Err(e.into());
                }
            }
        }
    }
}

pub async fn run(listener: TcpListener, db: Db) {
    let mut server = Server { listener, db };

    server.run().await.expect("Server error");
}
