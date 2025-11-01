use dsp::stream::Stream;
use std::sync::{Arc};

pub struct WSClient {
    ws: tungstenite::protocol::WebSocket<std::net::TcpStream>,
    stream: Arc<Stream::<f32>>,
}

impl WSClient {
    pub fn new(ws: tungstenite::protocol::WebSocket<std::net::TcpStream>, stream: Arc<Stream::<f32>>) -> Self {
        WSClient {
            ws: ws,
            stream: stream,
        }
    }

    pub fn serve(&mut self) {
        let reader_id = self.stream.start_reader();
        loop {
            let msg = match self.ws.read() {
                Ok(v) => v,
                Err(tungstenite::error::Error::ConnectionClosed) => {
                    println!("closing WS");
                    return;
                }
                Err(e) => panic!("WS error: {:?}", e),
            };
            // We do not want to send back ping/pong messages.
            //if msg.is_binary() || msg.is_text() {
            //    self.ws.send(msg).unwrap();
            //}
            let n_read = self.stream.read(reader_id).expect("reader should not stop lmao");
            let audio_buffer = self.stream.buf_read.read().unwrap();
            let audiobytes = unsafe {
                let slice = std::slice::from_raw_parts(audio_buffer.as_ptr() as *const u8, n_read * std::mem::size_of::<f32>());
                tungstenite::Bytes::copy_from_slice(slice)
            };
            self.stream.flush(reader_id);

            let mut send_bytes = Vec::with_capacity(1 + audiobytes.len());
            send_bytes.push(0x1u8); // type byte
            send_bytes.extend_from_slice(audiobytes.as_ref());
            self.ws.send(tungstenite::Message::Binary(send_bytes.into())).unwrap();
        }
    }
}
