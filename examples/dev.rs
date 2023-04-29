use ddp_rs::controller;
use std::{thread, time};

fn main() {
    let v = controller::Controller::new().unwrap();

    let ten_millis = time::Duration::from_secs(10);
    let now = time::Instant::now();

    thread::sleep(ten_millis);

    dbg!(v);
}
