use dsp::libwav::WavReaderTrait;
use dsp::volk_rs::{vec::AlignedVec, Complex};
use std::fs::File;

fn main() {
    let refcount_reader = std::sync::Arc::new(std::sync::Mutex::new(
        dsp::libwav::Reader::new(File::open("/home/user/Downloads/thebestfuckingmicrophone.wav").unwrap(), true).unwrap(),
    ));
    //let reader = dsp::libwav::Reader::new(File::open(std::env::temp_dir().join("/home/user/Downloads/rustysdr/doom.wav")).unwrap(), true).unwrap();
    //let mut writer = dsp::libwav::Writer::new(File::create(std::env::temp_dir().join("/home/user/Downloads/rustysdr/out.wav")).unwrap(), reader.get_samplerate(), reader.get_channels(), reader.get_sample_format()).unwrap();

    //let mut buffer: AlignedVec<complex<f32>> = AlignedVec::from_elem(complex { r: 0.0, i: 0.0 }, 1000);

    //reader.read_complex(&mut buffer).unwrap();
    //writer.write_complex(&buffer).unwrap();

    let server = std::net::TcpListener::bind("127.0.0.1:10000").unwrap();
    for stream in server.incoming() {
        let reader_clone = refcount_reader.clone(); // increments refcount, mutex makes sure it's locked
        std::thread::spawn(move || {
            let mut counter: u64 = 0;
            let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
            let mut buffer: AlignedVec<f32> = AlignedVec::from_elem(0.0, 4800);
            loop {
                let msg = match websocket.read() {
                    Ok(v) => v,
                    Err(tungstenite::error::Error::ConnectionClosed) => {
                        println!("closing WS");
                        return;
                    }
                    Err(e) => panic!("WS error: {:?}", e),
                };
                // We do not want to send back ping/pong messages.
                //if msg.is_binary() || msg.is_text() {
                //    websocket.send(msg).unwrap();
                //}
                reader_clone.lock().unwrap().read_samples(&mut buffer).unwrap();
                let audiobytes = unsafe {
                    let slice = std::slice::from_raw_parts(buffer.as_ptr() as *const u8, buffer.len() * std::mem::size_of::<f32>());
                    tungstenite::Bytes::copy_from_slice(slice)
                };
                websocket.send(tungstenite::Message::Text(counter.to_string().into()));
                counter += 1;
                websocket.send(tungstenite::Message::Binary(audiobytes));
            }
        });
    }
}
