use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub struct PinServer {
    listener: Option<TcpListener>,
    client: Option<TcpStream>,
}

impl PinServer {
    pub fn new() -> Self {
        Self {
            listener: None,
            client: None,
        }
    }

    pub fn start_server(&mut self, addr: &str) -> Result<(), String> {
        let listener = TcpListener::bind(addr).map_err(|e| format!("Failed to bind: {}", e))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set nonblocking: {}", e))?;
        self.listener = Some(listener);
        Ok(())
    }

    pub fn recive_data(&mut self) -> Option<(u8, u8)> {
        if self.client.is_none() {
            if let Some(ref listener) = self.listener {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream.set_nonblocking(true).ok();
                        self.client = Some(stream);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => {}
                }
            }
        }

        if let Some(ref mut stream) = self.client {
            let mut buf = [0u8; 2];
            match stream.read_exact(&mut buf) {
                Ok(_) => Some((buf[0], buf[1])),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
                Err(_) => {
                    self.client = None;
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn send_data(&mut self, port: u8, value: u8) -> Result<(), String> {
        if let Some(ref mut stream) = self.client {
            stream.write_all(&[port, value]).map_err(|e| format!("Send failed: {}", e))
        } else {
            Err("Not connected".to_string())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}
