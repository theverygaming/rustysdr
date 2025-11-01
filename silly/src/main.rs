use dsp::libwav::WavReaderTrait;
use dsp::volk_rs::{vec::AlignedVec, Complex};
use std::fs::File;
use std::{boxed::Box};
use dsp::am::AmDemod;
use dsp::stream::Stream;
use dsp::block::DspBlockConv;
mod client;
mod real_vfo;
mod buffered_rt_stream;

fn main() {
    /*let refcount_reader = std::sync::Arc::new(std::sync::Mutex::new(
        dsp::libwav::Reader::new(File::open(
            //"/home/user/.config/sdrpp/recordings/baseband_0Hz_08-54-41_24-12-2024.wav"
            "/home/user/.config/sdrpp/recordings/SAQ_2024-12-24.wav"
            //"/home/user/Downloads/sample.wav"
        ).unwrap(), true).unwrap(),
    ));*/
    //let mut reader = dsp::libwav::Reader::new(File::open(std::env::temp_dir().join("/home/user/.config/sdrpp/recordings/baseband_0Hz_08-54-41_24-12-2024.wav")).unwrap(), true).unwrap();
    let mut reader = dsp::libwav::Reader::new(File::open(std::env::temp_dir().join("/home/user/.config/sdrpp/recordings/baseband_169875337Hz_20-57-09_30-10-2025.wav")).unwrap(), true).unwrap();
    //let mut writer = dsp::libwav::Writer::new(File::create(std::env::temp_dir().join("/home/user/Downloads/rustysdr/out.wav")).unwrap(), reader.get_samplerate(), reader.get_channels(), reader.get_sample_format()).unwrap();

    let mut buffer: AlignedVec<Complex<f32>> = AlignedVec::from_elem(Complex { re: 0.0, im: 0.0 }, 1000);

    //reader.read_complex(&mut buffer).unwrap();
    //writer.write_complex(&buffer).unwrap();

    let raw_iq_stream = Stream::<Complex<f32>>::new(1048576);

    let source_raw_iq_stream = raw_iq_stream.clone();
    // IQ source
    std::thread::spawn(move || {
        source_raw_iq_stream.start_writer();
        loop {
            let mut buf_w = source_raw_iq_stream.buf_write.lock().unwrap();
            let n_write = buf_w.len();
            reader.read_complex(&mut buf_w).unwrap();
            drop(buf_w);
            if !source_raw_iq_stream.swap(n_write) {
                //break
                panic!("what");
            }
        }
    });

    let server = std::net::TcpListener::bind("127.0.0.1:10000").unwrap();
    for stream in server.incoming() {
        // let reader_clone = refcount_reader.clone(); // increments refcount, mutex makes sure it's locked
        let new_raw_iq_stream = raw_iq_stream.clone();
        let vfo_out_stream = Stream::<f32>::new(65536);
        let vfo_out_stream2 = vfo_out_stream.clone();
        // VFO worker
        std::thread::spawn(move || {
            let mut vfo = real_vfo::RealVFO::new(Box::new(AmDemod::new()));
            vfo.set_input_size(1048576);
            let vfo_out_size = vfo.compute_output_size(1048576);
            let reader_id = new_raw_iq_stream.start_reader();
            loop {
                let n_read = new_raw_iq_stream.read(reader_id).expect("reader should not stop lmao");
                let buf_r = new_raw_iq_stream.buf_read.read().unwrap();
                let mut buf_w = vfo_out_stream.buf_write.lock().unwrap();
                
                vfo.process(&buf_r[..n_read], &mut buf_w[..vfo_out_size]);
                new_raw_iq_stream.flush(reader_id);
                drop(buf_w);
                vfo_out_stream.swap(vfo_out_size);
            }
        });
        // VFO -> web buffer worker
        std::thread::spawn(move || {
            let mut bs = buffered_rt_stream::BufferedRTStream::new(vfo_out_stream2, 2);
            let web_in_stream2 = bs.output.clone();
            // web worker
            std::thread::spawn(move || {
                let mut wsc = client::WSClient::new(tungstenite::accept(stream.unwrap()).unwrap(), web_in_stream2);
                wsc.serve();
            });
            bs.run();
        });
    }
}
