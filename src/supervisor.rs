//! Supervisor module.

use std::collections::HashMap;

use crossbeam_channel::{
    unbounded,
    Sender,
};
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;

use crate::eee::entity::Entity;
use crate::eee::environment::Environment;
use crate::errors::Error;

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
}

/// Communication link between the supervisor and an environment.
pub struct EnvironmentLink {
    sender: Sender<String>,
    environment: Environment,
}

impl EnvironmentLink {
    /// Returns a mutable reference to the environment
    pub fn get_env_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }
}

impl Supervisor {
    /// Create a new environment.
    pub fn create_environment(&mut self, name: &str) -> Result<Environment, Error> {
        if self.environments.contains_key(name) {
            return Err(Error::App("Environment with that name already exists."));
        }

        // Create a communication channel between the supervisor and the new environment
        let (sender, receiver) = unbounded();

        // Create a new environment which gets the receiving end of the channel
        let env = Environment::new(name, receiver);

        // Create a link between the supervisor and the new environment through which the supervisor
        // will send messages to the environment.
        let link = EnvironmentLink { sender, environment: env.clone() };

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

        // Create a new entity which will subscribe to the specified environments
        let ent = Entity::default();

        // Let the entity join all specified environments
        for env_name in environments.iter() {
            let link = self.environments.get_mut(*env_name).unwrap();
            let env = link.get_env_mut();
            env.join_entity(ent.clone())?;
        }

        // Spawn the Entity future onto the Tokio runtime
        self.runtime.spawn(ent.clone().map_err(|_| ()));

        Ok(ent)
    }

    /// Submit a message to an enviroment.
    pub fn submit_message(&self, message: &str, env_name: &str) -> Result<(), Error> {
        match self.environments.get(env_name) {
            Some(link) => match link.sender.send(message.into()) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::App("Error seding the message to the environment")),
            },
            None => Err(Error::App("No environment with this name available")),
        }
    }

    ///
    pub fn wait(mut self) {
        self.runtime.run().unwrap();
    }

    /// Returns the number of supervised environments.
    pub fn num_environments(&self) -> usize {
        self.environments.len()
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self { runtime: Runtime::new().unwrap(), environments: HashMap::new() }
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
