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
/// let mut sv = Supervisor::new().expect("creating supervisor");
///
/// let _x = sv.create_environment("X").expect("error creating environment");
/// let _y = sv.create_environment("Y").expect("error creating environment");
///
/// let _a = sv.create_entity(vec!["X"]).expect("error creating entity");
/// let _b = sv.create_entity(vec!["X", "Y"]).expect("error creating entity");
///
/// sv.submit_effect("hello", "X").expect("error sending message");
/// sv.submit_effect("world", "Y").expect("error sending message");
/// ```
pub struct Supervisor {
    // The supervisor runtime
    runtime: Runtime,
    // Environments managed by the supervisor
    environments: HashMap<String, EnvironmentLink>,
    // Entities managed by the supervisor
    entities: HashMap<String, Entity>,
    // Graceful shutdown of the supervisor and all started async tasks.
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
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            runtime: Runtime::new()?,
            environments: HashMap::new(),
            entities: HashMap::new(),
            graceful_shutdown: GracefulShutdown::new(),
        })
    }

    /// Create a new environment.
    pub fn create_environment(
        &mut self,
        name: &str,
    ) -> Result<Environment, Error> {
        if self.environments.contains_key(name) {
            return Err(Error::App(
                "Environment with that name already exists.",
            ));
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
        let link = EnvironmentLink {
            sender,
            environment: env.clone(),
            waker: env.get_waker(),
        };

        // Store the link
        self.environments.insert(name.into(), link);

        // Spawn the Environment future onto the Tokio runtime
        self.runtime.spawn(env.clone().map_err(|_| ()));

        Ok(env)
    }

    /// Delete an environment.
    pub fn delete_environment(&mut self, env_name: &str) -> Result<(), Error> {
        match self.environments.remove(env_name) {
            Some(link) => {
                // Inform subscribed entities that this environment is going to be dropped
                link.environment.send_term_sig()?;
                Ok(())
            }
            None => Err(Error::App(
                "There is no environment with that name managed by this supervisor.",
            )),
        }
    }

    /// Create an entity and attach it to one or multiple environments.
    pub fn create_entity(
        &mut self,
        environments: Vec<&str>,
    ) -> Result<Entity, Error> {
        // Check, if all given environments are known to this supervisor
        if !environments
            .iter()
            .all(|env_name| self.environments.contains_key(*env_name))
        {
            return Err(Error::App(
                "At least one of the specified environments is unknown to this supervisor.",
            ));
        }

        // Get a shutdown listener for notifiying the environment in case of
        // supervisor shutdown
        let shutdown_listener = self.graceful_shutdown.get_listener();

        // Create a new entity which will subscribe to the specified
        // environments
        let ent = Entity::new(shutdown_listener);

        // Let the entity join all specified environments
        for env_name in environments.iter() {
            let link = self.environments.get_mut(*env_name).unwrap();
            let env = link.get_env_mut();
            env.register_joining_entity(ent.clone())?;
        }

        // Store the entity
        self.entities.insert(ent.uuid(), ent.clone());

        // Spawn the Entity future onto the Tokio runtime
        self.runtime.spawn(ent.clone().map_err(|_| ()));

        Ok(ent)
    }

    /// Delete an entity.
    pub fn delete_entity(&mut self, uuid: &str) -> Result<Entity, Error> {
        match self.entities.remove(uuid) {
            Some(ent) => {
                // Unsubscribe from all environments the entity has joined and
                // affected ent.send_sigterm
                //Ok(())
                unimplemented!()
            }
            None => Err(Error::App(
                "There is no entity with that uuid managed by this supervisor.",
            )),
        }
    }

    /// Submit an effect to an enviroment.
    pub fn submit_effect(
        &mut self,
        effect: &str,
        env_name: &str,
    ) -> Result<(), Error> {
        match self.environments.get(env_name) {
            Some(env_link) => {
                match env_link.sender.send(effect.into()) {
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
            None => {
                return Err(Error::App(
                    "No environment with this name available",
                ))
            }
        }

        Ok(())
    }

    /// Shuts down the supervisor programmatically without user intervention.
    pub fn shutdown(mut self) -> Result<(), Error> {
        // Send the signal to make all infinite futures return
        // Ok(Async::Ready(None))
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
    fn create_and_shutdown() {
        let sv = Supervisor::new().expect("creating supervisor");
        sv.shutdown().expect("shutting down");
    }

    #[test]
    fn create_non_joining_entity() {
        let mut sv = Supervisor::new().expect("creating supervisor");
        sv.create_entity(vec![]).expect("creating entity");
        sv.shutdown().expect("shutting down");
    }

    #[test]
    fn create_two_different_environments() {
        let mut sv = Supervisor::new().expect("creating supervisor");

        sv.create_environment("X").expect("creating X");
        sv.create_environment("Y").expect("creating Y");

        assert_eq!(2, sv.num_environments());

        sv.shutdown().expect("shutting down");
    }

    // Cannot create the same environment twice
    #[should_panic]
    #[test]
    fn forbid_creating_the_same_environment_twice() {
        let mut sv = Supervisor::new().expect("creating supervisor");

        sv.create_environment("X").expect("creating X");
        sv.create_environment("X").expect("creating X");

        sv.shutdown().expect("shutting down");
    }

    #[test]
    fn create_and_delete_environment() {
        let mut sv = Supervisor::new().expect("creating supervisor");

        sv.create_environment("X").expect("creating X");
        assert_eq!(1, sv.num_environments());

        sv.delete_environment("X").expect("deleting X");
        assert_eq!(0, sv.num_environments());

        sv.shutdown().expect("shutting down");
    }

    #[test]
    fn submit_two_effects() {
        let mut sv = Supervisor::new().expect("creating supervisor");
        let env = sv.create_environment("X").expect("creating X");
        let ent = sv.create_entity(vec!["X"]).expect("creating entity");

        sv.submit_effect("hello", "X").expect("submitting effect 1");
        sv.submit_effect("world", "X").expect("submitting effect 2");

        // Wait a little until the effects have propagated
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(2, env.num_received_effects());
        assert_eq!(2, ent.num_received_effects());

        sv.shutdown().expect("shutting down");
    }

    #[test]
    fn submit_many_effects_to_two_entities() {
        let mut sv = Supervisor::new().expect("creating supervisor");
        let env = sv.create_environment("X").expect("creating X");
        let ent_1 = sv.create_entity(vec!["X"]).expect("creating entity");
        let ent_2 = sv.create_entity(vec!["X"]).expect("creating entity");

        for i in 0..729 {
            sv.submit_effect(&i.to_string(), "X").expect("submitting effect");
        }

        // Wait a little until the effects have propagated
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(729, env.num_received_effects());
        assert_eq!(729, ent_1.num_received_effects());
        assert_eq!(729, ent_2.num_received_effects());

        sv.shutdown().expect("shutting down");
    }
}
