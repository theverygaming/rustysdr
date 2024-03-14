use std::fs::File;
use volk_rs::{Complex, vec::AlignedVec};
use dsp::block::DspBlock;
use dsp::chain::DspChain;
use dsp::mix::Mixer;
use dsp::fmnr::FMNr;

fn main() {
    let mut reader = dsp::wav::Reader::new(File::open(std::env::temp_dir().join("/home/user/Downloads/rustysdr/HDSDR_20220820_023025Z_920800kHz_RF.wav")).unwrap(), false).unwrap();
    let mut writer = dsp::wav::Writer::new(File::create(std::env::temp_dir().join("/home/user/Downloads/rustysdr/out.wav")).unwrap(), reader.get_samplerate(), reader.get_channels(), reader.get_sample_format()).unwrap();
    
    let mut buffer: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(2048);
    let mut buffer2: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(2048);

    let mut chain = DspChain::new();
    //chain.add_block(Box::new(Mixer::new(1000000.0, reader.get_samplerate().into())));
    chain.add_block(Box::new(FMNr::new(32, 2048)));
    
    let mut n_samps: usize = 0;

    let start = std::time::Instant::now();
    while let Ok(()) = reader.read_complex(&mut buffer) {
        chain.process(&mut buffer, &mut buffer2);
        writer.write_complex(&buffer2).unwrap();
        n_samps += buffer2.len();
    }
    let duration = start.elapsed().as_secs_f64();
    writer.flush().unwrap();

    println!("sample count: {} (ran @ {} S/s)", n_samps, n_samps as f64 / duration);
}
