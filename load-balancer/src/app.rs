use crate::config::{Algorithm, Config};
use crate::least_connection_service_picker::LeastConnectionServicePicker;
use crate::round_robin_service_picker::RoundRobinServicePicker;
use crate::service::Service;
use crate::service_picker::ServicePicker;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub struct App {
    config: Config,
    services: Arc<Vec<Mutex<Service>>>,
    service_picker: Box<dyn ServicePicker + Send + Sync>,
}

impl App {
    pub fn new(config: Config, services: Vec<Mutex<Service>>) -> Self {
        let service_picker: Box<dyn ServicePicker + Send + Sync> = match config.algorithm() {
            Algorithm::RoundRobin => Box::new(RoundRobinServicePicker::new()),
            Algorithm::LeastConnection => Box::new(LeastConnectionServicePicker::new()),
            _ => Box::new(RoundRobinServicePicker::new()),
        };

        Self {
            config,
            services: Arc::new(services),
            service_picker,
        }
    }

    pub fn get_services(&self) -> Arc<Vec<Mutex<Service>>> {
        self.services.clone()
    }

    pub fn get_service_index(&mut self, client_addr: SocketAddr) -> Option<usize> {
        match self
            .service_picker
            .get_service_index(client_addr, self.services.clone())
        {
            Ok(index) => Some(index),
            Err(err) => Option::None,
        }
    }
}
