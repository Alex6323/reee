#![allow(dead_code)]

use ::reee::eee::effect::Effect;
use ::reee::eee::entity::EntityCore;
use ::reee::supervisor::Supervisor;
use std::thread;
use std::time::Duration;

fn main() {
    test6();
}

// Simplest setup
fn test1() {
    let mut sv = Supervisor::new().unwrap();

    let x = sv.create_environment("X").unwrap();
    println!(">>> Created environment X");

    thread::sleep(Duration::from_millis(500));

    let mut a = sv.create_entity().unwrap();
    println!(">>> Created entity {}", &a.uuid()[0..5]);

    sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    println!(">>> Entity {} joined {}", &a.uuid()[0..5], x.name());

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effect 'hello' to {}", x.name());
    sv.submit_effect(Effect::Ascii("hello".into()), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    sv.wait_for_kill_signal().unwrap();
}

fn test2() {
    let mut sv = Supervisor::new().unwrap();

    let x = sv.create_environment("X").unwrap();
    let y = sv.create_environment("Y").unwrap();
    println!(">>> Created environments {}, {}", x.name(), y.name());

    thread::sleep(Duration::from_millis(500));

    let mut a = sv.create_entity().unwrap();
    let mut b = sv.create_entity().unwrap();
    println!(">>> Created entities {}, {}", &a.uuid()[0..5], &b.uuid()[0..5]);

    sv.join_environments(&mut a, vec![&x.name(), &y.name()]).unwrap();
    println!(">>> Entity {} joined {}, {}", &a.uuid()[0..5], x.name(), y.name());

    sv.join_environments(&mut b, vec![&y.name()]).unwrap();
    println!(">>> Entity {} joined {}", &b.uuid()[0..5], y.name());

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effect 'hello' to {}", x.name());
    sv.submit_effect(Effect::Ascii("hello".into()), &x.name()).unwrap();

    println!(">>> Sending effect 'world' to {}", y.name());
    sv.submit_effect(Effect::Ascii("world".into()), &y.name()).unwrap();

    thread::sleep(Duration::from_millis(500));

    sv.wait_for_kill_signal().unwrap();
}

fn test3() {
    let mut sv = Supervisor::new().unwrap();

    let x = sv.create_environment("X").unwrap();
    println!(">>> Created environment X");

    thread::sleep(Duration::from_millis(500));

    let mut a = sv.create_entity().unwrap();
    println!(">>> Created entity {}", &a.uuid()[0..5]);

    sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    println!(">>> Entity {} joined X", &a.uuid()[0..5]);

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effects to X");
    for s in "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".chars().map(|c| c.to_string()) {
        sv.submit_effect(Effect::Ascii(s), "X").expect("error sending msg");
    }

    thread::sleep(Duration::from_millis(1000));

    sv.wait_for_kill_signal().expect("error waiting for ctrl-c");
}

fn test4() {
    let mut sv = Supervisor::new().unwrap();

    let x = sv.create_environment("X").unwrap();
    let y = sv.create_environment("Y").unwrap();
    let mut a = sv.create_entity().unwrap();
    sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    sv.affect_environments(&mut a, vec![&y.name()]).unwrap();

    sv.submit_effect(Effect::Ascii("hello".into()), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    sv.wait_for_kill_signal().expect("error waiting for ctrl-c");
}

fn test5() {
    let mut sv = Supervisor::new().unwrap();

    let x = sv.create_environment("X").unwrap();
    let y = sv.create_environment("Y").unwrap();
    let z = sv.create_environment("Z").unwrap();

    let mut a = sv.create_entity().unwrap();

    sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    sv.affect_environments(&mut a, vec![&y.name(), &z.name()]).unwrap();

    sv.submit_effect(Effect::Ascii("hello".into()), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    sv.wait_for_kill_signal().expect("error waiting for ctrl-c");
}

struct ReverseQubic;
impl EntityCore for ReverseQubic {
    fn process_effect(&self, effect: Effect) -> Effect {
        let result = match effect {
            Effect::Ascii(s) => Effect::Ascii(s.chars().rev().collect::<String>()),
            _ => Effect::Empty,
        };
        result
    }
}

struct UppercaseQubic;
impl EntityCore for UppercaseQubic {
    fn process_effect(&self, effect: Effect) -> Effect {
        let result = match effect {
            Effect::Ascii(s) => Effect::Ascii(s.to_uppercase()),
            _ => Effect::Empty,
        };
        result
    }
}
fn test6() {
    let mut sv = Supervisor::new().unwrap();

    // Input environment
    let x = sv.create_environment("X").unwrap();

    // Return environments
    let y = sv.create_environment("Y").unwrap();
    let z = sv.create_environment("Z").unwrap();

    thread::sleep(Duration::from_millis(500));

    // An entity that reverses an ASCII string
    let mut a = sv.create_entity().unwrap();
    a.inject_core(Box::new(ReverseQubic));
    println!(">>> Created entity {} that reverses ASCII strings", &a.uuid()[0..5]);

    // An entity that uppercases an ASCII string
    let mut b = sv.create_entity().unwrap();
    b.inject_core(Box::new(UppercaseQubic));
    println!(">>> Created entity {} that uppercases ASCII strings", &b.uuid()[0..5]);

    // Make both entities listen to environment X
    sv.join_environments(&mut a, vec![&x.name()]).unwrap();
    sv.join_environments(&mut b, vec![&x.name()]).unwrap();

    // Connect entity A to return environment Y, and entity B to Z.
    sv.affect_environments(&mut a, vec![&y.name()]).unwrap();
    sv.affect_environments(&mut b, vec![&z.name()]).unwrap();

    thread::sleep(Duration::from_millis(500));

    // Send 'hello' to input environment X
    println!(">>> Sending effect 'hello' to {}", x.name());
    sv.submit_effect(Effect::Ascii("hello".into()), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    sv.wait_for_kill_signal().unwrap();
}
