use crate::resp::{Resp, RespValue};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
struct Server {
    listener: TcpListener,
}

pub async fn run(listener: TcpListener) {
    let mut server = Server { listener };

    server.run().await.expect("Server error");
}

impl Server {
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut socket = self.accept().await?;
            loop {
                let value = Resp::decode(&mut socket).await?;
                println!("{:?}", value);
                Resp::encode(RespValue::SimpleString("PONG".into()), &mut socket).await?;
                socket.flush().await?;
            }
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
