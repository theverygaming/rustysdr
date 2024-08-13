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
    pub fn new(stream_size: usize) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(DcBlock {
            offset: 0.0,
            rate: 0.0001,
            input: Stream::new(stream_size),
            output: Stream::new(stream_size),
            thread_handle: None,
        }))
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
        let n_read = self.input.read().unwrap();
        let buf_r = self.input.buf_read.lock().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();

        for i in 0..n_read {
            buf_w[i] = buf_r[i] - self.offset;
            self.offset += buf_w[i] * self.rate;
        }

        self.output.swap(n_read);
        self.input.flush();
        true
    }
}

impl DcBlockImpl for DcBlock<Complex<f32>> {
    fn run(&mut self) -> bool {
        let n_read = self.input.read().unwrap();
        let buf_r = self.input.buf_read.lock().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();

        for i in 0..n_read {
            buf_w[i] = buf_r[i] - self.offset;
            self.offset += buf_w[i] * Complex {re: self.rate, im: self.rate};
        }

        self.output.swap(n_read);
        self.input.flush();
        true
    }
}
