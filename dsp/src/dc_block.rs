use volk_rs::vec::AlignedVec;
use fftw::plan::*;
use fftw::types::*;
use volk_rs::Complex;
use crate::block::DspBlock;
use crate::windows;

// https://github.com/AlexandreRouma/SDRPlusPlus/blob/e1c48e9a1f6eca5b7c0a9cc8e0029181ac6c5f2d/core/src/dsp/correction/dc_blocker.h#L4

pub struct DcBlock<T> {
    offset: T,
    rate: f32
}

impl DcBlock<f32> {
    pub fn new() -> Self {
        DcBlock {
            offset: 0.0,
            rate: 0.0001,
        }
    }
}

impl DcBlock<Complex<f32>> {
    pub fn new() -> Self {
        DcBlock {
            offset: Complex {re: 0.0, im: 0.0},
            rate: 0.01,
        }
    }
}

impl DspBlock<f32> for DcBlock<f32> {
    fn process(&mut self, input: &mut [f32], output: &mut [f32]) {
        for i in 0..input.len() {
            output[i] = input[i] - self.offset;
            self.offset += output[i] * self.rate;
        }
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        input_size
    }

    fn set_input_size(&mut self, input_size: usize) {}
}

impl DspBlock<Complex<f32>> for DcBlock<Complex<f32>> {
    fn process(&mut self, input: &mut [Complex<f32>], output: &mut [Complex<f32>]) {
        for i in 0..input.len() {
            output[i] = input[i] - self.offset;
            self.offset += output[i] * Complex {re: self.rate, im: self.rate};
        }
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        input_size
    }

    fn set_input_size(&mut self, _input_size: usize) {}
}
