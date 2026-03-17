use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PinGPIO {
    gpio: Gpio,
    output_pins: HashMap<u8, OutputPin>, // AVR pin number -> GPIO output
    input_pins: HashMap<u8, InputPin>,   // AVR pin number -> GPIO input
    pin_mapping: HashMap<u8, u8>,        // AVR pin → RPi GPIO pin number
}

impl PinGPIO {
    pub fn new() -> Result<Self, String> {
        let gpio = Gpio::new().map_err(|e| format!("Failed to init GPIO: {}", e))?;

        Ok(Self {
            gpio,
            output_pins: HashMap::new(),
            input_pins: HashMap::new(),
            pin_mapping: Self::default_mapping(),
        })
    }

    /// Map Port A (bits 0-7) to RPi GPIO pins
    /// PA0 -> GPIO17, PA1 -> GPIO27, PA2 -> GPIO22, PA3 -> GPIO23, PA4 -> GPIO24, PA5 -> GPIO25, PA6 -> GPIO5, PA7 -> GPIO6
    fn default_mapping() -> HashMap<u8, u8> {
        let mut map = HashMap::new();
        let rpi_pins = [17, 27, 22, 23, 24, 25, 5, 6];
        for (avr_bit, &rpi_pin) in rpi_pins.iter().enumerate() {
            map.insert(avr_bit as u8, rpi_pin);
        }
        map
    }

    pub fn set_pin_direction(&mut self, avr_pin: u8, is_output: bool) -> Result<(), String> {
        // If pin mapping does not exist, skip
        let rpi_pin = match self.pin_mapping.get(&avr_pin) {
            Some(&pin) => pin,
            None => return Ok(()),
        };

        // Remove from current configuration
        self.output_pins.remove(&avr_pin);
        self.input_pins.remove(&avr_pin);

        // Configure as input or output
        if is_output {
            let pin = self
                .gpio
                .get(rpi_pin)
                .map_err(|e| format!("GPIO{} error: {}", rpi_pin, e))?
                .into_output();
            self.output_pins.insert(avr_pin, pin);
        } else {
            let pin = self
                .gpio
                .get(rpi_pin)
                .map_err(|e| format!("GPIO{} error: {}", rpi_pin, e))?
                .into_input();
            self.input_pins.insert(avr_pin, pin);
        }

        Ok(())
    }

    pub fn write_pin(&mut self, avr_pin: u8, high: bool) -> Result<(), String> {
        if let Some(pin) = self.output_pins.get_mut(&avr_pin) {
            let level = if high { Level::High } else { Level::Low };
            pin.write(level);
        }
        Ok(())
    }

    pub fn read_pin(&self, avr_pin: u8) -> Option<bool> {
        self.input_pins.get(&avr_pin).map(|pin| pin.is_high())
    }
}
