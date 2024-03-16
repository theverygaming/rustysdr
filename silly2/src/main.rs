use std::fs::File;
use volk_rs::{Complex, vec::AlignedVec};
use dsp::block::DspBlock;
use dsp::chain::DspChain;
use dsp::mix::Mixer;
use dsp::fmnr::FMNr;

fn main() {
    let f_in_path = std::env::args().nth(1).expect("missing input file arg");
    let f_out_path = std::env::args().nth(2).expect("missing output file arg");
    let fft_size = std::env::args().nth(3).expect("missing fft size arg").parse::<usize>().unwrap();

    let mut reader = dsp::wav::Reader::new(File::open(f_in_path).unwrap(), false).unwrap();
    let mut writer = dsp::wav::Writer::new(File::create(f_out_path).unwrap(), reader.get_samplerate(), reader.get_channels(), reader.get_sample_format()).unwrap();

    let buf_len: usize = 1048576;

    let mut buffer: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(buf_len);
    let mut buffer2: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(buf_len);

    let mut chain = DspChain::new();
    //chain.add_block(Box::new(Mixer::new(1000000.0, reader.get_samplerate().into())));
    chain.add_block(Box::new(FMNr::new(fft_size, buf_len)));

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
