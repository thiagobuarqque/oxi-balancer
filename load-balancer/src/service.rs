use std::sync::atomic::{AtomicU32, Ordering};
use log::{debug, info};

pub struct Service {
    address: String,
    connections_count: AtomicU32,
    status: ServiceStatus,
}

impl Service {
    pub fn new(address: String, status: ServiceStatus) -> Self {
        Self {
            address,
            connections_count: AtomicU32::new(0),
            status,
        }
    }

    pub fn healthy(address: String) -> Self {
        Self {
            address,
            connections_count: AtomicU32::new(0),
            status: ServiceStatus::HEALTHY,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn status(&self) -> &ServiceStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: ServiceStatus) {
        self.status = status
    }

    pub fn connections_count(&self) -> u32 {
        self.connections_count.load(Ordering::Relaxed)
    }

    pub fn increment_connections_count(&self) {
        debug!("Increment connection count for: {}", self.address);
        self.connections_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_connections_count(&self) {
        debug!("Decrementing connection count for: {}", self.address);
        self.connections_count.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Clone, PartialEq)]
pub enum ServiceStatus {
    HEALTHY,
    UNHEALTHY,
}
