//! Watcher

use futures::task::AtomicTask;
use std::sync::Arc;

pub struct Watcher {
    pub task: Arc<AtomicTask>,
}

impl Watcher {
    pub fn new() -> Self {
        Watcher { task: Arc::new(AtomicTask::new()) }
    }
}

impl Clone for Watcher {
    fn clone(&self) -> Self {
        Self { task: Arc::clone(&self.task) }
    }
}
