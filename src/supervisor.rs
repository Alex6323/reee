//! Supervisor module.

use crate::common::shutdown::GracefulShutdown;
use crate::common::watcher::Watcher;
use crate::eee::effect::Effect;
use crate::eee::entity::EntityHost;
use crate::eee::environment::Environment;
use crate::errors::Error;

use std::collections::HashMap;

use crossbeam_channel::{
    unbounded,
    Sender,
};
use tokio::prelude::*;
use tokio::runtime::Runtime;

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
    // The supervisor runtime
    runtime: Runtime,
    // Environments managed by the supervisor
    environments: HashMap<String, EnvironmentConnection>,
    // Entities managed by the supervisor
    entities: HashMap<String, EntityConnection>,
    // Graceful shutdown of the supervisor and all started async tasks.
    graceful_shutdown: GracefulShutdown,
}

/// Connection between the supervisor and an environment.
pub(crate) struct EnvironmentConnection {
    /// sender half of the channel between supervisor and environment
    pub sender: Sender<Effect>,
    /// the environment that is linked to the supervisor
    pub environment: Environment,
    /// a notfier for waking up the environment task/future
    pub waker: Watcher,
}

/// Connection between the supervisor and an entity.
pub(crate) struct EntityConnection {
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
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            runtime: Runtime::new()?,
            environments: HashMap::new(),
            entities: HashMap::new(),
            graceful_shutdown: GracefulShutdown::new(),
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
    pub fn create_environment(&mut self, name: &str) -> Result<Environment, Error> {
        if self.environments.contains_key(name) {
            return Err(Error::App("Environment with that name already exists."));
        }

        // Create a communication channel between the supervisor and the new
        // environment Note: maybe use 'bounded' to allow for back
        // pressure
        let (sender, receiver) = unbounded();

        // Get a shutdown listener for notifiying the environment in case of
        // supervisor shutdown
        let shutdown_listener = self.graceful_shutdown.get_listener();

        // Create a new environment which gets the receiving end of the channel
        let env = Environment::new(name, receiver, shutdown_listener);

        // Create a link between the supervisor and the new environment through
        // which the supervisor will send messages to the environment.
        let conn = EnvironmentConnection {
            sender,
            environment: env.clone(),
            waker: env.get_waker(),
        };

        // Store the link
        self.environments.insert(name.into(), conn);

        // Spawn the Environment future onto the Tokio runtime
        self.runtime.spawn(env.clone().map_err(|_| ()));

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
    pub fn delete_environment(&mut self, env_name: &str) -> Result<(), Error> {
        match self.environments.remove(env_name) {
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
    pub fn create_entity(&mut self) -> Result<EntityHost, Error> {
        let entity = EntityHost::new(self.graceful_shutdown.get_listener());

        // Store the entity
        self.entities
            .insert(entity.uuid().into(), EntityConnection { entity: entity.clone() });

        // Spawn the Entity future onto the Tokio runtime
        self.runtime.spawn(entity.clone().map_err(|_| ()));

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
    pub fn delete_entity(&mut self, uuid: &str) -> Result<(), Error> {
        match self.entities.remove(uuid) {
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
    ) -> Result<(), Error> {
        // Check, if all given environments are known to this supervisor
        if !environments.iter().all(|env_name| self.environments.contains_key(*env_name))
        {
            return Err(Error::App(
                "At least one of the specified environments is unknown to this supervisor.",
            ));
        }

        // Let the entity join all specified environments
        for env_name in environments.iter() {
            let conn = self.environments.get_mut(*env_name).unwrap();
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
    ) -> Result<(), Error> {
        // Check, if all given environments are known to this supervisor
        if !environments.iter().all(|env_name| self.environments.contains_key(*env_name))
        {
            return Err(Error::App(
                "At least one of the specified environments is unknown to this supervisor.",
            ));
        }

        // Let the entity affect all specified environments
        for env_name in environments.iter() {
            let conn = self.environments.get_mut(*env_name).unwrap();
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
    pub fn submit_effect(&mut self, effect: Effect, env_name: &str) -> Result<(), Error> {
        match self.environments.get(env_name) {
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

    /// Shuts down the supervisor programmatically without user intervention.
    ///
    /// # Example
    /// ```
    /// use reee::supervisor::Supervisor;
    ///
    /// let sv = Supervisor::new().unwrap();
    ///
    /// sv.shutdown().unwrap();
    /// ```
    pub fn shutdown(mut self) -> Result<(), Error> {
        // Send the signal to make all infinite futures return
        // Ok(Async::Ready(None))
        self.graceful_shutdown.send_sig_term()?;

        println!("Shutting down...");

        self.runtime.shutdown_on_idle().wait().unwrap();

        Ok(())
    }

    /// Shuts down the supervisor on CTRL-C.
    pub fn wait_for_kill_signal(self) -> Result<(), Error> {
        println!("Waiting for Ctrl-C...",);

        self.graceful_shutdown.wait_for_ctrl_c();

        println!();

        self.shutdown()
    }

    /// Returns the number of supervised environments.
    pub fn num_environments(&self) -> usize {
        self.environments.len()
    }

    /// Returns the number of supervised entities.
    pub fn num_entities(&self) -> usize {
        self.entities.len()
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
