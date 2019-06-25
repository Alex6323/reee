use ::reee::supervisor::Supervisor;

use std::thread;
use std::time::Duration;

fn main() {
    test3();
}

// Simplest setup
fn test1() {
    let mut sv = Supervisor::new().expect("creating supervisor");

    let _x = sv.create_environment("X").expect("error creating env");
    println!(">>> Created environment X");

    thread::sleep(Duration::from_millis(500));

    let a = sv.create_entity(vec!["X"]).expect("error assigning entity");
    println!(">>> Created entity: {}, subscribed to X", &a.uuid()[0..5]);

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effect 'hello' to X");
    sv.submit_effect("hello", "X").expect("error sending msg");

    thread::sleep(Duration::from_millis(1000));

    sv.wait_for_kill_signal().expect("error waiting for ctrl-c");
}

fn test2() {
    let mut sv = Supervisor::new().expect("creating supervisor");

    let _x = sv.create_environment("X").expect("error creating env");
    let _y = sv.create_environment("Y").expect("error creating env");
    println!(">>> Created environments X, Y");

    thread::sleep(Duration::from_millis(500));

    let a = sv.create_entity(vec!["X", "Y"]).expect("error assigning entity");
    let b = sv.create_entity(vec!["Y"]).expect("error assigning entity");
    println!("Created entity: {}, subscribed to X, Y", &a.uuid()[0..5]);
    println!("Created entity: {}, subscribed to Y", &b.uuid()[0..5]);

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effect 'hello' to X");
    sv.submit_effect("hello", "X").expect("error sending msg");

    println!(">>> Sending effect 'world' to Y");
    sv.submit_effect("world", "Y").expect("error sending msg");

    thread::sleep(Duration::from_millis(500));

    sv.wait_for_kill_signal().expect("error waiting for ctrl-c");
}

fn test3() {
    let mut sv = Supervisor::new().expect("creating supervisor");

    let _x = sv.create_environment("X").expect("error creating env");
    println!(">>> Created environment X");

    thread::sleep(Duration::from_millis(500));

    let a = sv.create_entity(vec!["X"]).expect("error assigning entity");
    println!(">>> Created entity: {}, subscribed to X", &a.uuid()[0..5]);

    thread::sleep(Duration::from_millis(500));

    println!(">>> Sending effects to X");
    for s in "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".chars().map(|c| c.to_string()) {
        sv.submit_effect(&s, "X").expect("error sending msg");
    }

    thread::sleep(Duration::from_millis(1000));

    sv.wait_for_kill_signal().expect("error waiting for ctrl-c");
}
