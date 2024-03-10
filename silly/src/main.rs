use std::fs::File;
use volk_rs::{types::complex, vec::AlignedVec};


fn main() {
    let refcount_reader = std::sync::Arc::new(
        std::sync::Mutex::new(
            dsp::wav::Reader::new(
                File::open(std::env::temp_dir().join("/home/user/Downloads/rustysdr/doom.wav")).unwrap(),
                true
            ).unwrap()
        )
    );
    //let reader = dsp::wav::Reader::new(File::open(std::env::temp_dir().join("/home/user/Downloads/rustysdr/doom.wav")).unwrap(), true).unwrap();
    //let mut writer = dsp::wav::Writer::new(File::create(std::env::temp_dir().join("/home/user/Downloads/rustysdr/out.wav")).unwrap(), reader.get_samplerate(), reader.get_channels(), reader.get_sample_format()).unwrap();
    
    //let mut buffer: AlignedVec<complex<f32>> = AlignedVec::from_elem(complex { r: 0.0, i: 0.0 }, 1000);

    //reader.read_complex(&mut buffer).unwrap();
    //writer.write_complex(&buffer).unwrap();

    let server = std::net::TcpListener::bind("127.0.0.1:10000").unwrap();
    for stream in server.incoming() {
        let reader_clone = refcount_reader.clone(); // increments refcount, mutex makes sure it's locked
        std::thread::spawn (move || {
            let mut counter: u64 = 0;
            let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read().unwrap(); // TODO: handle ConnectionClosed

                // We do not want to send back ping/pong messages.
                //if msg.is_binary() || msg.is_text() {
                //    websocket.send(msg).unwrap();
                //}
                let mut buffer: AlignedVec<f32> = AlignedVec::from_elem(0.0, 4800);
                reader_clone.lock().unwrap().read_samples(&mut buffer).unwrap();
                let e: Vec<u8>;
                unsafe {
                    e = std::slice::from_raw_parts(buffer.as_ptr() as *const u8, buffer.len() * std::mem::size_of::<f32>()).to_vec();
                }
                websocket.send(tungstenite::Message::Text(counter.to_string()));
                counter += 1;
                websocket.send(tungstenite::Message::Binary(e));
            }
        });
    }
}
