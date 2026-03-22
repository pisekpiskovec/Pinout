use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use Wire::{
    Message, CMD_REQUEST, CMD_RESET, CMD_RESPONSE, CMD_WRITE, PORT_A_ADDR, PORT_B_ADDR,
    PORT_C_ADDR, PORT_D_ADDR, PROTOCOL_VERSION,
};

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

    #[allow(clippy::collapsible_if)]
    pub fn recive_data(&mut self, pin_states: &[u8; 4]) -> Option<(u8, u8)> {
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
            loop {
                let mut buf = [0u8; 4];
                match stream.read_exact(&mut buf) {
                    Ok(_) => {
                        let data = Message::from_bytes(buf);

                        // Protocol version check
                        if data.version != Wire::PROTOCOL_VERSION {
                            continue;
                        }

                        // Command parsing
                        match data.command {
                            CMD_WRITE => return Some((data.address, data.value)),
                            CMD_REQUEST => {
                                let value = match data.address {
                                    PORT_A_ADDR => pin_states[0],
                                    PORT_B_ADDR => pin_states[1],
                                    PORT_C_ADDR => pin_states[2],
                                    PORT_D_ADDR => pin_states[3],
                                    _ => 0,
                                };

                                // Send response immediately
                                let response = Message {
                                    version: PROTOCOL_VERSION,
                                    command: CMD_RESPONSE,
                                    address: data.address,
                                    value,
                                }
                                .to_bytes();
                                if stream.write_all(&response).is_ok() {
                                    stream.flush().ok();
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => return None,
                    Err(_) => {
                        self.client = None;
                        return None;
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn send_data(&mut self, port: u8, value: u8) -> Result<(), String> {
        if let Some(ref mut stream) = self.client {
            let data = Message {
                version: PROTOCOL_VERSION,
                command: if port == 0xFF { CMD_RESET } else { CMD_WRITE },
                address: port,
                value,
            }
            .to_bytes();
            stream
                .write_all(&data)
                .map_err(|e| format!("Send failed: {}", e))
        } else {
            Err("Not connected".to_string())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}
