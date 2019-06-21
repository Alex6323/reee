//! Environment module.

use std::cell::RefCell;
use std::rc::Rc;

use bus::Bus;
use crossbeam_channel::Receiver;
use tokio::prelude::*;

use crate::eee::entity::Entity;
use crate::errors::Error;

const BUS_SIZE: usize = 100;

/// An environment in the EEE model.
pub struct Environment {
    /// name of the environment
    pub name: String,
    /// entities that joined this environment
    joined_entities: Rc<RefCell<Vec<Entity>>>,
    /// entities that affect this environment
    affecting_entities: Rc<RefCell<Vec<Entity>>>,
    /// a receiving channel that to the supervisor
    in_chan: Receiver<String>,
    /// the bus via which this environment will be broadcasting
    out_chan: Rc<RefCell<Bus<String>>>,
}

impl Environment {
    /// Creates a new environment.
    pub fn new(name: &str, in_chan: Receiver<String>) -> Self {
        Self {
            name: name.into(),
            joined_entities: Rc::new(RefCell::new(vec![])),
            affecting_entities: Rc::new(RefCell::new(vec![])),
            in_chan,
            out_chan: Rc::new(RefCell::new(Bus::new(BUS_SIZE))),
        }
    }
    /// Add an  entity that joins this evironment.
    pub fn join_entity(&mut self, mut entity: Entity) -> Result<(), Error> {
        let rx = self.out_chan.borrow_mut().add_rx();

        entity.join_environment(&self.name, rx)?;
        self.joined_entities.borrow_mut().push(entity);
        Ok(())
    }

    /// Add an entity that affects this environment.
    pub fn add_affector(&mut self, mut entity: Entity) -> Result<(), Error> {
        entity.affect_environment(&self.name)?;

        self.affecting_entities.borrow_mut().push(entity);
        Ok(())
    }
}

impl Future for Environment {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            match self.in_chan.try_recv() {
                Err(_) => return Ok(Async::NotReady),
                Ok(effect) => {
                    println!("env '{}' received effect: {}", self.name, effect);
                    self.out_chan.borrow_mut().broadcast(effect);
                }
            }
        }
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            joined_entities: Rc::clone(&self.joined_entities),
            affecting_entities: Rc::clone(&self.affecting_entities),
            in_chan: self.in_chan.clone(),
            out_chan: self.out_chan.clone(),
        }
    }
}
