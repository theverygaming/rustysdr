use dsp::block::Block;
use dsp::dc_block::DcBlock;
use dsp::stream::Stream;
use dsp::volk_rs::{vec::AlignedVec, Complex};
use std::fs::File;
use std::thread;

fn main() {
    let mut reader = dsp::wav::Reader::new(File::open("/tmp/in.wav").unwrap(), false).unwrap();
    if reader.get_channels() != 1 {
        panic!("expected one input channel");
    }
    let mut writer = dsp::wav::Writer::new(File::create("/tmp/out.wav").unwrap(), reader.get_samplerate(), 1, dsp::wav::WavSampleFormat::S16).unwrap();

    let mut block = DcBlock::<f32>::new(1024);

    let mut block_write = block.clone();
    let writer_thread = thread::spawn(move || loop {
        let stream = &block_write.get_output()[0];
        let n_read = stream.read().unwrap();
        let buf = stream.buf_read.lock().unwrap();

        writer.write_samples(&buf[0..n_read]).unwrap();
        writer.flush().unwrap();

        stream.flush();
    });

    let mut block_read = block.clone();
    let reader_thread = thread::spawn(move || loop {
        let stream = &block_read.get_input()[0];
        let mut buf = stream.buf_write.lock().unwrap();

        let n_read = buf.len();
        match reader.read_samples(&mut buf) {
            Ok(()) => {}
            Err(e) => {
                block_read.stop();
                block_read.get_input()[0].stop_writer();
                block_read.get_output()[0].stop_reader();
                return;
            }
        }
        drop(buf);

        if !stream.swap(n_read) {
            return;
        }
    });

    block.start();

    writer_thread.join().unwrap();
    reader_thread.join().unwrap();
}
