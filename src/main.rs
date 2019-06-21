use ::reee::supervisor::Supervisor;

use std::thread;
use std::time::Duration;

fn main() {
    let mut sv = Supervisor::default();

    // Create two environments
    let _x = sv.create_environment("X").expect("error creating env");
    let _y = sv.create_environment("Y").expect("error creating env");

    // Create two entities and attach them to the environments in some way
    let a = sv.create_entity(vec!["X"]).expect("error assigning entity");
    println!("Created entity {}", a.uuid);

    let b = sv.create_entity(vec!["X", "Y"]).expect("error assigning entity");
    println!("Created entity {}", b.uuid);

    println!("Number of environments: {}", sv.num_environments());

    println!("Sending messages...");
    thread::sleep(Duration::from_millis(1000));

    // Send messages to both environments
    sv.submit_message("hello", "X").expect("error sending msg");
    sv.submit_message("world", "Y").expect("error sending msg");

    thread::sleep(Duration::from_millis(1000));

    sv.submit_message("cat", "X").expect("error sending msg");
    sv.submit_message("dog", "Y").expect("error sending msg");

    // Entity a should receive 'hello'
    // Entity b should receive 'hello' and 'world'
    sv.wait();
}
