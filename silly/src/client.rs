use dsp::volk_rs::{vec::AlignedVec, Complex};

pub struct WSClient {
    ws: tungstenite::protocol::WebSocket<std::net::TcpStream>,
    audio_buffer: AlignedVec<f32>,
}

impl WSClient {
    pub fn new(ws: tungstenite::protocol::WebSocket<std::net::TcpStream>) -> Self {
        WSClient {
            ws: ws,
            audio_buffer: AlignedVec::from_elem(0.0, 4800),
        }
    }

    pub fn serve(&mut self) {
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
            //reader_clone.lock().unwrap().read_samples(&mut self.audio_buffer).unwrap();
            let audiobytes = unsafe {
                let slice = std::slice::from_raw_parts(self.audio_buffer.as_ptr() as *const u8, self.audio_buffer.len() * std::mem::size_of::<f32>());
                tungstenite::Bytes::copy_from_slice(slice)
            };

            let mut send_bytes = Vec::with_capacity(1 + audiobytes.len());
            send_bytes.push(0x1u8); // type byte
            send_bytes.extend_from_slice(audiobytes.as_ref());
            self.ws.send(tungstenite::Message::Binary(send_bytes.into())).unwrap();
        }
    }
}
