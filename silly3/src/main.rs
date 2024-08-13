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
        let stream = block_write.get_output().unwrap();
        let n_read = stream.read().unwrap();
        let buf = stream.buf_read.lock().unwrap();

        writer.write_samples(&buf[0..n_read]).unwrap();
        writer.flush().unwrap();

        stream.flush();
    });

    let mut block_read = block.clone();
    let reader_thread = thread::spawn(move || loop {
        let stream = block_read.get_input().unwrap();
        let mut buf = stream.buf_write.lock().unwrap();

        let n_read = buf.len();
        reader.read_samples(&mut buf).unwrap();
        drop(buf);

        stream.swap(n_read);
    });

    block.start();

    writer_thread.join().unwrap();
    reader_thread.join().unwrap();
}
