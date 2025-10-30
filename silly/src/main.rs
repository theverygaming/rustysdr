use dsp::libwav::WavReaderTrait;
use dsp::volk_rs::{vec::AlignedVec, Complex};
use std::fs::File;
mod client;
mod real_vfo;

fn main() {
    /*let refcount_reader = std::sync::Arc::new(std::sync::Mutex::new(
        dsp::libwav::Reader::new(File::open(
            //"/home/user/.config/sdrpp/recordings/baseband_0Hz_08-54-41_24-12-2024.wav"
            "/home/user/.config/sdrpp/recordings/SAQ_2024-12-24.wav"
            //"/home/user/Downloads/sample.wav"
        ).unwrap(), true).unwrap(),
    ));*/
    let mut reader = dsp::libwav::Reader::new(File::open(std::env::temp_dir().join("/home/user/.config/sdrpp/recordings/baseband_0Hz_08-54-41_24-12-2024.wav")).unwrap(), true).unwrap();
    //let mut writer = dsp::libwav::Writer::new(File::create(std::env::temp_dir().join("/home/user/Downloads/rustysdr/out.wav")).unwrap(), reader.get_samplerate(), reader.get_channels(), reader.get_sample_format()).unwrap();

    let mut buffer: AlignedVec<Complex<f32>> = AlignedVec::from_elem(Complex { re: 0.0, im: 0.0 }, 1000);

    reader.read_complex(&mut buffer).unwrap();
    //writer.write_complex(&buffer).unwrap();

    let server = std::net::TcpListener::bind("127.0.0.1:10000").unwrap();
    for stream in server.incoming() {
        // let reader_clone = refcount_reader.clone(); // increments refcount, mutex makes sure it's locked
        std::thread::spawn(move || {
            let mut wsc = client::WSClient::new(tungstenite::accept(stream.unwrap()).unwrap(), );
            wsc.serve();
        });
    }
}
