use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::service::Service;

pub trait ServicePicker {
    fn get_service_index(&mut self, client: SocketAddr, services: Arc<Vec<Mutex<Service>>>) -> Result<usize, String>;
}