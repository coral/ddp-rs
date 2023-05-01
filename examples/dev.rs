use ddp_rs::controller;
use std::{thread, time};

fn main() {
    let mut v = controller::Controller::new().unwrap();

    let (mut c, recv) = v
        .connect(
            "10.0.1.9:4048",
            ddp_rs::protocol::PixelConfig::default(),
            ddp_rs::protocol::ID::default(),
        )
        .unwrap();

    c.write(
        &vec![255, 255, 255, 128, 128, 12, 128, 128, 12, 128, 255, 12],
        0,
    )
    .unwrap();

    let resp = recv.recv().unwrap();
    dbg!(resp);

    let ten_millis = time::Duration::from_secs(10);

    thread::sleep(ten_millis);

    dbg!(v);
}
