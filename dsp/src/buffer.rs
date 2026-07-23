use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use volk_rs::vec::AlignedVec;
use crate::error::DspError;

pub struct CircularBuffer<T> {
    buf: AlignedVec<T>,
    head: usize, // read ptr
    tail: usize, // write ptr
    empty: bool,
}

impl<T: Copy> CircularBuffer<T> {
    pub fn new(n: usize) -> Self {
        CircularBuffer {
            buf: AlignedVec::new_zeroed(n),
            head: 0,
            tail: 0,
            empty: true,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        if self.empty {
            return 0;
        }
        if self.head == self.tail { // full
            return self.capacity();
        } else if self.head < self.tail { // writer not wrapped around
            return self.tail - self.head;
        } else { // writer wrapped around
            return (self.capacity() - self.head) + self.tail;
        }
    }

    pub fn available(&self) -> usize {
        return self.capacity() - self.len();
    }

    pub fn read(&self, nmax: usize) -> (&[T], &[T]) {
        let n = std::cmp::min(self.len(), nmax);
        if n == 0 {
            return (&[] as &[T], &[] as &[T]);
        }
        let n1 = std::cmp::min(n, self.capacity() - self.head);
        let s1 = &self.buf[self.head..self.head+n1];
        let s2 = if n1 != n { &self.buf[..n - n1] } else { &[] as &[T] };
        (s1, s2)
    }

    pub fn consume(&mut self, n: usize) -> Result<(), DspError> {
        if n > self.len() {
            return Err(DspError::new("not enough len to consume"));
        }
        self.pop(n);
        Ok(())
    }

    fn push(&mut self, n: usize) {
        if n > self.available() {
            panic!("tried to push over available");
        }
        self.empty = false;
        self.tail += n;
        self.tail %= self.capacity();
    }

    fn pop(&mut self, n: usize) {
        if n > self.len() {
            panic!("tried to pop over len");
        }
        if n == self.len() {
            self.empty = true;
        }
        self.head += n;
        self.head %= self.capacity();
    }

    pub fn write(&mut self, data: &[T]) -> Result<(), DspError> {
        if data.len() > self.available() {
            return Err(DspError::new("not enough space"));
        }

        let n_c1 = std::cmp::min(data.len(), self.capacity() - self.tail);
        self.buf[self.tail..self.tail+n_c1].copy_from_slice(&data[..n_c1]);
        if n_c1 != data.len() {
            let n_c2 = data.len() - n_c1;
            self.buf[..n_c2].copy_from_slice(&data[n_c1..n_c1+n_c2]);
        }
        self.push(data.len());
        Ok(())
    }
}

#[test]
fn test() -> Result<(), DspError> {
    let mut buf = CircularBuffer::<u32>::new(10);
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.available(), 10);

    buf.write(&[0, 1, 2, 3, 4])?;
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 5);
    assert_eq!(buf.available(), 5);

    buf.write(&[5])?;
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 6);
    assert_eq!(buf.available(), 4);

    buf.write(&[6, 7, 8, 9])?;
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 10);
    assert_eq!(buf.available(), 0);

    match buf.write(&[10, 11, 12]) {
        Ok(()) => { panic!("expected error"); },
        Err(_) => {},
    }

    let (r1, r2) = buf.read(100);
    assert_eq!(r1, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(r2, []);
    let (r1, r2) = buf.read(1);
    assert_eq!(r1, [0]);
    assert_eq!(r2, []);
    buf.consume(2)?;
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 8);
    assert_eq!(buf.available(), 2);

    let (r1, r2) = buf.read(6);
    assert_eq!(r1, [2, 3, 4, 5, 6, 7]);
    assert_eq!(r2, []);

    buf.write(&[10, 11])?;
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 10);
    assert_eq!(buf.available(), 0);

    let (r1, r2) = buf.read(100);
    assert_eq!(r1, [2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(r2, [10, 11]);
    buf.consume(9)?;
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 1);
    assert_eq!(buf.available(), 9);

    let (r1, r2) = buf.read(100);
    assert_eq!(r1, [11]);
    assert_eq!(r2, []);
    buf.consume(1)?;
    assert_eq!(buf.capacity(), 10);
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.available(), 10);

    Ok(())
}


struct LockfreeCircularBufferInner<T> {
    buf: AlignedVec<T>,
    // TODO: alledgedly aligning these to a cache line will improve performance somewhat
    head: AtomicUsize, // read ptr
    tail: AtomicUsize, // write ptr
}

impl<T: Copy> LockfreeCircularBufferInner<T> {
    // FIXME: BUG: full and empty cannot be distinguished.
    fn new(n: usize) -> LockfreeCircularBufferInner<T> {
        LockfreeCircularBufferInner {
            buf: AlignedVec::new_zeroed(n),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head == tail { // empty
            return 0;
        } else if head < tail { // writer not wrapped around
            return tail - head;
        } else { // writer wrapped around
            return ((self.capacity() * 2) - head) + tail;
        }
    }

    pub fn available(&self) -> usize {
        return self.capacity() - self.len();
    }

    fn push(&self, n: usize) {
        if n > self.available() {
            panic!("tried to push over available");
        }

        // relaxed is okay because only the writer ever writes the tail
        let tail = self.tail.load(Ordering::Relaxed);
        // Release so whatever we just wrote will be ordered before the next Acquire of this value
        self.tail.store((tail + n) % (self.capacity() * 2), Ordering::Release);
    }

    fn pop(&self, n: usize) {
        if n > self.len() {
            panic!("tried to pop over len");
        }

        // relaxed is okay because only the reader ever writes the head
        let head = self.head.load(Ordering::Relaxed);
        // Release so whatever we just wrote will be ordered before the next Acquire of this value
        self.head.store((head + n) % (self.capacity() * 2), Ordering::Release);
    }

    fn read_ranges(&self) -> (usize, usize, usize) {
        // relaxed is okay because only the reader ever writes the head
        let head = self.head.load(Ordering::Relaxed) % self.capacity();
        let n = self.len();
        let n1 = std::cmp::min(n, self.capacity() - head);
        let n2 = n - n1;
        (
            head,
            n1,
            n2,
        )
    }

    fn write_ranges(&self) -> (usize, usize, usize) {
        // relaxed is okay because only the writer ever writes the tail
        let tail = self.tail.load(Ordering::Relaxed) % self.capacity();
        let avl = self.available();
        let n1 = std::cmp::min(avl, self.capacity() - tail);
        let n2 = avl - n1;
        (
            tail,
            n1,
            n2,
        )
    }
}

pub struct LockfreeCircularBufferReader<T> {
    inner: Arc<LockfreeCircularBufferInner<T>>,
    active: bool,
}

unsafe impl<T: Send> Send for LockfreeCircularBufferReader<T> {}

impl<T: Copy> LockfreeCircularBufferReader<T> {
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn available(&self) -> usize {
        self.inner.available()
    }

    pub fn read(&mut self) -> Result<LockfreeCircularBufferReadGuard<'_, T>, DspError> {
        if self.active {
            return Err(DspError::new("only one reader may be active at a given time"));
        }
        self.active = true;
        Ok(LockfreeCircularBufferReadGuard::new(self))
    }
}

pub struct LockfreeCircularBufferReadGuard<'a, T: Copy> {
    reader: &'a mut LockfreeCircularBufferReader<T>,
    n_read: usize,
    read_start: usize,
    read_b1_max: usize,
    read_b2_max: usize,
}

impl<'a, T: Copy> LockfreeCircularBufferReadGuard<'a, T> {
    fn new(reader: &'a mut LockfreeCircularBufferReader<T>) -> LockfreeCircularBufferReadGuard<'a, T> {
        let (head, read_b1_max, read_b2_max) = reader.inner.read_ranges();

        LockfreeCircularBufferReadGuard {
            reader: reader,
            n_read: 0,
            read_start: head,
            read_b1_max: read_b1_max,
            read_b2_max: read_b2_max,
        }
    }

    pub fn as_slices<'b>(&'b self) -> (&'b [T], &'b [T]) {
        (
            &self.reader.inner.buf[self.read_start..self.read_start+self.read_b1_max],
            &self.reader.inner.buf[0..self.read_b2_max],
        )
    }

    pub fn increment_read(&mut self, n: usize) -> Result<(), DspError> {
        if self.n_read + n > self.read_b1_max + self.read_b2_max {
            return Err(DspError::new("not enough len to read"));
        }
        self.n_read += n;
        Ok(())
    }
}

impl<'a, T: Copy> Drop for LockfreeCircularBufferReadGuard<'a, T> {
    fn drop(&mut self) {
        if self.n_read > 0 {
            self.reader.inner.pop(self.n_read);
        }
        self.reader.active = false;
    }
}

pub struct LockfreeCircularBufferWriter<T> {
    inner: Arc<LockfreeCircularBufferInner<T>>,
    active: bool,
}

unsafe impl<T: Send> Send for LockfreeCircularBufferWriter<T> {}

impl<T: Copy> LockfreeCircularBufferWriter<T> {
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn available(&self) -> usize {
        self.inner.available()
    }

    pub fn write(&mut self) -> Result<LockfreeCircularBufferWriteGuard<'_, T>, DspError> {
        if self.active {
            return Err(DspError::new("only one writer may be active at a given time"));
        }
        self.active = true;
        Ok(LockfreeCircularBufferWriteGuard::new(self))
    }
}

pub struct LockfreeCircularBufferWriteGuard<'a, T: Copy> {
    writer: &'a mut LockfreeCircularBufferWriter<T>,
    n_written: usize,
    write_start: usize,
    write_b1_max: usize,
    write_b2_max: usize,
}

impl<'a, T: Copy> LockfreeCircularBufferWriteGuard<'a, T> {
    fn new(writer: &'a mut LockfreeCircularBufferWriter<T>) -> LockfreeCircularBufferWriteGuard<'a, T> {
        let (tail, write_b1_max, write_b2_max) = writer.inner.write_ranges();
        LockfreeCircularBufferWriteGuard {
            writer: writer,
            n_written: 0,
            write_start: tail,
            write_b1_max: write_b1_max,
            write_b2_max: write_b2_max,
        }
    }

    pub fn as_mut_slices<'b>(&'b mut self) -> (&'b mut [T], &'b mut [T]) {
        // this unsafe should be fine:tm:
        unsafe {
            (
                std::slice::from_raw_parts_mut((self.writer.inner.buf.as_ptr() as *mut T).add(self.write_start), self.write_b1_max),
                std::slice::from_raw_parts_mut(self.writer.inner.buf.as_ptr() as *mut T, self.write_b2_max),
            )
        }
    }

    pub fn write(&mut self, data: &[T]) -> Result<(), DspError> {
        if data.len() > self.write_b1_max + self.write_b2_max {
            return Err(DspError::new("not enough space"));
        }

        let (s1, s2) = self.as_mut_slices();
        let n_s1 = std::cmp::min(data.len(), s1.len());
        s1[..n_s1].copy_from_slice(&data[..n_s1]);
        if n_s1 != data.len() {
            let n_s2 = data.len() - n_s1;
            s2[..n_s2].copy_from_slice(&data[n_s1..n_s1+n_s2]);
        }
        self.increment_write(data.len())?;
        Ok(())
    }

    pub fn increment_write(&mut self, n: usize) -> Result<(), DspError> {
        if self.n_written + n > self.write_b1_max + self.write_b2_max {
            return Err(DspError::new("not enough available to write"));
        }
        self.n_written += n;
        Ok(())
    }
}

impl<'a, T: Copy> Drop for LockfreeCircularBufferWriteGuard<'a, T> {
    fn drop(&mut self) {
        if self.n_written > 0 {
            self.writer.inner.push(self.n_written);
        }
        self.writer.active = false;
    }
}

pub fn create_lockfree_circular_buffer<T: Copy>(n: usize) -> (LockfreeCircularBufferReader<T>, LockfreeCircularBufferWriter<T>) {
    let inner: Arc<LockfreeCircularBufferInner<T>> = Arc::new(LockfreeCircularBufferInner::new(n));
    (
        LockfreeCircularBufferReader {
            inner: inner.clone(),
            active: false,
        },
        LockfreeCircularBufferWriter {
            inner: inner.clone(),
            active: false,
        }
    )
}


#[test]
fn test_lockfree_circularbuffer() -> Result<(), DspError> {
    let (mut r, mut w) = create_lockfree_circular_buffer::<u32>(10);
    assert_eq!(r.capacity(), 10);
    assert_eq!(r.len(), 0);
    assert_eq!(r.available(), 10);

    let mut wr = w.write()?;
    wr.write(&[0, 1, 2, 3, 4])?;
    drop(wr);
    assert_eq!(w.capacity(), 10);
    assert_eq!(w.len(), 5);
    assert_eq!(w.available(), 5);

    let mut wr = w.write()?;
    wr.write(&[5])?;
    drop(wr);
    assert_eq!(r.capacity(), 10);
    assert_eq!(r.len(), 6);
    assert_eq!(r.available(), 4);

    let mut wr = w.write()?;
    wr.write(&[6, 7, 8, 9])?;
    drop(wr);
    assert_eq!(w.capacity(), 10);
    assert_eq!(w.len(), 10);
    assert_eq!(w.available(), 0);

    let mut wr = w.write()?;
    match wr.write(&[10, 11, 12]) {
        Ok(()) => { panic!("expected error"); },
        Err(_) => {},
    }
    drop(wr);

    let mut rr = r.read()?;
    let (r1, r2) = rr.as_slices();
    assert_eq!(r1, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(r2, []);
    let (r1, r2) = rr.as_slices();
    assert_eq!(r1, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(r2, []);
    rr.increment_read(2)?;
    drop(rr);
    assert_eq!(r.capacity(), 10);
    assert_eq!(r.len(), 8);
    assert_eq!(r.available(), 2);

    let rr = r.read()?;
    let (r1, r2) = rr.as_slices();
    assert_eq!(r1, [2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(r2, []);
    drop(rr);

    let mut wr = w.write()?;
    wr.write(&[10, 11])?;
    drop(wr);
    assert_eq!(w.capacity(), 10);
    assert_eq!(w.len(), 10);
    assert_eq!(w.available(), 0);

    let mut rr = r.read()?;
    let (r1, r2) = rr.as_slices();
    assert_eq!(r1, [2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(r2, [10, 11]);
    rr.increment_read(9)?;
    drop(rr);
    assert_eq!(r.capacity(), 10);
    assert_eq!(r.len(), 1);
    assert_eq!(r.available(), 9);

    let mut rr = r.read()?;
    let (r1, r2) = rr.as_slices();
    assert_eq!(r1, [11]);
    assert_eq!(r2, []);
    rr.increment_read(1)?;
    drop(rr);
    assert_eq!(w.capacity(), 10);
    assert_eq!(w.len(), 0);
    assert_eq!(w.available(), 10);

    Ok(())
}

#[test]
fn test_lockfree_circularbuffer_multithread() {
    let n_run = 2^16;
    let (mut reader, mut writer) = create_lockfree_circular_buffer::<u32>(15);

    let t1 = std::thread::spawn(move || {
        let mut n = 0;
        while n < n_run {
            if writer.available() < 3 {
                std::thread::yield_now();
                continue;
            }
            let mut w = writer.write().unwrap();
            w.write(&[n, n + 1, n + 2]).unwrap();
            n += 3;
            drop(w);
        }
    });
    let t2 = std::thread::spawn(move || {
        let mut n = 0;
        while n < n_run {
            if reader.len() < 1 {
                std::thread::yield_now();
                continue;
            }
            let mut r = reader.read().unwrap();
            let (a, b) = r.as_slices();
            for x in a.iter() {
                assert_eq!(*x, n);
                n += 1;
            }
            for x in b.iter() {
                assert_eq!(*x, n);
                n += 1;
            }
            r.increment_read(a.len() + b.len()).unwrap();
            drop(r);
        }
    });
    t1.join().unwrap();
    t2.join().unwrap();
}
