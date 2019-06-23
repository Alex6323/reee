//! Supervisor module.

use crate::common::shutdown::GracefulShutdown;
use crate::common::watcher::Watcher;
use crate::eee::entity::Entity;
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
/// let mut sv = Supervisor::default();
///
/// let _x = sv.create_environment("X").expect("error creating environment");
/// let _y = sv.create_environment("Y").expect("error creating environment");
///
/// let _a = sv.create_entity(vec!["X"]).expect("error creating entity");
/// let _b = sv.create_entity(vec!["X", "Y"]).expect("error creating entity");
///
/// sv.submit_message("hello", "X").expect("error sending message");
/// sv.submit_message("world", "Y").expect("error sending message");
/// ```
pub struct Supervisor {
    runtime: Runtime,
    environments: HashMap<String, EnvironmentLink>,
    graceful_shutdown: GracefulShutdown,
}

/// Link between the supervisor and an environment.
pub struct EnvironmentLink {
    /// sender half of the channel between supervisor and environment
    sender: Sender<String>,
    /// the environment that is linked to the supervisor
    environment: Environment,
    /// a notfier for waking up the environment task/future
    pub waker: Watcher,
}

impl EnvironmentLink {
    /// Returns a mutable reference to the environment
    pub fn get_env_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    /// Returns a reference to the environment
    pub fn get_env(&self) -> &Environment {
        &self.environment
    }
}

impl Supervisor {
    /// Creates a new supervisor.
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            environments: HashMap::new(),
            graceful_shutdown: GracefulShutdown::new(),
        }
    }

    /// Create a new environment.
    pub fn create_environment(&mut self, name: &str) -> Result<Environment, Error> {
        if self.environments.contains_key(name) {
            return Err(Error::App("Environment with that name already exists."));
        }

        // Create a communication channel between the supervisor and the new environment
        // Note: maybe use 'bounded' to allow for back pressure
        let (sender, receiver) = unbounded();

        // Get a shutdown listener for notifiying the environment in case of supervisor shutdown
        let shutdown_listener = self.graceful_shutdown.get_listener();

        // Create a new environment which gets the receiving end of the channel
        let env = Environment::new(name, receiver, shutdown_listener);

        // Create a link between the supervisor and the new environment through which the supervisor
        // will send messages to the environment.
        let link = EnvironmentLink { sender, environment: env.clone(), waker: env.get_waker() };

        // Store the link
        self.environments.insert(name.into(), link);

        // Spawn the Environment future onto the Tokio runtime
        self.runtime.spawn(env.clone().map_err(|_| ()));

        Ok(env)
    }

    /// Create an entity and attach it to one or multiple environments.
    pub fn create_entity(&mut self, environments: Vec<&str>) -> Result<Entity, Error> {
        // Check, if all given environments are known to this supervisor
        if !environments.iter().all(|env_name| self.environments.contains_key(*env_name)) {
            return Err(Error::App(
                "At least one of the specified environments is unknown to this supervisor.",
            ));
        }

        // Get a shutdown listener for notifiying the environment in case of supervisor shutdown
        let shutdown_listener = self.graceful_shutdown.get_listener();

        // Create a new entity which will subscribe to the specified environments
        let ent = Entity::new(shutdown_listener);

        // Let the entity join all specified environments
        for env_name in environments.iter() {
            let link = self.environments.get_mut(*env_name).unwrap();
            let env = link.get_env_mut();
            env.register_joining_entity(ent.clone())?;
        }

        // Spawn the Entity future onto the Tokio runtime
        self.runtime.spawn(ent.clone().map_err(|_| ()));

        Ok(ent)
    }

    /// Submit an effect to an enviroment.
    pub fn submit_effect(&mut self, effect: &str, env_name: &str) -> Result<(), Error> {
        match self.environments.get(env_name) {
            Some(env_link) => {
                match env_link.sender.send(effect.into()) {
                    Err(_) => {
                        return Err(Error::App("Error sending the message to the environment"))
                    }
                    _ => (),
                }
                // Notify the task associated with this environment to wake up and do some work
                env_link.waker.task.notify();
            }
            None => return Err(Error::App("No environment with this name available")),
        }

        Ok(())
    }

    /// Delete an environment.
    pub fn delete_environment(&mut self, env_name: &str) -> Result<(), Error> {
        match self.environments.remove(env_name) {
            Some(link) => {
                link.environment.send_term_sig()?;
                Ok(())
            }
            None => Err(Error::App(
                "There is no environment with that name managed by this supervisor.",
            )),
        }
    }

    /// Shuts down the supervisor.
    pub fn shutdown(mut self) -> Result<(), Error> {
        // Send the signal to make all infinite futures return Ok(Async::Ready(None))
        self.graceful_shutdown.send_term_sig()?;

        println!("Shutting down...");

        self.runtime.shutdown_on_idle().wait().unwrap();

        Ok(())
    }

    /// Shuts down the supervisor on CTRL-C.
    pub fn wait_for_kill_signal(self) -> Result<(), Error> {
        // Wait for CTRL-C
        println!("Waiting for user interaction...",);
        self.graceful_shutdown.wait_for_ctrl_c();

        self.shutdown()
    }

    /// This method can be used in unit/integration tests to shutdown the supervisor
    /// programmatically.
    #[cfg(test)]
    pub fn shutdown(mut self) {
        // Send signal to stop infinite futures
        drop(self.grace_full_shutdown);

        // Wait until all futures return Ok(Async::Ready(None)), then shutdown the runtime
        self.runtime.shutdown_on_idle().wait().unwrap();
    }

    /// Returns the number of supervised environments.
    pub fn num_environments(&self) -> usize {
        self.environments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_supervisor() {
        let _sv = Supervisor::default();
    }

    #[test]
    fn two_different_environments() {
        let mut sv = Supervisor::default();
        sv.create_environment("a").unwrap();
        sv.create_environment("b").unwrap();
    }

    // Cannot create the same environment twice
    #[test]
    #[should_panic]
    fn create_environment() {
        let mut sv = Supervisor::default();
        sv.create_environment("a").unwrap();
        sv.create_environment("a").unwrap();
    }
}
