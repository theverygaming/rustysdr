use dsp::block::Block;
use dsp::dc_block::DcBlock;
use dsp::libwav::WavReaderTrait;
use dsp::stream::Stream;
use dsp::volk_rs::{vec::AlignedVec, Complex};
use dsp::wav::WavReaderBlock;
use std::fs::File;
use std::thread;

fn main() {
    let mut reader = dsp::libwav::Reader::new(File::open("/tmp/in.wav").unwrap(), false).unwrap();
    if reader.get_channels() != 1 {
        panic!("expected one input channel");
    }
    let mut writer = dsp::libwav::Writer::new(File::create("/tmp/out.wav").unwrap(), reader.get_samplerate(), 1, dsp::libwav::WavSampleFormat::S16).unwrap();

    let mut block_read = WavReaderBlock::<f32, dsp::libwav::Reader<std::fs::File>>::new(1024, reader);
    let mut block_proc = DcBlock::<f32>::new(1024);

    let mut block_write = block_proc.clone();
    let writer_thread = thread::spawn(move || loop {
        let stream = &block_write.get_output()[0];
        let n_read = stream.read().unwrap();
        let buf = stream.buf_read.lock().unwrap();

        println!("write");

        writer.write_samples(&buf[0..n_read]).unwrap();
        writer.flush().unwrap();

        stream.flush();
    });

    block_read.start();
    block_proc.start();

    writer_thread.join().unwrap();
}
