//! Supervisor module.

use crate::common::trigger::TriggerHandle;
use crate::common::watcher::Watcher;
use crate::eee::effect::Effect;
use crate::eee::entity::EntityHost;
use crate::eee::environment::Environment;
use crate::errors::{Error, Result};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Sender};
use tokio::prelude::*;

/// Registry for Environments.
///
/// # Example
/// ```
/// use reee::supervisor::Supervisor;
///
/// // Create a supervisor
/// let mut sv = Supervisor::new().unwrap();
///
/// // Create two environments X, Y
/// let x = sv.create_environment("X").unwrap();
/// let y = sv.create_environment("Y").unwrap();
///
/// // Create two entities
/// let mut a = sv.create_entity().unwrap();
/// let mut b = sv.create_entity().unwrap();
///
/// // Let them join environments
/// sv.join_environments(&mut a, vec![&x.name()]).unwrap();
/// sv.join_environments(&mut b, vec![&x.name(), &y.name()]).unwrap();
///
/// // Submit two effects to each environment
/// sv.submit_effect("hello", "X").unwrap();
/// sv.submit_effect("world", "Y").unwrap();
///
/// // Wait a little for effects to propagate
/// std::thread::sleep(std::time::Duration::from_millis(500));
///
/// assert_eq!(1, x.num_received_effects());
/// assert_eq!(1, y.num_received_effects());
/// assert_eq!(1, a.num_received_effects());
/// assert_eq!(2, b.num_received_effects());
/// ```
pub struct Supervisor {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    /// Environments managed by the supervisor
    environments: HashMap<String, EnvironmentConnection>,

    /// Entities managed by the supervisor
    entities: HashMap<String, EntityConnection>,

    /// A listener for supervisor shutdown
    shutdown_listener: TriggerHandle,
    /* A notfier for waking up the supervisor's task/future
     *waker: Watcher, */
}

impl Clone for Supervisor {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Connection between the supervisor and an environment.
pub(crate) struct EnvironmentConnection {
    /// Sender half of the channel between supervisor and environment
    pub sender: Sender<Effect>,

    /// The environment that is linked to the supervisor
    pub environment: Environment,

    /// A notfier for waking up the environment task/future
    pub waker: Watcher,
}

/// Connection between the supervisor and an entity.
pub(crate) struct EntityConnection {
    /// An entity.
    pub entity: EntityHost,
}

impl Supervisor {
    /// Creates a new supervisor.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let sv = Supervisor::new().unwrap();
    /// ```
    pub fn new(shutdown_listener: TriggerHandle) -> Result<Self> {
        let inner = Arc::new(Mutex::new(Inner {
            environments: HashMap::new(),
            entities: HashMap::new(),
            shutdown_listener,
        }));

        Ok(Self {
            inner,
        })
    }

    /// Creates a new environment.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let mut sv = Supervisor::new().unwrap();
    ///
    /// sv.create_environment("X").unwrap();
    /// ```
    pub fn create_environment(
        &mut self,
        name: &str,
        sd_handle: TriggerHandle,
    ) -> Result<Environment> {
        let mut inner = unlock!(self.inner);

        if inner.environments.contains_key(name) {
            return Err(Error::App("Environment with that name already exists."));
        }

        // Create a communication channel between the supervisor and the new
        // environment.
        let (sender, receiver) = unbounded();

        // Create a new environment which gets the receiving end of the channel
        let env = Environment::new(name, receiver, sd_handle);

        // Create a link between the supervisor and the new environment through
        // which the supervisor will send messages to the environment.
        let conn = EnvironmentConnection {
            sender,
            environment: env.clone(),
            waker: env.get_waker(),
        };

        // Store the link
        inner.environments.insert(name.into(), conn);

        Ok(env)
    }

    /// Delete an environment.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let mut sv = Supervisor::new().unwrap();
    ///
    /// let x = sv.create_environment("X").unwrap();
    ///
    /// sv.delete_environment(&x.name()).unwrap();
    /// ```
    pub fn delete_environment(&mut self, env_name: &str) -> Result<()> {
        let mut inner = unlock!(self.inner);
        match inner.environments.remove(env_name) {
            Some(env_conn) => {
                // Inform subscribed entities that this environment is going to be dropped
                env_conn.environment.send_sig_term()?;
                Ok(())
            }
            None => Err(Error::App(
                "There is no environment with that name managed by this supervisor.",
            )),
        }
    }

    /// Create an entity.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let mut sv = Supervisor::new().unwrap();
    ///
    /// sv.create_entity().unwrap();
    /// ```
    pub fn create_entity(&mut self, sd_handle: TriggerHandle) -> Result<EntityHost> {
        let mut inner = unlock!(self.inner);
        let entity = EntityHost::new(sd_handle);

        // Store the entity
        inner.entities
            .insert(entity.uuid().into(), EntityConnection { entity: entity.clone() });

        Ok(entity)
    }

    /// Delete an entity.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let mut sv = Supervisor::new().unwrap();
    /// let mut a = sv.create_entity().unwrap();
    ///
    /// sv.delete_entity(a.uuid()).unwrap();
    /// ```
    pub fn delete_entity(&mut self, uuid: &str) -> Result<()> {
        let mut inner = unlock!(self.inner);
        match inner.entities.remove(uuid) {
            Some(ent_conn) => {
                // Unsubscribe from all environments the entity has joined and
                ent_conn.entity.send_sig_term()?;
                Ok(())
            }
            None => Err(Error::App(
                "There is no entity with that uuid managed by this supervisor.",
            )),
        }
    }

    /// Lets the specified entity join one or multiple environments.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let mut sv = Supervisor::new().unwrap();
    /// let x = sv.create_environment("X").unwrap();
    /// let mut a = sv.create_entity().unwrap();
    ///
    /// sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    /// ```
    pub fn join_environments(
        &mut self,
        mut entity: &mut EntityHost,
        environments: Vec<&str>,
    ) -> Result<()> {
        let mut inner = unlock!(self.inner);
        // Check, if all given environments are known to this supervisor
        if !environments.iter().all(|env_name| inner.environments.contains_key(*env_name))
        {
            return Err(Error::App(
                "At least one of the specified environments is unknown to this supervisor.",
            ));
        }

        // Let the entity join all specified environments
        for env_name in environments.iter() {
            let conn = inner.environments.get_mut(*env_name).unwrap();
            conn.environment.register_joining_entity(&mut entity)?;
        }

        Ok(())
    }

    /// Lets the specified entity leave one or multiple environments.
    pub fn leave_environments(
        &mut self,
        mut _host: &mut EntityHost,
        _environments: Vec<&str>,
    ) {
        //
    }

    /// Lets the specified entity affect one or multiple environments.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let mut sv = Supervisor::new().unwrap();
    /// let x = sv.create_environment("X").unwrap();
    /// let mut a = sv.create_entity().unwrap();
    ///
    /// sv.affect_environments(&mut a, vec![&x.name()]).unwrap();
    /// ```
    pub fn affect_environments(
        &mut self,
        entity: &mut EntityHost,
        environments: Vec<&str>,
    ) -> Result<()> {
        let mut inner = unlock!(self.inner);
        // Check, if all given environments are known to this supervisor
        if !environments.iter().all(|env_name| inner.environments.contains_key(*env_name))
        {
            return Err(Error::App(
                "At least one of the specified environments is unknown to this supervisor.",
            ));
        }

        // Let the entity affect all specified environments
        for env_name in environments.iter() {
            let conn = inner.environments.get_mut(*env_name).unwrap();
            conn.environment.register_affecting_entity(entity)?;
        }

        Ok(())
    }

    /*
    pub fn stop_affecting_environments(
        &mut self,
        entity: &mut Entity,
        environments: Vec<&str>,
    ) -> Result<(), Error> {
        //
    }
    */

    /// Submit an effect to an enviroment.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let mut sv = Supervisor::new().unwrap();
    /// let x = sv.create_environment("X").unwrap();
    ///
    /// sv.submit_effect("hello", &x.name()).unwrap();
    /// ```
    pub fn submit_effect(&mut self, effect: Effect, env_name: &str) -> Result<()> {
        let inner = unlock!(self.inner);
        match inner.environments.get(env_name) {
            Some(env_link) => {
                match env_link.sender.send(effect) {
                    Err(_) => {
                        return Err(Error::App(
                            "Error sending the message to the environment",
                        ))
                    }
                    _ => (),
                }
                // Notify the task associated with this environment to wake up
                // and do some work
                env_link.waker.task.notify();
            }
            None => return Err(Error::App("No environment with this name available")),
        }

        Ok(())
    }

    /// Returns the number of supervised environments.
    pub fn num_environments(&self) -> usize {
        let inner = unlock!(self.inner);
        inner.environments.len()
    }

    /// Returns the number of supervised entities.
    pub fn num_entities(&self) -> usize {
        let inner = unlock!(self.inner);
        inner.entities.len()
    }
}

impl Future for Supervisor {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        //self.waker.task.register();
        let mut inner = unlock!(self.inner);

        // Check for shutdown signal
        match inner.shutdown_listener.0.poll() {
            // sig-term received
            Ok(Async::Ready(Some(is_term))) => {
                if is_term {
                    println!("Supervisor received sig-term");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_two_different_environments() {
        let mut sv = Supervisor::new().unwrap();

        sv.create_environment("X").unwrap();
        sv.create_environment("Y").unwrap();

        assert_eq!(2, sv.num_environments());
    }

    // Cannot create the same environment twice
    #[should_panic]
    #[test]
    fn forbid_creating_the_same_environment_twice() {
        let mut sv = Supervisor::new().unwrap();

        sv.create_environment("X").unwrap();
        sv.create_environment("X").unwrap();
    }

    #[test]
    fn create_and_delete_environment() {
        let mut sv = Supervisor::new().unwrap();

        let x = sv.create_environment("X").unwrap();
        assert_eq!(1, sv.num_environments());

        sv.delete_environment(&x.name()).unwrap();
        assert_eq!(0, sv.num_environments());
    }

    #[test]
    fn submit_two_effects() {
        let mut sv = Supervisor::new().unwrap();

        let x = sv.create_environment("X").unwrap();
        let mut a = sv.create_entity().unwrap();

        sv.join_environments(&mut a, vec![&x.name()]).unwrap();

        sv.submit_effect("hello", &x.name()).unwrap();
        sv.submit_effect("world", &x.name()).unwrap();

        // Wait a little until the effects have propagated
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(2, x.num_received_effects());
        assert_eq!(2, a.num_received_effects());
    }

    #[test]
    fn submit_many_effects_to_two_entities() {
        let mut sv = Supervisor::new().unwrap();

        let x = sv.create_environment("X").unwrap();

        let mut a = sv.create_entity().unwrap();
        let mut b = sv.create_entity().unwrap();

        sv.join_environments(&mut a, vec![&x.name()]).unwrap();
        sv.join_environments(&mut b, vec![&x.name()]).unwrap();

        for i in 0..729 {
            sv.submit_effect(&i.to_string(), &x.name()).unwrap();
        }

        // Wait a little until the effects have propagated
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(729, x.num_received_effects());
        assert_eq!(729, a.num_received_effects());
        assert_eq!(729, b.num_received_effects());
    }
}
