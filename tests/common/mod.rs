use ::reee::eee::entity::Entity;
use ::reee::eee::environment::Environment;
use ::reee::supervisor::Supervisor;

#[macro_use]
pub mod macros;

/// Creates a supervisor, and environment X, and an entity
pub fn get_supervisor_environment_entity() -> (Supervisor, Environment, Entity)
{
    let mut sv = Supervisor::new().expect("creating supervisor");
    let env = sv.create_environment("X").expect("creating environment X");
    let ent =
        sv.create_entity(vec!["X"]).expect("creating an entity joining X");
    (sv, env, ent)
}
