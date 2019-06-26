use ::reee::supervisor::Supervisor;

#[macro_use]
mod common;

use crate::common::*;

#[test]
fn pipe() {
    //
    let mut sv = Supervisor::new().unwrap();

    let x = sv.create_environment("X").unwrap();
    let y = sv.create_environment("Y").unwrap();
    let mut a = sv.create_entity().unwrap();
    sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    sv.affect_environments(&mut a, vec![&y.name()]).unwrap();

    sv.submit_effect("hello", &x.name()).unwrap();
}
