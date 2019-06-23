//! Signaling trigger events across asynchronous tasks.
use crate::errors::Error;

use tokio::sync::watch::{
    self,
    Receiver,
    Sender,
};

pub struct TriggerHandle(pub Receiver<bool>);

pub(crate) struct Trigger {
    trigger: Sender<bool>,
    handle: Receiver<bool>,
}

impl Trigger {
    pub fn new() -> Self {
        let (trigger, handle) = watch::channel(false);
        Self { trigger, handle }
    }

    pub fn get_handle(&self) -> TriggerHandle {
        TriggerHandle(self.handle.clone())
    }

    pub fn pull(&mut self) -> Result<(), Error> {
        Ok(self.trigger.broadcast(true)?)
    }
}
