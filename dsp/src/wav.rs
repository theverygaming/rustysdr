use crate::block::Block;
use crate::libwav::WavReaderTrait;
use crate::stream::Stream;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use volk_rs::Complex;

pub struct WavReaderBlock<T, R: WavReaderTrait> {
    wav_reader: Mutex<R>,
    output: Arc<Stream<T>>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
    reader_id: Mutex<usize>,
}

impl<R: WavReaderTrait> WavReaderBlock<f32, R> {
    pub fn new(stream_size: usize, wav_reader: R) -> Arc<Self> {
        if wav_reader.get_channels() != 1 {
            panic!("expected one input channel");
        }
        Arc::new(WavReaderBlock {
            wav_reader: Mutex::new(wav_reader),
            output: Stream::new(stream_size),
            thread_handle: Mutex::new(None),
            reader_id: Mutex::new(0),
        })
    }
}

impl<R: WavReaderTrait> WavReaderBlock<Complex<f32>, R> {
    pub fn new(stream_size: usize, wav_reader: R) -> Arc<Self> {
        if wav_reader.get_channels() != 2 {
            panic!("expected two input channels");
        }
        Arc::new(WavReaderBlock {
            wav_reader: Mutex::new(wav_reader),
            output: Stream::new(stream_size),
            thread_handle: Mutex::new(None),
            reader_id: Mutex::new(0),
        })
    }
}

crate::impl_block_trait!(WavReaderBlockImpl);

impl<T: 'static, R: WavReaderTrait + Send + 'static> Block<T, T> for Arc<WavReaderBlock<T, R>>
where
    WavReaderBlock<T, R>: WavReaderBlockImpl,
    T: Send,
{
    fn get_input(&mut self) -> Vec<Arc<Stream<T>>> {
        vec![]
    }

    fn get_output(&mut self) -> Vec<Arc<Stream<T>>> {
        vec![self.output.clone()]
    }

    crate::impl_block_methods!();
}

impl<R: WavReaderTrait> WavReaderBlockImpl for WavReaderBlock<f32, R> {
    fn run(&self) -> bool {
        let mut reader = self.wav_reader.lock().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();
        let n_read = buf_w.len();

        match reader.read_samples(&mut buf_w) {
            Ok(()) => {}
            Err(e) => {
                return false;
            }
        }
        drop(buf_w);

        if !self.output.swap(n_read) {
            return false;
        }
        true
    }
}

impl<R: WavReaderTrait> WavReaderBlockImpl for WavReaderBlock<Complex<f32>, R> {
    fn run(&self) -> bool {
        let mut reader = self.wav_reader.lock().unwrap();
        let mut buf_w = self.output.buf_write.lock().unwrap();
        let n_read = buf_w.len();

        match reader.read_complex(&mut buf_w) {
            Ok(()) => {}
            Err(e) => {
                return false;
            }
        }
        drop(buf_w);

        if !self.output.swap(n_read) {
            return false;
        }
        true
    }
}
