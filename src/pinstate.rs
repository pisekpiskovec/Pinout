use std::collections::HashMap;

#[derive(Debug)]
pub struct PinState {
    pins: HashMap<u8, u8>,
}

impl PinState {
    pub fn new() -> Self {
        let mut pins = HashMap::new();
        pins.insert(0x39, 0); // PinA
        pins.insert(0x36, 0); // PinB
        pins.insert(0x33, 0); // PinC
        pins.insert(0x30, 0); // PinD

        Self { pins }
    }

    pub fn update_port(&mut self, address: u8, value: u8) {
        self.pins.insert(address, value);
    }

    pub fn get_port(&self, address: u8) -> Option<u8> {
        self.pins.get(&address).copied()
    }

    pub fn get_pin(&self, address: u8, bit: u8) -> bool {
        if let Some(port_value) = self.pins.get(&address) {
            (port_value >> bit) & 1 == 1
        } else {
            false
        }
    }
}
