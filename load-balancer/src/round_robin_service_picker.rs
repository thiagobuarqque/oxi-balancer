use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::service::{Service, ServiceStatus};
use crate::service_picker::ServicePicker;

pub struct RoundRobinServicePicker {
    index: usize
}

impl RoundRobinServicePicker {
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl ServicePicker for RoundRobinServicePicker {
    fn get_service_index(&mut self, _client: SocketAddr, services: Arc<Vec<Mutex<Service>>>) -> Result<usize, String> {
        if services.len() == 1 {
            return Ok(0);
        }

        for _ in 0..(services.len() - 1) {
            self.index = (self.index + 1) % services.len();

            let service_mutex = services.get(self.index).unwrap();

            let service_guard = service_mutex.lock().unwrap();

            if *service_guard.status() == ServiceStatus::HEALTHY {
                drop(service_guard);

                return Ok(self.index)
            }

            drop(service_guard);
        }

        Err("No healthy service available".to_string())
    }
}