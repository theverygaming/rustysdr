use std::io::{Read, Write};
use std::net::TcpStream;
use dsp::volk_rs::Complex;
use dsp::volk_rs::vec::AlignedVec;


pub struct RtlTcpClient {
    stream: TcpStream,
}

impl RtlTcpClient {
    pub fn new(conn: &str) -> Self {
        let mut stream = TcpStream::connect(conn).unwrap();
        RtlTcpClient {
            stream: stream,
        }
    }

    pub fn read(&mut self, output: &mut [Complex<f32>]) {
        let mut buf = AlignedVec::new_zeroed(output.len()*2);
        self.stream.read_exact(&mut buf).unwrap();

        let scalar = 1.0 / 127.5;
        let mut n: usize = 0;
        for e in output.iter_mut() {
            *e = Complex { re: (buf[n] as f32 * scalar) - 1.0, im: (buf[n+1] as f32 * scalar) - 1.0 };
            n += 2;
        }
    }

    fn send_cmd(&mut self, cmd: u8, arg: u32) {
        let mut buf: [u8; 5] = [0; 5];
        buf[0] = cmd;
        buf[1..5].copy_from_slice(&arg.to_be_bytes());
        self.stream.write(&buf).unwrap();
    }

    pub fn set_freq(&mut self, freq: u32) {
        self.send_cmd(0x01, freq);
    }

    pub fn set_sr(&mut self, sr: u32) {
        self.send_cmd(0x02, sr);
    }
}
