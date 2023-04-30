use ddp_rs::controller;
use std::{thread, time};

fn main() {
    let mut v = controller::Controller::new().unwrap();

    let mut c = v
        .connect(
            "10.0.1.9:4048",
            ddp_rs::protocol::PixelConfig::default(),
            ddp_rs::protocol::ID::default(),
        )
        .unwrap();

    c.write(&vec![255, 255, 255, 128, 128, 128], 0).unwrap();

    let ten_millis = time::Duration::from_secs(10);
    let now = time::Instant::now();

    thread::sleep(ten_millis);

    dbg!(v);
}
