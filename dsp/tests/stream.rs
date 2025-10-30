use dsp::stream::Stream;
use std::thread;

fn get_next_num(counter: &mut usize) -> usize {
    *counter += 1;
    return *counter;
}

#[test]
fn test_stream() {
    let stream = Stream::<usize>::new(5000);

    let stream_read1 = stream.clone();
    let reader_id1 = stream_read1.start_reader();
    let reader_thread1 = thread::spawn(move || {
        let mut counter: usize = 0;
        for it in 0..1000 {
            let n_read = match stream_read1.read(reader_id1) {
                Some(x) => x,
                None => {
                    panic!("premature reader stop");
                }
            };
            let buf = stream_read1.buf_read.read().unwrap();
            for i in 0..buf.len() {
                let exp = get_next_num(&mut counter);
                assert!(buf[i] == exp, "buffer value read invalid v: {} exp: {} i: {} it: {}", buf[i], exp, i, it);
            }
            drop(buf);
            let exp = get_next_num(&mut counter);
            assert!(n_read == exp, "n_read invalid n_read: {} exp: {} it: {}", n_read, exp, it);
            stream_read1.flush(reader_id1);
        }
    });

    let stream_read2 = stream.clone();
    let reader_id2 = stream_read2.start_reader();
    let reader_thread2 = thread::spawn(move || {
        let mut counter: usize = 0;
        for it in 0..1000 {
            let n_read = match stream_read2.read(reader_id2) {
                Some(x) => x,
                None => {
                    panic!("premature reader stop");
                }
            };
            let buf = stream_read2.buf_read.read().unwrap();
            for i in 0..buf.len() {
                let exp = get_next_num(&mut counter);
                assert!(buf[i] == exp, "buffer value read invalid v: {} exp: {} i: {} it: {}", buf[i], exp, i, it);
            }
            drop(buf);
            let exp = get_next_num(&mut counter);
            assert!(n_read == exp, "n_read invalid n_read: {} exp: {} it: {}", n_read, exp, it);
            stream_read2.flush(reader_id2);
        }
    });

    let stream_write = stream.clone();
    let writer_thread = thread::spawn(move || {
        let mut counter: usize = 0;
        for _ in 0..1000 {
            let mut buf = stream_write.buf_write.lock().unwrap();
            for i in 0..buf.len() {
                buf[i] = get_next_num(&mut counter);
            }
            drop(buf);
            stream_write.swap(get_next_num(&mut counter));
        }
    });

    reader_thread1.join().unwrap();
    reader_thread2.join().unwrap();
    // when reading is done we should be able to stop the reader
    stream.clone().stop_reader(reader_id1);
    stream.clone().stop_reader(reader_id2);
    writer_thread.join().unwrap();
}

// FIXME: stream start & stop tests
