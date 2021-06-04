//! # dac8568 library
//! A small library for using the dac8568.

#![allow(missing_docs)]
#![deny(warnings)]
#![no_std]
#![allow(dead_code)]

use embedded_hal::digital::v2::OutputPin;

#[derive(PartialEq, Copy, Clone)]
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
    /// Power down
    PowerDownComm = 4,
    /// Write to clear code register [Untested]
    WriteToClearCodeRegister = 5,
    /// Write to LDAC register [Untested]
    WriteToLDACRegister = 6,
    /// Software reset [Untested]
    SoftwareReset = 7,
}

/// Setup Mode. Currently this library only been tested
/// Using the default Static mode
pub enum SetupMode {
    /// Static Mode
    Static = 8,
    /// Flex Mode [Untested]
    Flex = 9,
}

pub enum ClearCodeFeature {
    ClearToZeroScale = 0,
    ClearToMidScale = 1,
    ClearToFullScale = 2,
    IgnoreClearPin = 3,
}

pub enum InternalRefCommFeature {
    PowerDownIntRefStatic = 0,
    PowerUpIntRefStatic = 1,
    //   PowerUpIntRefFlex = 0,
    //   PowerUpIntRefAlwaysFlex = 0,
    //   PowerDownIntRefFlex = 0,
    //   SwitchFromFlexToStatic = 0
}

/// TODO
pub enum Register {
    A = 1,
    B = 2,
    C = 4,
    D = 8,
    E = 16,
    F = 32,
    G = 64,
    H = 128,
}

/// TODO
pub enum PowerModes {
    PowerUp = 0,
    PowerDown1KToGround = 16,
    PowerDown100KToGround = 32,
    PowerDownHighZToGround = 48,
}

/// TODO
pub enum InternalRefCommData {
    Default = 0,
    PowerUpIntRefFlex = 32768,
    PowerUpIntRefAlwaysFlex = 40960,
    PowerDownIntRefFlex = 49152,
}

///

/// The Message that is eventually serialized and transmitted to the DAC
/// The inputshiftregister (SR) of the DAC7568,DAC8168,and DAC8568
/// is 32 bits wide(as shown in Table1, Table2, and Table3, respectively),
/// and consists of four Prefix bits (DB31 to DB28),
/// four control bits (DB27 to DB24), 16 databits (DB23 to DB4),
/// and four additional feature bits. The 16 databits comprise the 16-, 14-, or 12-bit input code
pub struct Message {
    prefix: u8,  // 4 bits
    control: u8, // 4 bits
    address: u8, // 4 bits
    data: u16,   // 16 bits
    feature: u8, // 4 bits
}

impl Message {
    /// Get internal power message [Untested]
    pub fn get_power_internal_message() -> Message {
        Message {
            prefix: 0x00,
            control: 0x08,
            address: 0x00,
            data: 0x0000,
            feature: 0x01,
        }
    }

    /// Get enable message [Untested]
    pub fn get_enable_message(
        control: ControlType,
        data: InternalRefCommData,
        feature: InternalRefCommFeature,
    ) -> Message {
        Message {
            prefix: 0,
            control: control as u8,
            address: 0,
            data: data as u16,
            feature: feature as u8,
        }
    }

    /// Get write message, which will update a channel with a given value
    pub fn get_write_message(channel: Channel, value: u16) -> Message {
        Message {
            prefix: 0,
            control: ControlType::WriteToChannelAndUpdateSingleRegister as u8,
            address: channel as u8,
            data: value,
            feature: 0,
        }
    }

    /// Get the message payload
    pub fn get_payload(&self) -> [u8; 4] {
        let mut payload: u32 = 0x00;
        payload = payload | ((self.prefix as u32) << 28);
        payload = payload | ((self.control as u32) << 24);
        payload = payload | ((self.address as u32) << 20);
        payload = payload | ((self.data as u32) << 4);
        payload = payload | ((self.feature as u32) << 0);
        payload.to_be_bytes()
    }
}

/// DAC8568
pub struct Dac<SPI, SYNC> {
    spi: SPI,
    sync: SYNC,
}

/// DAC Related errors
#[derive(Clone, Debug)]
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
        Self { spi, sync }
    }

    /// Consume the dac and return the underlying SPI and GPIO pins used by it
    pub fn release(self) -> (SPI, SYNC) {
        (self.spi, self.sync)
    }

    /// Write to the DAC via a blocking call on the specified SPI interface
    pub fn write(&mut self, message: Message) -> Result<(), DacError> {
        let command: [u8; 4] = message.get_payload();

        self.sync.set_low().unwrap_or_default();
        let result = self.spi.write(&command);
        self.sync.set_high().unwrap_or_default();

        match result {
            Ok(v) => Ok(v),
            Err(_e) => Err(DacError::BusWriteError),
        }
    }
}
