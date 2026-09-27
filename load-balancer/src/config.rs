use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{DataChange, ModifyKind},
};
use std::path::Path;

#[derive(Copy, Clone)]
pub enum Algorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnection,
    WeightedLeastConnection,
    ResourceBased,
}

pub struct Config {
    algorithm: Algorithm,
}

impl Config {
    pub fn new(algorithm: Algorithm) -> Self {
        Self { algorithm }
    }

    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }
}
