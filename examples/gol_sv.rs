//! Game-of-Life EEE implementation

use common::gol::*;

use ::reee::eee::effect::Effect;
use ::reee::eee::entity::EntityCore;
use ::reee::supervisor::Supervisor;

use std::time::Instant;

fn main() {
    println!("Running Game-Of-Life EEE implementation...");

    let mut sv = Supervisor::new().expect("couldn't create supervisor");

    let x = sv.create_environment("cur_gen").expect("error creating 'cur_gen' env.");
    let y = sv.create_environment("new_gen").expect("error creating 'new_gen' env.");

    let mut a = sv.create_entity().expect("error creating entity");

    sv.join_environments(&mut a, vec![&x.name()]).expect("error joining 'cur_gen' env.");
    sv.affect_environments(&mut a, vec![&y.name()])
        .expect("error affecting 'new_gen' env");
}
