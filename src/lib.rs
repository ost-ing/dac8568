//! # dac8568 library
//! A small library for using the TI DAC8568, DAC7568 and DAC8168

#![deny(warnings)]
#![no_std]

use embedded_hal::digital::v2::OutputPin;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
/// The broadcast channel
pub enum Channel {
    /// Channel A
    A = 0,
    /// Channel B
    B = 1,
    /// Channel C
    C = 2,
    /// Channel D
    D = 3,
    /// Channel E
    E = 4,
    /// Channel F
    F = 5,
    /// Channel G
    G = 6,
    /// Channel H
    H = 7,
    /// No message
    NOMSG = 8,
    /// Broadcast (all channels)
    BROADCAST = 9,
}

impl Channel {
    /// Get Channel from an index
    pub fn from_index(index: u8) -> Channel {
        match index {
            0 => Channel::A,
            1 => Channel::B,
            2 => Channel::C,
            3 => Channel::D,
            4 => Channel::E,
            5 => Channel::F,
            6 => Channel::G,
            7 => Channel::H,
            _ => panic!("Unsupported index for dac8568 channel select"),
        }
    }
}

/// The message control type
pub enum ControlType {
    /// Write to input register [Untested]
    WriteToInputRegister = 0,
    /// Update register [Untested]
    UpdateRegister = 1,
    /// Write to channel and update all registers [Untested]
    WriteToChannelAndUpdateAllRegisters = 2,
    /// Write to channel and update single register
    WriteToChannelAndUpdateSingleRegister = 3,
    /// Power down [Untested]
    PowerDownComm = 4,
    /// Write to clear code register [Untested]
    WriteToClearCodeRegister = 5,
    /// Write to LDAC register [Untested]
    WriteToLDACRegister = 6,
    /// Software reset [Untested]
    SoftwareReset = 7,
}

///

/// The Message that is eventually serialized and transmitted to the DAC
/// The input shift register (SR) of the DAC7568, DAC8168, and DAC8568
/// is 32 bits wide, and consists of four Prefix bits (DB31 to DB28),
/// four control bits (DB27 to DB24), 16 databits (DB23 to DB4),
/// and four additional feature bits. The 16 databits comprise the 16-, 14-, or 12-bit input code
#[derive(Copy, Clone)]
pub struct Message {
    prefix: u8,  // 4 bits
    control: u8, // 4 bits
    address: u8, // 4 bits
    data: u16,   // 16 bits
    feature: u8, // 4 bits
}

impl Message {
    /// Get internal reference message
    /// Used for switching DAC8568 from its default state using an external reference
    /// To using its internal 2.5v reference
    pub fn get_internal_reference_message(internal: bool) -> Message {
        Message {
            prefix: 0x00,
            control: 0x08,
            address: 0x00,
            data: 0x0000,
            feature: if internal { 0x01 } else { 0x00 },
        }
    }

    /// Get voltage message, which will update a channel with a given value
    pub fn get_voltage_message(channel: Channel, value: u16, is_inverted: bool) -> Message {
        let output = if is_inverted { u16::MAX - value } else { value };

        Message {
            prefix: 0,
            control: ControlType::WriteToChannelAndUpdateSingleRegister as u8,
            address: channel as u8,
            data: output,
            feature: 0,
        }
    }

    /// Get software reset message
    /// 8.2.10 Software Reset Function
    /// The DAC7568, DAC8168, and DAC8568 contain a software reset feature.
    /// If the software reset feature is executed, all registers inside the device are reset to default settings; that is,
    /// all DAC channels are reset to the power-on reset code (power on reset to zero scale for grades A and C; power on reset to midscale for grades B and D).
    pub fn get_software_reset_message() -> Message {
        Message {
            prefix: 0b0001,
            control: 0b1100,
            address: 0x00,
            data: 0x00,
            feature: 0x00,
        }
    }

    /// Get the message payload word
    pub fn get_payload_word(&self) -> u32 {
        let mut payload: u32 = 0x00;
        payload |= (self.prefix as u32) << 28;
        payload |= (self.control as u32) << 24;
        payload |= (self.address as u32) << 20;
        payload |= (self.data as u32) << 4;
        payload |= self.feature as u32;
        payload
    }

    /// Get the message payload
    pub fn get_payload_bytes(&self) -> [u8; 4] {
        self.get_payload_word().to_be_bytes()
    }
}

/// DAC8568
pub struct Dac<SPI, SYNC> {
    /// The SPI interface
    spi: SPI,
    /// The SPI's sync (select) line
    sync: SYNC,
    /// If the output of the DAC is inverted.
    /// Useful if the hardware engineer has designed an inverting gain stage after the DAC output
    is_inverted: bool,
}

/// DAC Related errors
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum DacError {
    /// Unable to write to bus
    BusWriteError,
}

impl<SPI, SYNC> Dac<SPI, SYNC>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
    SYNC: OutputPin,
{
    /// Initialize a new instance of dac8568
    pub fn new(spi: SPI, sync: SYNC) -> Self {
        Self {
            spi,
            sync,
            is_inverted: false,
        }
    }

    /// Consume the dac and return the underlying SPI and GPIO pins used by it
    pub fn release(self) -> (SPI, SYNC) {
        (self.spi, self.sync)
    }

    /// Sets the output signal of the DAC to be inverted or non-inverted (default)
    /// Useful if the hardware engineer has designed an inverting gain stage after the DAC output
    pub fn set_inverted_output(&mut self, state: bool) {
        self.is_inverted = state;
    }

    /// Set the specified value to the given channel. This will update the DAC
    /// to output the desired voltage
    pub fn set_voltage(&mut self, channel: Channel, voltage: u16) -> Result<(), DacError> {
        let message = Message::get_voltage_message(channel, voltage, self.is_inverted);
        self.write(message)
    }

    /// Configure the DAC to use its internal reference mode of 2.5v rather than using an external
    /// voltage reference
    pub fn use_internal_reference(&mut self) -> Result<(), DacError> {
        let message = Message::get_internal_reference_message(true);
        self.write(message)
    }

    /// Configure the DAC to use its external reference mode rather than using the internal reference
    pub fn use_external_reference(&mut self) -> Result<(), DacError> {
        let message = Message::get_internal_reference_message(false);
        self.write(message)
    }

    /// Perform a software reset, clearing out all registers
    /// 8.2.10 Software Reset Function
    /// The DAC7568, DAC8168, and DAC8568 contain a software reset feature.
    /// If the software reset feature is executed, all registers inside the device are reset to default settings; that is,
    /// all DAC channels are reset to the power-on reset code (power on reset to zero scale for grades A and C; power on reset to midscale for grades B and D).
    pub fn reset(&mut self) -> Result<(), DacError> {
        let message = Message::get_software_reset_message();
        self.write(message)
    }

    /// Write to the DAC via a blocking call on the specified SPI interface
    fn write(&mut self, message: Message) -> Result<(), DacError> {
        let command: [u8; 4] = message.get_payload_bytes();

        self.sync.set_low().unwrap_or_default();
        let result = self.spi.write(&command);
        self.sync.set_high().unwrap_or_default();

        match result {
            Ok(v) => Ok(v),
            Err(_e) => Err(DacError::BusWriteError),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn inverts_signal() {
        let message = super::Message::get_voltage_message(super::Channel::A, 0, false);
        assert_eq!(message.data, 0);

        let message = super::Message::get_voltage_message(super::Channel::A, 0, true);
        assert_eq!(message.data, u16::MAX);
    }
}
