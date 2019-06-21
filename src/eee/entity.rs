//! Entity

use std::cell::RefCell;
use std::collections::{
    HashMap,
    HashSet,
};
use std::rc::Rc;

use bus::BusReader as Receiver;

use tokio::{
    io,
    prelude::*,
};
use uuid::Uuid;

use crate::errors::Error;

/// An entity in the EEE model.
pub struct Entity {
    /// A unique identifier of this entity.
    pub uuid: String,
    joined_environments: Rc<RefCell<HashMap<String, Receiver<String>>>>,
    affected_environments: Rc<RefCell<HashSet<String>>>,
}

impl Entity {
    /// Registers an environment as joined by this entity.
    pub fn join_environment(&mut self, name: &str, rx: Receiver<String>) -> Result<(), Error> {
        if self.joined_environments.borrow().contains_key(name) {
            return Err(Error::App("This entity already joined that environment"));
        }

        // Store the name and the receiver handle of that environment
        self.joined_environments.borrow_mut().insert(name.into(), rx);
        Ok(())
    }

    /// Registers an environment as affected by this entity.
    pub fn affect_environment(&mut self, name: &str) -> Result<(), Error> {
        if self.affected_environments.borrow().contains(name) {
            return Err(Error::App("This entity already affects that environment"));
        }

        // Store the name and the receiver handle of that environment
        self.affected_environments.borrow_mut().insert(name.into());
        Ok(())
    }

    /// Returns true, if this entity has joined the specified environment, otherwise false.
    pub fn has_joined(&self, name: &str) -> bool {
        self.joined_environments.borrow().contains_key(name)
    }

    /// Returns true, if this entity has joined the specified environment, otherwise false.
    pub fn is_affecting(&self, name: &str) -> bool {
        self.affected_environments.borrow().contains(name)
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            joined_environments: Rc::new(RefCell::new(HashMap::new())),
            affected_environments: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

impl Future for Entity {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            let mut i = 0;
            {
                let mut environments = self.joined_environments.borrow_mut();
                for (env, receiver) in environments.iter_mut() {
                    //let data = receiver.recv().expect("error");
                    match receiver.try_recv() {
                        Err(_) => i += 1,
                        Ok(data) => {
                            println!(
                                "Entity '{}' received '{}' from environment '{}'",
                                self.uuid, data, env
                            );
                        }
                    }
                }
            }
            if i == self.joined_environments.borrow().len() {
                return Ok(Async::NotReady);
            }
        }
    }
}

impl Clone for Entity {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid.clone(),
            joined_environments: Rc::clone(&self.joined_environments),
            affected_environments: Rc::clone(&self.affected_environments),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a new entity
    #[test]
    fn new_entity() {
        let _ent = Entity::default();
    }
}
