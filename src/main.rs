#![allow(dead_code)]

use reee::node::Node;
use reee::eee::Effect;
use reee::eee::Entity;

use std::thread;
use std::time::Duration;

fn main() {
    test6();
}

fn test0() {
    let mut node = Node::new().unwrap();
    node.init();
    node.create_environment("X").unwrap();
    node.create_entity().unwrap();
    node.run().unwrap();
}

// Simplest setup
fn test1() {
    let mut node = Node::new().unwrap();

    let x = node.create_environment("X").unwrap();
    println!(">>> Created environment X");

    thread::sleep(Duration::from_millis(500));

    let mut a = node.create_entity().unwrap();
    println!(">>> Created entity {}", &a.uuid()[0..5]);

    node.join_environments(&mut a, vec![&x.name()]).unwrap();
    println!(">>> Entity {} joined {}", &a.uuid()[0..5], x.name());

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effect 'hello' to {}", x.name());
    node.submit_effect(Effect::from("hello"), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    node.run().unwrap();
}

fn test2() {
    let mut node = Node::new().unwrap();

    let x = node.create_environment("X").unwrap();
    let y = node.create_environment("Y").unwrap();
    println!(">>> Created environments {}, {}", x.name(), y.name());

    thread::sleep(Duration::from_millis(500));

    let mut a = node.create_entity().unwrap();
    let mut b = node.create_entity().unwrap();
    println!(">>> Created entities {}, {}", &a.uuid()[0..5], &b.uuid()[0..5]);

    node.join_environments(&mut a, vec![&x.name(), &y.name()]).unwrap();
    println!(">>> Entity {} joined {}, {}", &a.uuid()[0..5], x.name(), y.name());

    node.join_environments(&mut b, vec![&y.name()]).unwrap();
    println!(">>> Entity {} joined {}", &b.uuid()[0..5], y.name());

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effect 'hello' to {}", x.name());
    node.submit_effect(Effect::from("hello"), &x.name()).unwrap();

    println!(">>> Sending effect 'world' to {}", y.name());
    node.submit_effect(Effect::from("world"), &y.name()).unwrap();

    thread::sleep(Duration::from_millis(500));

    node.run().unwrap();
}

fn test3() {
    let mut node = Node::new().unwrap();

    let x = node.create_environment("X").unwrap();
    println!(">>> Created environment X");

    thread::sleep(Duration::from_millis(500));

    let mut a = node.create_entity().unwrap();
    println!(">>> Created entity {}", &a.uuid()[0..5]);

    node.join_environments(&mut a, vec![&x.name()]).unwrap();
    println!(">>> Entity {} joined X", &a.uuid()[0..5]);

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effects to X");
    for s in "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".chars().map(|c| c.to_string()) {
        node.submit_effect(Effect::from(s), "X").expect("error sending msg");
    }

    thread::sleep(Duration::from_millis(1000));

    node.run().expect("error waiting for ctrl-c");
}

fn test4() {
    let mut node = Node::new().unwrap();

    let x = node.create_environment("X").unwrap();
    let y = node.create_environment("Y").unwrap();

    let mut a = node.create_entity().unwrap();

    node.join_environments(&mut a, vec![&x.name()]).unwrap();
    node.affect_environments(&mut a, vec![&y.name()]).unwrap();
    node.submit_effect(Effect::from("hello"), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    node.run().expect("error waiting for ctrl-c");
}

fn test5() {
    let mut node = Node::new().unwrap();

    let x = node.create_environment("X").unwrap();
    let y = node.create_environment("Y").unwrap();
    let z = node.create_environment("Z").unwrap();

    let mut a = node.create_entity().unwrap();

    node.join_environments(&mut a, vec![&x.name()]).unwrap();
    node.affect_environments(&mut a, vec![&y.name(), &z.name()]).unwrap();

    node.submit_effect(Effect::from("hello"), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    node.run().expect("error waiting for ctrl-c");
}

struct StringReverse;
impl Entity for StringReverse{
    fn process_effect(&mut self, effect: Effect, _environment: &str) -> Effect {
        let result = match effect {
            Effect::String(s) => Effect::from(s.chars().rev().collect::<String>()),
            _ => Effect::Empty,
        };
        result
    }
}

struct StringUppercase;
impl Entity for StringUppercase {
    fn process_effect(&mut self, effect: Effect, _environment: &str) -> Effect {
        let result = match effect {
            Effect::String(s) => Effect::from(s.to_uppercase()),
            _ => Effect::Empty,
        };
        result
    }
}

// Customized Entities
fn test6() {
    let mut node = Node::new().unwrap();

    // Input environment
    let x = node.create_environment("X").unwrap();

    // Return environments
    let y = node.create_environment("Y").unwrap();
    let z = node.create_environment("Z").unwrap();

    thread::sleep(Duration::from_millis(500));

    // An entity that reverses an ASCII string
    let mut a = node.create_entity().unwrap();
    a.inject_core(Box::new(StringReverse));
    println!(">>> Created entity {} that reverses ASCII strings", &a.uuid()[0..5]);

    // An entity that uppercases an ASCII string
    let mut b = node.create_entity().unwrap();
    b.inject_core(Box::new(StringUppercase));
    println!(">>> Created entity {} that uppercases ASCII strings", &b.uuid()[0..5]);

    // Make both entities listen to environment X
    node.join_environments(&mut a, vec![&x.name()]).unwrap();
    node.join_environments(&mut b, vec![&x.name()]).unwrap();

    // Connect entity A to return environment Y, and entity B to Z.
    node.affect_environments(&mut a, vec![&y.name()]).unwrap();
    node.affect_environments(&mut b, vec![&z.name()]).unwrap();

    thread::sleep(Duration::from_millis(500));

    // Send 'hello' to input environment X
    println!(">>> Sending effect 'hello' to {}", x.name());
    node.submit_effect(Effect::from("hello"), &x.name()).unwrap();

    thread::sleep(Duration::from_millis(1000));

    node.run().unwrap();
}

// Entity sending effects
fn test7() {
    let mut node = Node::new().unwrap();

    let x = node.create_environment("X").unwrap();
    let mut a = node.create_entity().unwrap();
    let mut b = node.create_entity().unwrap();
    let mut c = node.create_entity().unwrap();

    node.affect_environments(&mut a, vec![&x.name()]).unwrap();

    node.join_environments(&mut b, vec![&x.name()]).unwrap();
    node.join_environments(&mut c, vec![&x.name()]).unwrap();

    // NOTE: the effect will be enqueued and processed by the supervisor in FIFO style
    //a.submit_effect(Effect::from("hello"), &x.name()).unwrap();
}
