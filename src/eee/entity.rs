//! Entity

use super::effect::Effect;
use crate::common::trigger::TriggerHandle;
use crate::common::watcher::Watcher;
use crate::constants::BROADCAST_BUFFER_SIZE;
use crate::errors::Error;

use std::collections::HashMap;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use std::sync::{
    Arc,
    Mutex,
};

use bus::Bus as Broadcaster;
use bus::BusReader as Receiver;
use tokio::{
    io,
    prelude::*,
};
use uuid::Uuid;

/// An entity in the EEE model.
pub struct Entity {
    /// A unique identifier of this entity.
    uuid: String,
    /// The environments this entity has joined.
    joined_environments: Arc<Mutex<HashMap<String, JoinedEnvironment>>>,
    /// The environments this entity affects.
    affected_environments: Arc<Mutex<HashMap<String, AffectedEnvironment>>>,
    /// Sender half of the outgoing broadcast channel for affecting
    /// environments
    out_chan: Arc<Mutex<Broadcaster<Effect>>>,
    /// A handle to signal supervisor shutdown
    shutdown_listener: Arc<Mutex<TriggerHandle>>,
    /// A waker to wake up this entitie's task/future
    waker: Watcher,
    /// The number of received effects.
    num_received_effects: Arc<AtomicUsize>,
}

/// Encapsulation of necessary data received from a joined environment.
struct JoinedEnvironment {
    /// effect receiving channel half
    pub in_chan: Receiver<Effect>,
    /// environment sig term listener
    pub term_sig: TriggerHandle,
}

/// Encapsulation of necessary data received from an affected environment.
struct AffectedEnvironment {
    /// a waker to wake the affected environment's task/future
    pub env_waker: Watcher,
}

impl Entity {
    /// Creates a new entity.
    pub fn new(shutdown_listener: TriggerHandle) -> Self {
        let waker = Watcher::new();
        Self {
            uuid: Uuid::new_v4().to_string(),
            joined_environments: shared_mut!(HashMap::new()),
            affected_environments: shared_mut!(HashMap::new()),
            out_chan: shared_mut!(Broadcaster::new(BROADCAST_BUFFER_SIZE)),
            shutdown_listener: shared_mut!(shutdown_listener),
            waker,
            num_received_effects: shared!(AtomicUsize::new(0)),
        }
    }

    /// Registers an environment as joined by this entity.
    pub fn join_environment(
        &mut self,
        name: &str,
        in_chan: Receiver<Effect>,
        term_sig: TriggerHandle,
    ) -> Result<(), Error> {
        //
        let mut joined = unlock!(self.joined_environments);

        if joined.contains_key(name) {
            return Err(Error::App("This entity already joined that environment"));
        }

        // Store the name and an environment listener
        joined.insert(name.into(), JoinedEnvironment { in_chan, term_sig });

        Ok(())
    }

    /// Registers an environment as affected by this entity.
    pub fn affect_environment(
        &mut self,
        env_name: &str,
        env_waker: Watcher,
    ) -> Result<(), Error> {
        let mut affected = unlock!(self.affected_environments);

        if affected.contains_key(env_name) {
            return Err(Error::App("This entity already affects that environment"));
        }

        // Store the name and the receiver handle of that environment
        affected.insert(env_name.into(), AffectedEnvironment { env_waker });

        Ok(())
    }

    /// Returns the uuid of this entity.
    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    /// Returns the number of effects that this entity has received.
    pub fn num_received_effects(&self) -> usize {
        self.num_received_effects.load(Ordering::Relaxed)
    }

    /// Returns a waker that allows to wake this entity's task/future.
    pub fn get_waker(&self) -> Watcher {
        self.waker.clone()
    }

    /// Returns a list of all environments this entity has joined.
    pub fn joined_environments(&self) -> Vec<String> {
        unlock!(self.joined_environments)
            .keys()
            .map(|key| key.to_string())
            .collect::<Vec<String>>()
    }

    /// Returns a list of all environments this entity is affecting.
    pub fn affected_environments(&self) -> Vec<String> {
        unlock!(self.affected_environments)
            .keys()
            .map(|key| key.to_string())
            .collect::<Vec<String>>()
    }

    /// Returns true, if this entity has joined the specified environment,
    /// otherwise false.
    pub fn has_joined(&self, env_name: &str) -> bool {
        unlock!(self.joined_environments).contains_key(env_name)
    }

    /// Returns true, if this entity has joined the specified environment,
    /// otherwise false.
    pub fn is_affecting(&self, env_name: &str) -> bool {
        unlock!(self.affected_environments).contains_key(env_name)
    }

    /// Returns the number of joined environments.
    pub fn num_joined(&self) -> usize {
        unlock!(self.joined_environments).len()
    }

    /// Returns the number of affected environments.
    pub fn num_affected(&self) -> usize {
        unlock!(self.affected_environments).len()
    }
}

impl Future for Entity {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        self.waker.task.register();

        // this scope will modify 'joined_environments'
        {
            let num_effects = self.num_received_effects.load(Ordering::Acquire);
            let mut num = 0;

            let mut joined = unlock!(self.joined_environments);
            let mut to_drop = vec![];

            'outer: loop {
                // number of dry in-channels
                let mut num_dry = 0;

                // Check each joined environment if there is a new effect
                for (env, JoinedEnvironment { in_chan, term_sig: _ }) in joined.iter_mut()
                {
                    // Try to receive as many effects as possible from that
                    // environment TODO: maybe make this a
                    // for-loop with an upper limit to give other
                    // futures time to progress as well
                    'inner: loop {
                        match in_chan.try_recv() {
                            Ok(effect) => {
                                num += 1;

                                println!(
                                    "Ent. {} received effect '{}' from environment '{}' ({})",
                                    &self.uuid[0..5],
                                    effect,
                                    env,
                                    num_effects + num,
                                )
                            }
                            _ => {
                                num_dry += 1;
                                break 'inner;
                            }
                        }
                    }
                }

                // If all channels are dry this future can finally go to sleep
                // until awakened again
                if num_dry >= joined.len() {
                    break 'outer;
                }
            }

            self.num_received_effects.store(num_effects + num, Ordering::Release);

            // Check if any environment sent a sig-term
            for (env, JoinedEnvironment { in_chan: _, term_sig }) in joined.iter_mut() {
                match term_sig.0.poll() {
                    Ok(Async::Ready(Some(is_term))) => {
                        if is_term {
                            println!(
                                "Ent. {} received sig-term from environment '{}'",
                                &self.uuid[0..5],
                                env
                            );

                            // Remember to unsubscribe from that environment
                            to_drop.push(env.clone());
                        }
                    }
                    _ => (),
                }
            }

            // Remove all environments we received a term signal from
            for env in to_drop {
                joined.remove(&env);
                println!(
                    "Ent. {} unsubscribed from environment '{}'",
                    &self.uuid[0..5],
                    env
                );
            }
        } // we're finished with mutating 'joined_environments'

        // Check if the supervisor is about to shutdown
        match unlock!(self.shutdown_listener).0.poll() {
            // sig-term received
            // NOTE: the 'watch' channel always yields Some!!
            Ok(Async::Ready(Some(is_term))) => {
                if is_term {
                    println!("Ent. {} received sig-term", &self.uuid[0..5]);
                    // End this future
                    return Ok(Async::Ready(()));
                }
            }
            _ => (),
        }

        // Entity goes to sleep
        Ok(Async::NotReady)
    }
}

impl Clone for Entity {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid.clone(),
            joined_environments: Arc::clone(&self.joined_environments),
            affected_environments: Arc::clone(&self.affected_environments),
            out_chan: Arc::clone(&self.out_chan),
            shutdown_listener: Arc::clone(&self.shutdown_listener),
            waker: self.waker.clone(),
            num_received_effects: Arc::clone(&self.num_received_effects),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::trigger::Trigger;

    #[test]
    fn each_entity_has_uuid() {
        let shutdown_listener = Trigger::new().get_handle();

        let entity = Entity::new(shutdown_listener);

        assert!(!entity.uuid().is_empty())
    }
}
