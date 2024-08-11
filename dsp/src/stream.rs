use std::sync::{Arc, Mutex, Condvar};
use volk_rs::vec::AlignedVec;

// TODO: implement starting & stopping streams

pub struct Stream<T> {
    pub buf_write: Mutex<AlignedVec<T>>,
    pub buf_read: Mutex<AlignedVec<T>>,
    write_size: Mutex<usize>,
    read_done: Mutex<bool>,
    read_done_cv: Condvar,
    write_done: Mutex<bool>,
    write_done_cv: Condvar,
}

impl<T> Stream<T> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(Stream {
            buf_write: Mutex::new(AlignedVec::new_zeroed(stream_size)),
            buf_read: Mutex::new(AlignedVec::new_zeroed(stream_size)),
            write_size: Mutex::new(0),
            read_done: Mutex::new(true),
            read_done_cv: Condvar::new(),
            write_done: Mutex::new(false),
            write_done_cv: Condvar::new(),
        })
    }

    // swaps the read & write buffers, called after write is done
    // returns false if the writer has been stopped
    pub fn swap(self: &Arc<Self>, n: usize) -> bool {
        // eep until a read is done
        let mut read_done = self.read_done.lock().unwrap();
        while !*read_done {
            read_done = self.read_done_cv.wait(read_done).unwrap();
        }

        std::mem::swap(&mut self.buf_write.lock().unwrap(), &mut self.buf_read.lock().unwrap());
        *read_done = false;

        *self.write_size.lock().unwrap() = n;
        *self.write_done.lock().unwrap() = true;
        self.write_done_cv.notify_all();

        true
    }

    // reads data from the buffer, returns size written, Option is None when the reader is stopped
    pub fn read<'a>(self: &'a Arc<Self>) -> Option<usize> {
        // eep until a write is done
        let mut write_done = self.write_done.lock().unwrap();
        while !*write_done {
            write_done = self.write_done_cv.wait(write_done).unwrap();
        }

        Some(*self.write_size.lock().unwrap())
    }

    // called when reading has been finished
    pub fn flush(self: &Arc<Self>) {
        *self.read_done.lock().unwrap() = true;
        self.read_done_cv.notify_all();
    }
}
