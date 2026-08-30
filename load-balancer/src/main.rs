use log::info;
use std::env;
use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("No backend service address specified.");
    }

    let addresses = Vec::from(&args[1..]);

    info!("Redirecting traffic to backend services {:?}", addresses.clone());

    let listener = TcpListener::bind(format!("127.0.0.1:{}", 8080)).await?;

    let mut services_index = 0;

    loop {
        let (mut socket, _) = listener.accept().await?;

        let addr = addresses[services_index].clone();

        info!("Redirecting user TCP session to {:?}", addr.clone());

        tokio::spawn(async move {
            let mut outbound = match TcpStream::connect(addr.clone()).await {
                Ok(mut outbound) => {

                    if let Err(e) = copy_bidirectional(&mut socket, &mut outbound).await {
                        println!("Failed to transfer; error={e}");
                    }

                },
                Err(e) => {
                    println!("Failed to connect to server {}", addr);
                    return;
                }
            };
        });

        services_index = (services_index + 1) % addresses.len();
    }
}
