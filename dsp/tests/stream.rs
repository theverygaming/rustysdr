use std::thread;
use dsp::stream::Stream;

fn get_next_num(counter: &mut usize) -> usize {
    *counter += 1;
    return *counter
}

#[test]
fn test_stream() {
    let stream = Stream::<usize>::new(5000);

    let stream_read = stream.clone();
    let reader_thread = thread::spawn(move || {
        let mut counter: usize = 0;
        for it in 0..1000 {
            let n_read = stream_read.read().unwrap();
            let buf = stream_read.buf_read.lock().unwrap();
            for i in 0..buf.len() {
                let exp = get_next_num(&mut counter);
                assert!(buf[i] == exp, "buffer value read invalid v: {} exp: {} i: {} it: {}", buf[i], exp, i, it);
            }
            drop(buf);
            let exp = get_next_num(&mut counter);
            assert!(n_read == exp, "n_read invalid n_read: {} exp: {} it: {}", n_read, exp, it);
            stream_read.flush();
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

    reader_thread.join().unwrap();
    writer_thread.join().unwrap();
}
