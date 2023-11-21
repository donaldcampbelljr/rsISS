use crate::iss::{get_position, Iss};
use std::{thread, time};

pub mod iss;


fn main() {
    println!("Hello, world!");

    let mut iss = Iss::new();
    let delay = time::Duration::from_secs(2);
    thread::sleep(delay);
    iss.update_position();

    println!("ENDING PROGRAM");
}
