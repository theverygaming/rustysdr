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
    reader_id: Mutex<usize>,
}

impl DcBlock<f32> {
    pub fn new(stream_size: usize, input: Arc<Stream<f32>>) -> Arc<Self> {
        Arc::new(DcBlock {
            offset: Mutex::new(0.0),
            rate: Mutex::new(0.0001),
            input: input,
            output: Stream::new(stream_size),
            thread_handle: Mutex::new(None),
            reader_id: Mutex::new(0),
        })
    }
}

impl DcBlock<Complex<f32>> {
    pub fn new(stream_size: usize, input: Arc<Stream<Complex<f32>>>) -> Arc<Self> {
        Arc::new(DcBlock {
            offset: Mutex::new(Complex { re: 0.0, im: 0.0 }),
            rate: Mutex::new(0.01),
            input: input,
            output: Stream::new(stream_size),
            thread_handle: Mutex::new(None),
            reader_id: Mutex::new(0),
        })
    }
}

crate::impl_block!(
    DcBlock,
    DcBlockImpl,
    fn get_input(&mut self) -> Vec<Arc<Stream<T>>> {
        vec![self.input.clone()]
    },
    fn get_output(&mut self) -> Vec<Arc<Stream<T>>> {
        vec![self.output.clone()]
    }
);

impl DcBlockImpl for DcBlock<f32> {
    fn run(&self) -> bool {
        let reader_id = self.reader_id.lock().unwrap();
        let n_read = match self.input.read(*reader_id) {
            Some(x) => x,
            None => {
                return false;
            }
        };
        let buf_r = self.input.buf_read.read().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();

        let mut offset = self.offset.lock().unwrap();
        let rate = self.rate.lock().unwrap();
        for i in 0..n_read {
            buf_w[i] = buf_r[i] - *offset;
            *offset += buf_w[i] * *rate;
        }
        drop(buf_w);

        if !self.output.swap(n_read) {
            return false;
        }
        self.input.flush(*reader_id);
        true
    }
}

impl DcBlockImpl for DcBlock<Complex<f32>> {
    fn run(&self) -> bool {
        let reader_id = self.reader_id.lock().unwrap();
        let n_read = match self.input.read(*reader_id) {
            Some(x) => x,
            None => {
                return false;
            }
        };
        let buf_r = self.input.buf_read.read().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();

        let mut offset = self.offset.lock().unwrap();
        let rate = self.rate.lock().unwrap();
        for i in 0..n_read {
            buf_w[i] = buf_r[i] - *offset;
            *offset += buf_w[i] * Complex { re: *rate, im: *rate };
        }
        drop(buf_w);

        if !self.output.swap(n_read) {
            return false;
        }
        self.input.flush(*reader_id);
        true
    }
}
