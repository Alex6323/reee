use ::reee::eee::entity::Entity;
use ::reee::eee::environment::Environment;
use ::reee::supervisor::Supervisor;

#[macro_use]
pub mod macros;

/// Creates a supervisor, and environment X, and an entity
pub fn get_supervisor_environment_entity() -> (Supervisor, Environment, Entity) {
    let mut sv = Supervisor::new().unwrap();
    let x = sv.create_environment("X").unwrap();
    let mut a = sv.create_entity().unwrap();
    sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    (sv, x, a)
}
