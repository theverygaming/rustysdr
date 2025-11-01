use dsp::volk_rs::{vec::AlignedVec, Complex};
use dsp::stream::Stream;
use std::sync::{Arc};

pub struct BufferedRTStream<T> {
    input: Arc<Stream<T>>,
    pub output: Arc<Stream<T>>,
    buffer: AlignedVec<T>,
}

impl<T: Copy> BufferedRTStream<T> {
    pub fn new(input: Arc<Stream<T>>, buf_size_mult: usize) -> Self {
        let inp_buf_size = input.buf_read.read().unwrap().len();
        BufferedRTStream {
            input: input,
            output: Stream::<T>::new(inp_buf_size),
            buffer: AlignedVec::new_zeroed(inp_buf_size * buf_size_mult),
        }
    }

    pub fn run(&mut self) {
        let reader_id = self.input.start_reader();
        self.output.start_writer();
        loop {
            let n_read = self.input.read(reader_id).expect("reader should not stop lmao");
            let buf_r = self.input.buf_read.read().unwrap();
            let mut buf_w = self.output.buf_write.lock().unwrap();

            buf_w[..n_read].copy_from_slice(&buf_r[..n_read]);
            self.input.flush(reader_id);
            drop(buf_w);
            self.output.swap(n_read);
            //self.output.ready_to_swap();
        }
    }
}
