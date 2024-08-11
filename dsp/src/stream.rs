use std::sync::{Arc, Mutex, Condvar};
use volk_rs::vec::AlignedVec;

// TODO: implement starting & stopping streams

pub struct Stream<T> {
    pub buf_write: Mutex<AlignedVec<T>>,
    pub buf_read: Mutex<AlignedVec<T>>,
    write_size: Mutex<usize>,
    read_done: Mutex<bool>,
    write_cv: Condvar,
    data_ready: Mutex<bool>,
    read_cv: Condvar,
}

impl<T> Stream<T> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(Stream {
            buf_write: Mutex::new(AlignedVec::new_zeroed(stream_size)),
            buf_read: Mutex::new(AlignedVec::new_zeroed(stream_size)),
            write_size: Mutex::new(0),
            read_done: Mutex::new(true),
            write_cv: Condvar::new(),
            data_ready: Mutex::new(false),
            read_cv: Condvar::new(),
        })
    }

    // swaps the read & write buffers, called after write is done
    // returns false if the writer has been stopped
    pub fn swap(self: &Arc<Self>, n: usize) -> bool {
        // eep until a read is done
        let mut read_done = self.read_done.lock().unwrap();
        while !*read_done {
            read_done = self.write_cv.wait(read_done).unwrap();
        }

        std::mem::swap(&mut *self.buf_write.try_lock().unwrap(), &mut *self.buf_read.try_lock().unwrap());
        *read_done = false;

        *self.write_size.lock().unwrap() = n;
        *self.data_ready.lock().unwrap() = true;
        self.read_cv.notify_all();

        true
    }

    // reads data from the buffer, returns size written, Option is None when the reader is stopped
    pub fn read<'a>(self: &'a Arc<Self>) -> Option<usize> {
        // eep until a write is done
        let mut data_ready = self.data_ready.lock().unwrap();
        while !*data_ready {
            data_ready = self.read_cv.wait(data_ready).unwrap();
        }

        Some(*self.write_size.lock().unwrap())
    }

    // called when reading has been finished
    pub fn flush(self: &Arc<Self>) {
        *self.data_ready.lock().unwrap() = false;
        *self.read_done.lock().unwrap() = true;
        self.write_cv.notify_all();
    }
}
