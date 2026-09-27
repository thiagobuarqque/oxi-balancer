pub mod app;
pub mod config;
mod service;

use crate::app::App;
use crate::config::{Algorithm, Config};
use crate::service::Service;
use log::{debug, info};
use std::env;
use std::sync::{Arc, Mutex};
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

    let app = Arc::new(App::new(
        Config::new(Algorithm::RoundRobin),
        to_services(addresses.as_slice()),
    ));

    let listener = TcpListener::bind(format!("127.0.0.1:{}", 8080)).await?;

    let mut services_index = 0;

    loop {
        let (mut socket, _) = listener.accept().await?;

        let _app = Arc::clone(&app);

        tokio::spawn(async move {
            let (services, index) = {
                let services = _app.get_services();
                let index = services_index % services.len();

                (services, index)
            };

            let service = services.get(services_index).unwrap();

            // TODO: handle ignoring UNHEALTHY service

            let address = {
                let service_guard = service.lock().unwrap();

                let address = service_guard.address().to_string();

                address
            };

            debug!("Redirecting user TCP session to {:?}", address);

            match TcpStream::connect(address).await {
                Ok(mut outbound) => {
                    {
                        let service_guard = service.lock().unwrap();

                        service_guard.increment_connections_count();

                        drop(service_guard);

                        print_servers(_app.clone());
                    };

                    match copy_bidirectional(&mut socket, &mut outbound).await {
                        Ok((a_count, b_count)) => {
                            let service_guard = service.lock().unwrap();

                            service_guard.decrement_connections_count();

                            drop(service_guard);

                            print_servers(_app.clone());
                        }
                        Err(err) => {
                            let service_guard = service.lock().unwrap();

                            service_guard.decrement_connections_count();

                            drop(service_guard);

                            print_servers(_app.clone());

                            println!("Failed to transfer; error={err}");
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to connect to server \nMessage: {e}");
                }
            };
        });

        services_index = (services_index + 1) % addresses.len();
    }
}

fn print_servers(app: Arc<App>) {
    let services = app.get_services();

    services.iter().for_each(|service| {
        let service_guard = service.lock().unwrap();

        println!(
            "Server: {} | Count: {}",
            service_guard.address(),
            service_guard.connections_count()
        );
    });
}

pub fn to_services(addresses: &[String]) -> Vec<Mutex<Service>> {
    addresses
        .into_iter()
        .map(|address| Mutex::new(Service::healthy(address.clone())))
        .collect::<Vec<_>>()
}
