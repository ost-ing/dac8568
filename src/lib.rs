//! # dac8568 library
//! A small library for using the dac8568.

#![allow(missing_docs)]
#![deny(warnings)]
#![no_std]
#![allow(dead_code)]

use embedded_hal::digital::v2::OutputPin;

pub enum Channel {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    NOMSG = 8,
    BROADCAST = 9,
}

impl Channel {
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

pub enum ControlType {
    WriteToInputRegister = 0,
    UpdateRegister = 1,
    WriteToChannelAndUpdateAllRegisters = 2,
    WriteToChannelAndUpdateSingleRegister = 3,
    PowerDownComm = 4,
    WriteToClearCodeRegister = 5,
    WriteToLDACRegister = 6,
    SoftwareReset = 7,
}

pub enum SetupMode {
    Static = 8,
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
    pub fn get_power_message(mode: PowerModes, channel: u8) -> Message {
        Message {
            prefix: 0,
            control: ControlType::PowerDownComm as u8,
            data: (mode as u16) | (channel as u16 >> 4),
            feature: channel,
            address: 0,
        }
    }

    pub fn get_enable_message(
        control: ControlType,
        data: InternalRefCommData,
        feature: InternalRefCommFeature,
    ) -> Message {
        Message {
            prefix: 0,
            control: control as u8,
            data: data as u16,
            feature: feature as u8,
            address: 0,
        }
    }

    pub fn get_write_message(channel: Channel, value: u16) -> Message {
        Message {
            prefix: 0,
            feature: 0,
            control: ControlType::WriteToChannelAndUpdateSingleRegister as u8,
            address: channel as u8,
            data: value,
        }
    }

    fn get_payload(&self) -> [u8; 4] {
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
pub struct Dac<NSS> {
    nss: NSS,
    active: bool,
}

/// DAC Related errors
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum DacError {
    /// Unable to write to bus
    BusWriteError,
}

impl<NSS> Dac<NSS>
where
    NSS: OutputPin,
{
    /// Initialize a new instance of dac8568
    pub fn new(nss: NSS) -> Self {
        Self { nss, active: false }
    }

    pub fn enable(&mut self) {
        self.active = true;
    }

    /// For asynchronous communication methods (e.g. Interrupt or DMA), this function
    /// prepares the DAC for the transfer, generates the command and passes it back to the initiator
    /// via the callback parameter
    pub fn prepare_transfer<F: FnMut([u8; 4]) -> ()>(&mut self, message: Message, mut callback: F) {
        if !self.active {
            return;
        }
        let command: [u8; 4] = message.get_payload();

        self.nss.set_low().unwrap_or_default();
        callback(command);
        self.nss.set_high().unwrap_or_default();
    }

    /// Write to the DAC via a blocking call on the specified SPI interface
    pub fn write_blocking(
        &mut self,
        spi: &mut dyn embedded_hal::blocking::spi::Write<u8, Error = ()>,
        message: Message,
    ) -> Result<(), DacError> {
        if !self.active {
            return Ok(());
        }
        let command: [u8; 4] = message.get_payload();

        self.nss.set_low().unwrap_or_default();
        let result = spi.write(&command);
        self.nss.set_high().unwrap_or_default();

        match result {
            Ok(v) => Ok(v),
            Err(_e) => Err(DacError::BusWriteError),
        }
    }
}
