pub mod app;
pub mod config;
mod round_robin_service_picker;
mod service;
mod service_picker;

use crate::app::App;
use crate::config::{Algorithm, Config};
use crate::service::Service;
use log::{debug, info};
use std::env;
use std::sync::Mutex;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("No backend service address specified.");
    }

    let addresses = Vec::from(&args[1..]);

    info!(
        "Redirecting traffic to backend services {:?}",
        addresses.clone()
    );

    let mut app = App::new(
        Config::new(Algorithm::RoundRobin),
        to_services(addresses.as_slice()),
    );

    let listener = TcpListener::bind(format!("127.0.0.1:{}", 8080)).await?;

    loop {
        let (socket, client_address) = listener.accept().await?;
        let Some(index) = app.get_service_index(client_address) else {
            continue;
        };
        let services = app.get_services();

        tokio::spawn(async move { handle_client(&services[index], socket).await });
    }
}

async fn handle_client(target_service: &Mutex<Service>, mut origin_stream: TcpStream) {
    let address = target_service.lock().unwrap().address().to_string();

    debug!("Redirecting user TCP session to {:?}", address);

    match TcpStream::connect(address).await {
        Ok(mut target_stream) => {
            // Connection succeeded
            target_service.lock().unwrap().increment_connections_count();

            match copy_bidirectional(&mut origin_stream, &mut target_stream).await {
                Ok((a_count, b_count)) => {
                    // Both directions closed connection
                    target_service.lock().unwrap().decrement_connections_count();

                    info!(
                        "Connection closed successfully! | Data a -> b: {a_count} | Data b -> a: {b_count}"
                    );
                }
                Err(err) => {
                    // An error occurred
                    target_service.lock().unwrap().decrement_connections_count();

                    info!("Failed to transfer; error={err}");
                }
            }
        }
        Err(e) => {
            // Failed to connect
            info!("Failed to connect to server \nMessage: {e}");
        }
    }
}

pub fn to_services(addresses: &[String]) -> Vec<Mutex<Service>> {
    addresses
        .into_iter()
        .map(|address| Mutex::new(Service::healthy(address.clone())))
        .collect::<Vec<_>>()
}
