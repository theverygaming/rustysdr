use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock, MutexGuard, TryLockError};
use volk_rs::vec::AlignedVec;
use crate::buffer::CircularBuffer;
use crate::error::DspError;

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

    pub fn ready_to_swap(self: &Arc<Self>) -> bool {
        let readers_read = self.readers_read.lock().unwrap();
        let readers_total = self.readers_total.lock().unwrap();
        return *readers_read >= *readers_total;
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

// the core idea behind these streams is from: https://github.com/ThomasHabets/rustradio/blob/3b29791b340470819eb23019f1153e6c1fd93107/src/stream.rs

pub struct Stream2<T> {
    buf: Mutex<CircularBuffer<T>>,
    cv_read: Condvar,
    cv_write: Condvar,
}

impl<T: Copy> Stream2<T> {
    fn new(nbuf: usize) -> Stream2<T> {
        Stream2 {
            buf: Mutex::new(CircularBuffer::<T>::new(nbuf)),
            cv_read: Condvar::new(),
            cv_write: Condvar::new(),
        }
    }
}


pub struct ReadStream<T> {
    stream: Arc<Stream2<T>>,
}

pub struct StreamReader<'a, T: Copy> {
    stream: Arc<Stream2<T>>,
    guard: MutexGuard<'a, CircularBuffer<T>>,
    nread: usize,
}

impl<'a, T: Copy> StreamReader<'a, T> {
    pub fn read(&self) -> (&[T], &[T]) {
        self.guard.read(self.nread)
    }

    pub fn done(&mut self) {
        if self.nread == 0 {
            return;
        }
        self.guard.consume(self.nread).unwrap();
        self.nread = 0;
        self.stream.cv_write.notify_all();
    }
}


pub struct StreamWriter<'a, T> {
    stream: Arc<Stream2<T>>,
    guard: MutexGuard<'a, CircularBuffer<T>>
}

impl<'a, T: Copy> StreamWriter<'a, T> {
    pub fn write(&mut self, data: &[T]) -> Result<(), DspError> {
        self.guard.write(data)
    }

    pub fn done(&mut self) {
        self.stream.cv_read.notify_all();
    }
}


impl<T: Copy> ReadStream<T> {
    pub fn read_lock_try(&self, nmin: usize, nmax: usize) -> Result<StreamReader<'_, T>, TryLockError<MutexGuard<'_, CircularBuffer<T>>>> {
        let guard = self.stream.buf.try_lock()?;
        if (*guard).len() < nmin {
            return Err(std::sync::TryLockError::WouldBlock); // FIXME: wrong error type
        }
        let n = if nmax != 0 { std::cmp::min((*guard).len(), nmax) } else { (*guard).len() };
        Ok(StreamReader {
            stream: self.stream.clone(),
            guard: guard,
            nread: n,
        })
    }

    pub fn read_lock_wait(&self, nmin: usize, nmax: usize) -> StreamReader<'_, T> {
        let mut guard = self.stream.buf.lock().unwrap();
        assert!(nmin <= (*guard).capacity());
        while (*guard).len() < nmin {
            guard = self.stream.cv_read.wait(guard).unwrap();
        }
        let n = if nmax != 0 { std::cmp::min((*guard).len(), nmax) } else { (*guard).len() };
        StreamReader {
            stream: self.stream.clone(),
            guard: guard,
            nread: n,
        }
    }
}

pub struct WriteStream<T> {
    stream: Arc<Stream2<T>>,
}

impl<T: Copy> WriteStream<T> {
    pub fn new(nbuf: usize) -> (WriteStream<T>, ReadStream<T>) {
        let stream = Arc::new(Stream2::<T>::new(nbuf));
        (
            WriteStream {
                stream: stream.clone(),
            },
            ReadStream {
                stream: stream.clone(),
            },
        )
    }

    pub fn write_lock_try(&self, nmin: usize) -> Result<StreamWriter<'_, T>, TryLockError<MutexGuard<'_, CircularBuffer<T>>>> {
        let guard = self.stream.buf.try_lock()?;
        if (*guard).available() < nmin {
            return Err(std::sync::TryLockError::WouldBlock); // FIXME: wrong error type
        }
        Ok(StreamWriter {
            stream: self.stream.clone(),
            guard: guard,
        })
    }

    pub fn write_lock_wait(&self, nmin: usize) -> StreamWriter<'_, T> {
        let mut guard = self.stream.buf.lock().unwrap();
        assert!(nmin <= (*guard).capacity());
        while (*guard).available() < nmin {
            guard = self.stream.cv_write.wait(guard).unwrap();
        }
        StreamWriter {
            stream: self.stream.clone(),
            guard: guard,
        }
    }
}

#[test]
fn test_stream_small() {
    let (sw, sr) = WriteStream::<i32>::new(10);

    let t1 = std::thread::spawn(move || {
        let mut n = 0;
        while n < 500 {
            let mut writer = sw.write_lock_wait(3);
            writer.write(&[n, n + 1, n + 2]).unwrap();
            n += 3;
            writer.done();
        }
    });
    let t2 = std::thread::spawn(move || {
        let mut n = 0;
        while n < 500 {
            let mut reader = sr.read_lock_wait(1, 0);
            let (a, b) = reader.read();
            for x in a.iter() {
                assert_eq!(*x, n);
                n += 1;
            }
            for x in b.iter() {
                assert_eq!(*x, n);
                n += 1;
            }
            reader.done();
        }
    });
    t1.join().unwrap();
    t2.join().unwrap();
}

pub trait Stream3Reader<T> {
    type Guard<'a>: Stream3ReadGuard<'a, T> where Self: 'a;

    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn available(&self) -> usize;
    fn start_read(&mut self) -> Result<Self::Guard<'_>, DspError>;
}

pub trait Stream3ReadGuard<'a, T> {
    fn iter<'b>(&'b self) -> impl Iterator<Item = &'b [T]> where T: 'b;
    fn increment_read(&mut self, n: usize) -> Result<(), DspError>;
}

pub trait Stream3Writer<T> {
    type Guard<'a>: Stream3WriteGuard<'a, T> where Self: 'a;

    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn available(&self) -> usize;
    fn start_write(&mut self) -> Result<Self::Guard<'_>, DspError>;
}

pub trait Stream3WriteGuard<'a, T> {
    fn iter<'b>(&'b mut self) -> impl Iterator<Item = &'b mut [T]> where T: 'b;
    fn write(&mut self, data: &[T]) -> Result<(), DspError>;
    fn increment_write(&mut self, n: usize) -> Result<(), DspError>;
}
