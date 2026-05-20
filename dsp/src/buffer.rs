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
