use std::fs::File;
use volk_rs::{Complex, vec::AlignedVec};
//use dsp::block::{Block};
use dsp::stream::{Stream};
use std::thread;

fn main() {
    let s = Stream::<f32>::new(1024);

    let s_clone = s.clone();
    thread::spawn(move || {
        while true {
            println!("eeping...");
            thread::sleep(std::time::Duration::from_millis(1500));
            let n = s_clone.read().unwrap();
            s_clone.flush();
            println!("done eeping {}", n);
        }
    });

    thread::sleep(std::time::Duration::from_millis(1500));
    s.swap(1024);
    println!("first swap done");
    s.swap(1023);
    println!("second swap done");
    s.swap(1022);
    println!("third swap done");
    thread::sleep(std::time::Duration::from_millis(2000));
    println!("exiting");
}
