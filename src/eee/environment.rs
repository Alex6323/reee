//! Environment module.

use super::effect::Effect;
use crate::common::trigger::{
    Trigger,
    TriggerHandle,
};
use crate::common::watcher::Watcher;
use crate::constants::BROADCAST_BUFFER_SIZE;
use crate::eee::entity::Entity;
use crate::errors::Error;

use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use std::sync::{
    Arc,
    Mutex,
};

use bus::Bus as Broadcaster;
use crossbeam_channel::Receiver;
use tokio::prelude::*;

/// An environment in the EEE model.
pub struct Environment {
    /// Name of the environment
    name: String,
    /// Entities that joined this environment
    joined_entities: Arc<Mutex<Vec<JoinedEntity>>>,
    /// Entities that affect this environment
    affecting_entities: Arc<Mutex<Vec<AffectingEntity>>>,
    /// Receiver half of the channel to the supervisor
    in_chan: Arc<Receiver<Effect>>,
    /// Sender half of the outgoing broadcast channel to send data to entities.
    out_chan: Arc<Mutex<Broadcaster<Effect>>>,
    /// A notifier that signals the end of this environment to subscribed
    /// entities
    drop_notifier: Arc<Mutex<Trigger>>,
    /// A listener for supervisor shutdown
    shutdown_listener: Arc<Mutex<TriggerHandle>>,
    /// A notifier that allows to wake this environments task/future
    waker: Watcher,
    /// The number of received effects.
    num_received_effects: Arc<AtomicUsize>,
}

/// Link between environment and an entity.
struct JoinedEntity {
    ///
    entity: Entity,
    /// A waker to wake up the entity's task/future
    pub waker: Watcher,
}

/// An abstraction
struct AffectingEntity {
    entity: Entity,
}

impl Environment {
    /// Creates a new environment.
    pub fn new(
        name: &str,
        in_chan: Receiver<Effect>,
        shutdown_listener: TriggerHandle,
    ) -> Self {
        let waker = Watcher::new();
        Self {
            name: name.into(),
            joined_entities: shared_mut!(vec![]),
            affecting_entities: shared_mut!(vec![]),
            in_chan: shared!(in_chan),
            out_chan: shared_mut!(Broadcaster::new(BROADCAST_BUFFER_SIZE)),
            drop_notifier: shared_mut!(Trigger::new()),
            shutdown_listener: shared_mut!(shutdown_listener),
            waker,
            num_received_effects: shared!(AtomicUsize::new(0)),
        }
    }

    /// Registers an entity that wants to join this evironment.
    pub fn register_joining_entity(
        &mut self,
        mut entity: Entity,
    ) -> Result<(), Error> {
        // Data required by the joining entity
        let out_chan = unlock!(self.out_chan).add_rx();
        let sig_term = unlock!(self.drop_notifier).get_handle();
        entity.join_environment(&self.name, out_chan, sig_term)?;

        // Data required by the joined environment
        let ent_waker = entity.get_waker();
        let joiner = JoinedEntity { entity, waker: ent_waker };

        unlock!(self.joined_entities).push(joiner);

        Ok(())
    }

    /// Registers and entity that wants to affect this environment.
    pub fn register_affecting_entity(
        &mut self,
        mut entity: Entity,
    ) -> Result<(), Error> {
        // Data required by the affecting entity
        let env_waker = self.waker.clone();
        entity.affect_environment(&self.name, env_waker)?;

        // Data requied by the affected environment
        let affector = AffectingEntity { entity: entity };
        unlock!(self.affecting_entities).push(affector);

        Ok(())
    }

    /// Inform joined entities that this environment is going to be dropped.
    pub fn send_term_sig(&self) -> Result<(), Error> {
        println!("Environment '{}' sending_term_sig", self.name);
        unlock!(self.drop_notifier).pull()
    }

    /// Returns the uuid of this entity.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the number of effects that this entity has received.
    pub fn num_received_effects(&self) -> usize {
        self.num_received_effects.load(Ordering::Relaxed)
        //*unlock!(self.num_received_effects)
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
            let joined = unlock!(self.joined_entities);
            let mut out_chan = unlock!(self.out_chan);

            // TODO: maybe make this a for-loop with some predefined max number
            // of effects to not block other futures from making
            // progress
            let mut num_received =
                self.num_received_effects.load(Ordering::Acquire);

            let mut num = 0;
            loop {
                // Try to receive a new effect from the supervisor
                match self.in_chan.try_recv() {
                    Ok(effect) => {
                        num += 1;

                        println!(
                            "Env. {} received effect '{}' ({})",
                            self.name,
                            effect,
                            num_received + num
                        );
                        out_chan.broadcast(effect);

                        // Wake all joined entities if half of the broadcaster
                        // buffer size if full
                        if num == BROADCAST_BUFFER_SIZE / 2 {
                            for JoinedEntity { entity: _, waker } in
                                joined.iter()
                            {
                                waker.task.notify();
                            }

                            num_received += num;
                            num = 0;
                        }
                    }
                    _ => break,
                }
            }
            self.num_received_effects
                .store(num_received + num, Ordering::Release);

            for JoinedEntity { entity: _, waker } in joined.iter() {
                waker.task.notify();
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

        // otherwise go to sleep
        return Ok(Async::NotReady);
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            joined_entities: Arc::clone(&self.joined_entities),
            affecting_entities: Arc::clone(&self.affecting_entities),
            in_chan: Arc::clone(&self.in_chan),
            out_chan: Arc::clone(&self.out_chan),
            drop_notifier: Arc::clone(&self.drop_notifier),
            shutdown_listener: Arc::clone(&self.shutdown_listener),
            waker: self.waker.clone(),
            num_received_effects: Arc::clone(&self.num_received_effects),
        }
    }
}
