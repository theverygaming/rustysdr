use dsp::am::AmDemod;
use dsp::block::{DspBlock, DspBlockConv};
use dsp::chain::DspChain;
use dsp::filters;
use dsp::fir::FirFilter;
use dsp::fm_cochannel::FMCochannelCancel;
use dsp::fmnr::FMNr;
use dsp::mix::Mixer;
use dsp::resamp::RationalResampler;
use dsp::volk_rs::{vec::AlignedVec, Complex};
use std::fs::File;
mod rtl_tcp;

fn main() {
    let f_in_path = std::env::args().nth(1).expect("missing input file arg");
    let f_out_path = std::env::args().nth(2).expect("missing output file arg");
    let fft_size = std::env::args().nth(3).expect("missing fft size arg").parse::<usize>().unwrap();

    let mut reader = dsp::wav::Reader::new(File::open(f_in_path).unwrap(), false).unwrap();
    let mut writer = dsp::wav::Writer::new(File::create(f_out_path).unwrap(), 1024000 / 40, reader.get_channels(), dsp::wav::WavSampleFormat::S16).unwrap();
    let mut writer_real = dsp::wav::Writer::new(File::create("/tmp/out_real.wav").unwrap(), 1024000 / 40, 1, dsp::wav::WavSampleFormat::S16).unwrap();

    let buf_len: usize = 1048576;

    let mut buffer: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(buf_len);

    let mut chain = DspChain::new();
    chain.add_block(Box::new(Mixer::new(-305000.0, 1024000 as f64)));
    chain.add_block(Box::new(RationalResampler::new(1, 40)));
    let mut taps = AlignedVec::new_zeroed(127);
    dsp::filters::lowpass(&mut taps, (1024000 / 40) as f32, 5000.0, 1.0);
    let mut fir = Box::new(FirFilter::new());
    fir.set_taps(taps);
    chain.add_block(fir);
    //chain.add_block(Box::new(FMCochannelCancel::new(fft_size, 2)));
    //chain.add_block(Box::new(FMNr::new(fft_size)));
    //chain.add_block(Box::new(RationalResampler::new(1, 256)));
    //chain.add_block(Box::new(FMCochannelCancel::new(fft_size, 8, buf_len)));
    //chain.add_block(Box::new(RationalResampler::new(1, 2, buf_len)));

    chain.set_input_size(buf_len);

    let mut buffer2: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(chain.compute_output_size(buf_len));
    let mut buffer3: AlignedVec<f32> = AlignedVec::new_zeroed(chain.compute_output_size(buf_len));

    let mut n_samps: usize = 0;

    let mut rtl = rtl_tcp::RtlTcpClient::new("127.0.0.1:1234");
    rtl.set_freq(7000000 + 125000000);
    rtl.set_sr(1024000);

    let mut am_demod = AmDemod::new();

    let start = std::time::Instant::now();
    /*while let Ok(()) = reader.read_complex(&mut buffer) {
        chain.process(&mut buffer, &mut buffer2);
        writer.write_complex(&buffer2).unwrap();
        n_samps += buffer.len();
        if n_samps > 20000000 {
            break;
        }
    }*/
    loop {
        rtl.read(&mut buffer);
        chain.process(&mut buffer, &mut buffer2);
        writer.write_complex(&buffer2).unwrap();
        am_demod.process(&mut buffer2, &mut buffer3);
        writer_real.write_samples(&buffer3).unwrap();
        n_samps += buffer.len();
        if n_samps > 1024000 * 20 {
            break;
        }
    }
    let duration = start.elapsed().as_secs_f64();
    writer.flush().unwrap();
    writer_real.flush().unwrap();

    println!("processed {} input samples (ran @ {} S/s)", n_samps, n_samps as f64 / duration);
}
