use std::{env, thread};
use std::thread::Thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::sleep;
use log::info;
use rand::RngExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Missing port number argument");
    }

    let port = args[1].clone().parse::<u16>()?;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back

                let random_millis: u64 = rand::random_range(500..=3000);

                info!("Sleeping for {}ms...", random_millis);

                sleep(Duration::from_millis(random_millis)).await;

                let prefix = format!("Server at {} says: ", port);

                let mut message = Vec::from(prefix.as_bytes());

                message.extend_from_slice(&buf);

                if let Err(e) = socket.write_all(&message).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
