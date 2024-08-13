use std::sync::Arc;
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
}

impl DcBlock<f32> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(DcBlock {
            offset: 0.0,
            rate: 0.0001,
            input: Stream::new(stream_size),
            output: Stream::new(stream_size),
        })
    }
}

impl DcBlock<Complex<f32>> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(DcBlock {
            offset: Complex {re: 0.0, im: 0.0},
            rate: 0.01,
            input: Stream::new(stream_size),
            output: Stream::new(stream_size),
        })
    }
}

trait DcBlockSupportedType {
    fn run(&mut self) -> bool;
}

impl<T> Block<T, T> for Arc<DcBlock<T>>
where
    T: DcBlockSupportedType,
{
    fn get_input(&mut self) -> Arc<Stream<T>> {
        self.input.clone()
    }

    fn get_output(&mut self) -> Arc<Stream<T>> {
        self.output.clone()
    }

    fn start(&mut self) {
        let mut clone = self.clone();
        thread::spawn(move || {
            while clone.run() {}
        });
    }

    fn stop(&mut self) {
        //handle.join().unwrap();
    }
}

impl DcBlockSupportedType for DcBlock<f32> {
    fn run(&mut self) -> bool {
        true
    }
}

impl DcBlockSupportedType for DcBlock<Complex<f32>> {
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
