use crate::block::Block;
use crate::stream::Stream;
use std::sync::{Arc, Mutex};
use std::thread;
use volk_rs::Complex;

// https://github.com/AlexandreRouma/SDRPlusPlus/blob/e1c48e9a1f6eca5b7c0a9cc8e0029181ac6c5f2d/core/src/dsp/correction/dc_blocker.h#L4

pub struct DcBlock<T> {
    offset: Mutex<T>,
    rate: Mutex<f32>,
    input: Arc<Stream<T>>,
    output: Arc<Stream<T>>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl DcBlock<f32> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(DcBlock {
            offset: Mutex::new(0.0),
            rate: Mutex::new(0.0001),
            input: Stream::new(stream_size),
            output: Stream::new(stream_size),
            thread_handle: Mutex::new(None),
        })
    }
}

impl DcBlock<Complex<f32>> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(DcBlock {
            offset: Mutex::new(Complex { re: 0.0, im: 0.0 }),
            rate: Mutex::new(0.01),
            input: Stream::new(stream_size),
            output: Stream::new(stream_size),
            thread_handle: Mutex::new(None),
        })
    }
}

crate::impl_block!(
    DcBlock,
    DcBlockImpl,
    fn get_input(&mut self) -> Option<Arc<Stream<T>>> {
        Some(self.input.clone())
    },
    fn get_output(&mut self) -> Option<Arc<Stream<T>>> {
        Some(self.output.clone())
    }
);

impl DcBlockImpl for DcBlock<f32> {
    fn run(&self) -> bool {
        let n_read = self.input.read().unwrap();
        let buf_r = self.input.buf_read.lock().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();

        let mut offset = self.offset.lock().unwrap();
        let rate = self.rate.lock().unwrap();
        for i in 0..n_read {
            buf_w[i] = buf_r[i] - *offset;
            *offset += buf_w[i] * *rate;
        }
        drop(buf_w);

        self.output.swap(n_read);
        self.input.flush();
        true
    }
}

impl DcBlockImpl for DcBlock<Complex<f32>> {
    fn run(&self) -> bool {
        let n_read = self.input.read().unwrap();
        let buf_r = self.input.buf_read.lock().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();

        let mut offset = self.offset.lock().unwrap();
        let rate = self.rate.lock().unwrap();
        for i in 0..n_read {
            buf_w[i] = buf_r[i] - *offset;
            *offset += buf_w[i] * Complex { re: *rate, im: *rate };
        }
        drop(buf_w);

        self.output.swap(n_read);
        self.input.flush();
        true
    }
}
