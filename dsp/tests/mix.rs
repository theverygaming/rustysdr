use volk_rs::{Complex, vec::AlignedVec};
use dsp::block::DspBlock;

#[test]
fn mixer_works() {
    let mut mx = dsp::mix::Mixer::new(100.0, 5000.0);
    mx.set(100.0, 5000.0);
    let mut input: AlignedVec<Complex<f32>> = AlignedVec::from_elem(Complex {re: 0.4, im: 0.2}, 500);
    let mut output: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(500);
    mx.process(&mut input, &mut output);
}
