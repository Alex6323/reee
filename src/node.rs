//! A node featuring a Supervisor.

use crate::common::shutdown::GracefulShutdown;
use crate::eee::Effect;
use crate::eee::EntityHost;
use crate::eee::Environment;
use crate::errors::Result;
use crate::supervisor::Supervisor;

use tokio::prelude::*;
use tokio::runtime::Runtime;

/// A node featuring a Supervisor
pub struct Node {
    /// The Tokio runtime for this node.
    runtime: Runtime,

    /// The supervisor used for messaging.
    supervisor: Supervisor,

    /// Graceful shutdown of the supervisor and all started async tasks.
    graceful_shutdown: GracefulShutdown,
}

impl Node {
    /// Creates a new [`Node`].
    pub fn new() -> Result<Self> {
        let graceful_shutdown = GracefulShutdown::new();
        let sd_handle = graceful_shutdown.get_listener();

        Ok(Self {
            runtime: Runtime::new()?,
            supervisor: Supervisor::new(sd_handle)?,
            graceful_shutdown,
        })
    }

    /// Initializes the node.
    pub fn init(&mut self) {
        // Spawn the Supervisor onto the runtime
        self.runtime.spawn(self.supervisor.clone().map_err(|_| ()));
    }

    /// Shuts down the node on CTRL-C.
    pub fn run(self) -> Result<()> {
        println!("Waiting for Ctrl-C...",);

        self.graceful_shutdown.wait_for_ctrl_c();

        println!();

        self.shutdown()
    }

    /// Creates an environment.
    pub fn create_environment(&mut self, name: &str) -> Result<Environment> {
        let sd_handle = self.graceful_shutdown.get_listener();
        let env = self.supervisor.create_environment(name, sd_handle)?;

        // Spawn the Environment future onto the Tokio runtime
        self.runtime.spawn(env.clone().map_err(|_| ()));

        Ok(env)
    }

    /// Creates an entity.
    pub fn create_entity(&mut self) -> Result<EntityHost> {
        let sd_handle = self.graceful_shutdown.get_listener();
        let ent = self.supervisor.create_entity(sd_handle)?;

        // Spawn the Entity future onto the Tokio runtime
        self.runtime.spawn(ent.clone().map_err(|_| ()));

        Ok(ent)
    }

    /// Shuts down then node.
    pub fn shutdown(mut self) -> Result<()> {
        // Send the signal to make all infinite futures return
        // Ok(Async::Ready(None))
        self.graceful_shutdown.send_sig_term()?;

        println!("Shutting down...");

        self.runtime.shutdown_on_idle().wait().unwrap();

        Ok(())
    }

    /// Let an entity join a single or multiple environments.
    pub fn join_environments(
        &mut self,
        entity: &mut EntityHost,
        environments: Vec<&str>,
    ) -> Result<()> {
        self.supervisor.join_environments(entity, environments)
    }

    /// Let an entity affect a single or multiple environments.
    pub fn affect_environments(
        &mut self,
        entity: &mut EntityHost,
        environments: Vec<&str>,
    ) -> Result<()> {
        self.supervisor.affect_environments(entity, environments)
    }

    /// Submit an effect
    pub fn submit_effect(&mut self, effect: Effect, env_name: &str) -> Result<()> {
        self.supervisor.submit_effect(effect, env_name)
    }
}
