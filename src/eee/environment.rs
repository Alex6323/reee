//! Environment module.

use crate::common::trigger::{
    Trigger,
    TriggerHandle,
};
use crate::common::watcher::Watcher;
use crate::eee::entity::Entity;
use crate::errors::Error;

use std::sync::{
    Arc,
    Mutex,
};

use bus::Bus;
use crossbeam_channel::Receiver;
use tokio::prelude::*;

const BUS_SIZE: usize = 100;

/// An environment in the EEE model.
pub struct Environment {
    /// name of the environment
    pub name: Arc<String>,
    /// entities that joined this environment
    joined_entities: Arc<Mutex<Vec<EntityLink>>>,
    /// entities that affect this environment
    affecting_entities: Arc<Mutex<Vec<Entity>>>,
    /// the receiving side of a channel to the supervisor
    in_chan: Arc<Receiver<String>>,
    /// the outgoing broadcast channel
    out_chan: Arc<Mutex<Bus<String>>>,
    /// a notifier that signals the end of this environment to subscribed entities
    drop_notifier: Arc<Mutex<Trigger>>,
    /// a handle to signal supervisor shutdown
    shutdown_listener: Arc<Mutex<TriggerHandle>>,
    /// a notify that allows to wake this environments task/future
    waker: Watcher,
}

/// Link between environment and an entity.
struct EntityLink {
    entity: Entity,
    pub waker: Watcher,
}

impl Environment {
    /// Creates a new environment.
    pub fn new(name: &str, in_chan: Receiver<String>, shutdown_listener: TriggerHandle) -> Self {
        let waker = Watcher::new();
        Self {
            name: shared!(name.into()),
            joined_entities: shared_mut!(vec![]),
            affecting_entities: shared_mut!(vec![]),
            in_chan: shared!(in_chan),
            out_chan: shared_mut!(Bus::new(BUS_SIZE)),
            drop_notifier: shared_mut!(Trigger::new()),
            shutdown_listener: shared_mut!(shutdown_listener),
            waker,
        }
    }

    /// Registers an entity that wants to join this evironment.
    pub fn register_joining_entity(&mut self, mut entity: Entity) -> Result<(), Error> {
        //
        let out_chan = unlock!(self.out_chan).add_rx();

        let sig_term = unlock!(self.drop_notifier).get_handle();

        entity.join_environment(&self.name, out_chan, sig_term, self.waker.clone())?;

        let waker = entity.get_waker();
        let link = EntityLink { entity, waker };

        unlock!(self.joined_entities).push(link);

        Ok(())
    }

    /// Registers and entity that wants to affect this environment.
    pub fn register_affecting_entity(&mut self, mut entity: Entity) -> Result<(), Error> {
        entity.affect_environment(&self.name)?;

        unlock!(self.affecting_entities).push(entity);
        Ok(())
    }

    /// Inform joined entities that this environment is going to be dropped.
    pub fn send_term_sig(&self) -> Result<(), Error> {
        println!("Environment '{}' sending_term_sig", self.name);
        unlock!(self.drop_notifier).pull()
    }

    /// Returns a waker that allows to wake this environments task/future.
    pub fn get_waker(&self) -> Watcher {
        self.waker.clone()
    }
}

impl Future for Environment {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        self.waker.task.register();

        // As long as effects can be received go on broadcasting them
        {
            let mut out_chan = unlock!(self.out_chan);

            // TODO: maybe make this a for-loop with some predefined max number of effects to
            // not block other futures from making progress
            loop {
                // Try to receive a new effect from the supervisor
                match self.in_chan.try_recv() {
                    Ok(effect) => {
                        println!("Env. {} received effect '{}'", self.name, effect);
                        out_chan.broadcast(effect);
                    }
                    _ => break,
                }
            }
        }

        // Check for shutdown signal
        match unlock!(self.shutdown_listener).0.poll() {
            // sig-term received
            Ok(Async::Ready(Some(is_term))) => {
                if is_term {
                    println!("Env. {} received sig-term", self.name);
                    // End this future
                    return Ok(Async::Ready(()));
                }
            }
            _ => (),
        }

        // Wake all joined entities
        {
            let joined = unlock!(self.joined_entities);
            for EntityLink { entity: _, waker } in joined.iter() {
                waker.task.notify();
            }
        }

        // otherwise go to sleep
        return Ok(Async::NotReady);
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            joined_entities: Arc::clone(&self.joined_entities),
            affecting_entities: Arc::clone(&self.affecting_entities),
            in_chan: Arc::clone(&self.in_chan),
            out_chan: Arc::clone(&self.out_chan),
            drop_notifier: Arc::clone(&self.drop_notifier),
            shutdown_listener: Arc::clone(&self.shutdown_listener),
            waker: self.waker.clone(),
        }
    }
}
