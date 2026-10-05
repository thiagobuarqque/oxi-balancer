use crate::service::{Service, ServiceStatus};
use crate::service_picker::ServicePicker;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub struct RoundRobinServicePicker {
    index: usize,
}

impl RoundRobinServicePicker {
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl ServicePicker for RoundRobinServicePicker {
    fn get_service_index(
        &mut self,
        _client: SocketAddr,
        services: Arc<Vec<Mutex<Service>>>,
    ) -> Result<usize, String> {
        for _ in 0..services.len() {
            let index = self.index;

            let service_mutex = services.get(index).unwrap();

            let service_guard = service_mutex.lock().unwrap();

            self.index = (self.index + 1) % services.len();

            if *service_guard.status() == ServiceStatus::HEALTHY {
                drop(service_guard);

                return Ok(index);
            }

            drop(service_guard);
        }

        Err("No healthy service available".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_wraps_around_on_list() {
        let mut round_robin_service_picker = RoundRobinServicePicker::new();

        let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let services: Vec<Mutex<Service>> = vec![
            Mutex::new(Service::healthy("127.0.0.1:8081".to_string())),
            Mutex::new(Service::healthy("127.0.0.1:8082".to_string())),
            Mutex::new(Service::healthy("127.0.0.1:8083".to_string())),
        ];

        let arc_services = Arc::new(services);

        for i in 0..(arc_services.len() * 2) {
            let index = round_robin_service_picker
                .get_service_index(socket_address, arc_services.clone())
                .unwrap();

            assert_eq!(index, i % arc_services.len());
        }
    }

    #[test]
    fn test_unhealthy_service_is_ignored() {
        let mut round_robin_service_picker = RoundRobinServicePicker::new();

        let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let services: Vec<Mutex<Service>> = vec![
            Mutex::new(Service::new(
                "127.0.0.1:8081".to_string(),
                ServiceStatus::UNHEALTHY,
            )),
            Mutex::new(Service::healthy("127.0.0.1:8082".to_string())),
        ];

        let arc_services = Arc::new(services);

        let index = round_robin_service_picker
            .get_service_index(socket_address, arc_services.clone())
            .unwrap();

        assert_eq!(index, 1);

        let index = round_robin_service_picker
            .get_service_index(socket_address, arc_services)
            .unwrap();

        assert_eq!(index, 1);
    }
}
