use crate::service::{Service, ServiceStatus};
use crate::service_picker::ServicePicker;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub struct LeastConnectionServicePicker;

impl LeastConnectionServicePicker {
    pub fn new() -> Self {
        Self
    }
}

impl ServicePicker for LeastConnectionServicePicker {
    fn get_service_index(
        &mut self,
        _client: SocketAddr,
        services: Arc<Vec<Mutex<Service>>>,
    ) -> Result<usize, String> {
        let mut least: Option<(usize, u32)> = None;

        for (index, service_mutex) in services.iter().enumerate() {
            let service_guard = service_mutex.lock().unwrap();

            if *service_guard.status() != ServiceStatus::HEALTHY {
                continue;
            }

            let count = service_guard.connections_count();

            if least.map_or(true, |(_, least_count)| count < least_count) {
                least = Some((index, count));
            }
        }

        least
            .map(|(index, _)| index)
            .ok_or("No healthy service available".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn socket_address() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
    }

    fn service_with_connections(address: &str, count: u32) -> Mutex<Service> {
        let service = Service::healthy(address.to_string());

        for _ in 0..count {
            service.increment_connections_count();
        }

        Mutex::new(service)
    }

    #[test]
    fn test_picks_service_with_least_connections() {
        let mut least_connection_service_picker = LeastConnectionServicePicker::new();

        let services = Arc::new(vec![
            service_with_connections("127.0.0.1:8081", 3),
            service_with_connections("127.0.0.1:8082", 1),
            service_with_connections("127.0.0.1:8083", 2),
        ]);

        let index = least_connection_service_picker
            .get_service_index(socket_address(), services)
            .unwrap();

        assert_eq!(index, 1);
    }

    #[test]
    fn test_tie_picks_first_service() {
        let mut least_connection_service_picker = LeastConnectionServicePicker::new();

        let services = Arc::new(vec![
            service_with_connections("127.0.0.1:8081", 2),
            service_with_connections("127.0.0.1:8082", 1),
            service_with_connections("127.0.0.1:8083", 1),
        ]);

        let index = least_connection_service_picker
            .get_service_index(socket_address(), services)
            .unwrap();

        assert_eq!(index, 1);
    }

    #[test]
    fn test_follows_connection_count_changes() {
        let mut least_connection_service_picker = LeastConnectionServicePicker::new();

        let services = Arc::new(vec![
            service_with_connections("127.0.0.1:8081", 0),
            service_with_connections("127.0.0.1:8082", 0),
        ]);

        let index = least_connection_service_picker
            .get_service_index(socket_address(), services.clone())
            .unwrap();

        assert_eq!(index, 0);

        services[0].lock().unwrap().increment_connections_count();

        let index = least_connection_service_picker
            .get_service_index(socket_address(), services.clone())
            .unwrap();

        assert_eq!(index, 1);
    }

    #[test]
    fn test_unhealthy_service_is_ignored() {
        let mut least_connection_service_picker = LeastConnectionServicePicker::new();

        let services = Arc::new(vec![
            Mutex::new(Service::new(
                "127.0.0.1:8081".to_string(),
                ServiceStatus::UNHEALTHY,
            )),
            service_with_connections("127.0.0.1:8082", 5),
        ]);

        let index = least_connection_service_picker
            .get_service_index(socket_address(), services)
            .unwrap();

        assert_eq!(index, 1);
    }

    #[test]
    fn test_no_healthy_service_returns_error() {
        let mut least_connection_service_picker = LeastConnectionServicePicker::new();

        let services = Arc::new(vec![Mutex::new(Service::new(
            "127.0.0.1:8081".to_string(),
            ServiceStatus::UNHEALTHY,
        ))]);

        let result =
            least_connection_service_picker.get_service_index(socket_address(), services);

        assert!(result.is_err());
    }
}
