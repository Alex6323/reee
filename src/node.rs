//! A node featuring a Supervisor.

use crate::common::shutdown::GracefulShutdown;
use crate::eee::entity::EntityHost;
use crate::eee::environment::Environment;
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
            graceful_shutdown: GracefulShutdown::new(),
        })
    }

    /// Initializes the node.
    pub fn init(&mut self) {
        // Spawn the Supervisor onto the runtime

        // Spawn the Environment future onto the Tokio runtime
        //self.runtime.spawn(self.supervisor.map_err(|_| ()));
    }

    /// Creates an environment.
    pub fn create_environment(&mut self, name: &str) -> Result<Environment> {
        let sd_handle = self.graceful_shutdown.get_listener();

        //
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

    /// Shuts down the supervisor on CTRL-C.
    pub fn wait_for_kill_signal(self) -> Result<()> {
        println!("Waiting for Ctrl-C...",);

        self.graceful_shutdown.wait_for_ctrl_c();

        println!();

        self.shutdown()
    }
}
