use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use volk_rs::vec::AlignedVec;

struct ReaderState {
    done_reading: bool,
}

pub struct Stream<T> {
    pub buf_write: Mutex<AlignedVec<T>>,
    pub buf_read: RwLock<AlignedVec<T>>,
    write_size: Mutex<usize>,
    readers_total: Mutex<usize>,
    readers_read: Mutex<usize>,
    reader_states: Mutex<HashMap<usize, ReaderState>>,
    next_reader_id: Mutex<usize>,
    write_cv: Condvar,
    read_cv: Condvar,
    writer_active: Mutex<bool>,
}

impl<T> Stream<T> {
    pub fn new(stream_size: usize) -> Arc<Self> {
        Arc::new(Stream {
            buf_write: Mutex::new(AlignedVec::new_zeroed(stream_size)),
            buf_read: RwLock::new(AlignedVec::new_zeroed(stream_size)),
            write_size: Mutex::new(0),
            readers_total: Mutex::new(0),
            readers_read: Mutex::new(0),
            reader_states: Mutex::new(HashMap::new()),
            next_reader_id: Mutex::new(1),
            write_cv: Condvar::new(),
            read_cv: Condvar::new(),
            writer_active: Mutex::new(true),
        })
    }

    // swaps the read & write buffers, called after write is done
    // returns false if the writer has been stopped
    pub fn swap(self: &Arc<Self>, n: usize) -> bool {
        let mut writer_active = self.writer_active.lock().unwrap();
        let mut readers_read = self.readers_read.lock().unwrap();
        let mut readers_total = self.readers_total.lock().unwrap();

        while *readers_read < *readers_total && *writer_active {
            drop(writer_active);
            drop(readers_total);
            readers_read = self.write_cv.wait(readers_read).unwrap();
            writer_active = self.writer_active.lock().unwrap();
            readers_total = self.readers_total.lock().unwrap();
        }

        if !*writer_active {
            return false;
        }

        std::mem::swap(&mut *self.buf_write.lock().unwrap(), &mut *self.buf_read.write().unwrap());
        *readers_read = 0;
        *self.write_size.lock().unwrap() = n;

        let mut states = self.reader_states.lock().unwrap();
        for state in states.values_mut() {
            state.done_reading = false;
        }
        drop(states);

        self.read_cv.notify_all();
        true
    }

    // reads data from the buffer, returns size written, Option is None when the reader is stopped
    pub fn read<'a>(self: &'a Arc<Self>, reader_id: usize) -> Option<usize> {
        let mut states = self.reader_states.lock().unwrap();
        loop {
            let state = match states.get_mut(&reader_id) {
                Some(s) => s,
                None => return None, // reader was stopped
            };

            if !state.done_reading {
                break;
            }

            states = self.read_cv.wait(states).unwrap();
        }

        Some(*self.write_size.lock().unwrap())
    }


    // called when reading has been finished
    pub fn flush(self: &Arc<Self>, reader_id: usize) {
        let mut states = self.reader_states.lock().unwrap();
        if let Some(state) = states.get_mut(&reader_id) {
            state.done_reading = true;
        }

        let mut readers_read = self.readers_read.lock().unwrap();
        *readers_read += 1;
        let readers_total = *self.readers_total.lock().unwrap();
        if *readers_read >= readers_total {
            self.write_cv.notify_all();
        }
    }

    pub fn start_reader(self: &Arc<Self>) -> usize {
        let mut id_gen = self.next_reader_id.lock().unwrap();
        let id = *id_gen;
        *id_gen += 1;

        self.reader_states.lock().unwrap().insert(id, ReaderState { done_reading: true });
        *self.readers_total.lock().unwrap() += 1;
        *self.readers_read.lock().unwrap() += 1;

        id
    }

    pub fn stop_reader(self: &Arc<Self>, id: usize) {
        let mut states = self.reader_states.lock().unwrap();
        if states.remove(&id).is_none() {
            panic!("reader stopped twice");
        }

        let mut total = self.readers_total.lock().unwrap();
        let mut done = self.readers_read.lock().unwrap();

        *total -= 1;
        if *done > *total {
            *done = *total;
        }
        self.write_cv.notify_all();
        self.read_cv.notify_all();
    }

    pub fn start_writer(self: &Arc<Self>) {
        *self.writer_active.lock().unwrap() = true;
    }

    pub fn stop_writer(self: &Arc<Self>) {
        *self.writer_active.lock().unwrap() = false;
        self.write_cv.notify_all();
        self.read_cv.notify_all();
    }
}
