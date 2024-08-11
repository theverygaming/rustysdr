use std::fs::File;
use dsp::volk_rs::{Complex, vec::AlignedVec};
//use dsp::block::{Block};
use dsp::stream::{Stream};
use std::thread;

fn main() {
    let s = Stream::<f32>::new(1024);

    s.swap(1025);

    let s_clone = s.clone();
    thread::spawn(move || {
        while true {
            let n = s_clone.read().unwrap();
            s_clone.flush();
            println!("read.. {}", n);
        }
    });

    thread::sleep(std::time::Duration::from_millis(200));
    s.swap(1024);
    println!("1 swap done");
    s.swap(1023);
    println!("2 swap done");
    s.swap(1022);
    println!("3 swap done");
    s.swap(1021);
    println!("4 swap done");
    s.swap(1020);
    println!("5 swap done");
    s.swap(1019);
    println!("6 swap done");
    s.swap(1018);
    println!("7 swap done");
    s.swap(1017);
    println!("8 swap done");
    s.swap(1016);
    println!("9 swap done");
    s.swap(1015);
    println!("10 swap done");
    s.swap(1014);
    println!("11 swap done");
    thread::sleep(std::time::Duration::from_millis(500));
    println!("exiting");
}
