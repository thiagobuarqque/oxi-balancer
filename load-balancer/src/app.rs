use crate::config::Config;
use crate::service::Service;
use std::sync::{Arc, Mutex};

pub struct App {
    config: Config,
    services: Arc<Vec<Mutex<Service>>>,
}

impl App {
    pub fn new(config: Config, services: Vec<Mutex<Service>>) -> Self {
        Self {
            config,
            services: Arc::new(services),
        }
    }

    pub fn get_services(&self) -> Arc<Vec<Mutex<Service>>> {
        self.services.clone()
    }
}
