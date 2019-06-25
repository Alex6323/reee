macro_rules! sleep {
    ($duration:expr) => {
        std::thread::sleep(std::time::Duration::from_millis($duration));
    };
}
