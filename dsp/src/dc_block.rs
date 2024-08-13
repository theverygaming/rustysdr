use std::sync::{Arc, Mutex};
use std::thread;
use volk_rs::Complex;
use crate::block::Block;
use crate::stream::Stream;

// https://github.com/AlexandreRouma/SDRPlusPlus/blob/e1c48e9a1f6eca5b7c0a9cc8e0029181ac6c5f2d/core/src/dsp/correction/dc_blocker.h#L4

pub struct DcBlock<T> {
    offset: T,
    rate: f32,
    input: Arc<Stream<T>>,
    output: Arc<Stream<T>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl DcBlock<f32> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(DcBlock {
            offset: 0.0,
            rate: 0.0001,
            input: Stream::new(stream_size),
            output: Stream::new(stream_size),
            thread_handle: None,
        })
    }
}

impl DcBlock<Complex<f32>> {
    pub fn new(stream_size: usize) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(DcBlock {
            offset: Complex {re: 0.0, im: 0.0},
            rate: 0.01,
            input: Stream::new(stream_size),
            output: Stream::new(stream_size),
            thread_handle: None,
        }))
    }
}

crate::impl_block!(DcBlock, DcBlockImpl);

impl DcBlockImpl for DcBlock<f32> {
    fn run(&mut self) -> bool {
        true
    }
}

impl DcBlockImpl for DcBlock<Complex<f32>> {
    fn run(&mut self) -> bool {
        true
    }
}


/*

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

*/
