//! # dac8568 library
//! A small library for using the dac8568.

#![allow(missing_docs)]
#![deny(warnings)]
#![no_std]
#![allow(dead_code)]

use embedded_hal::digital::v2::OutputPin;

pub enum ChannelSelect {
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
  Flex = 9
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
  H = 128
}

/// TODO
pub enum PowerModes {
  PowerUp = 0,
  PowerDown1KToGround = 16,
  PowerDown100KToGround = 32,
  PowerDownHighZToGround = 48
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
pub struct Message {
    feature: u8, // 4 bits
    /// Todo, only DAC8568 is supported. DAC7568 = 12, DAC8168 = 14
    data: u16,  // data
    address: u8, // 4 bits
    control: u8, // 4 bits
    prefix: u8, // 4 bits
}

impl Message {
    fn get_payload(&self) -> [u8; 4] {
        [self.prefix, self.control, self.address, (self.data << 8) as u8, (self.data << 0) as u8, self.feature]
    }
}

/// DAC8568
pub struct Dac<NSS, LDAC, CLR> {
    nss: NSS,
    ldac: LDAC,
    clear: CLR,
    active: bool,
}

/// DAC Related errors
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum DacError {
    /// Unable to write to bus
    BusWriteError,
}


impl<NSS, LDAC, CLR> Dac<NSS, LDAC, CLR>
where
    NSS: OutputPin,
    LDAC: OutputPin,
    CLR: OutputPin,
{
    /// Initialize a new instance of dac8568
    pub fn new(nss: NSS, ldac: LDAC, clear: CLR) -> Self {
        Self {
            nss,
            ldac,
            clear,
            active: false,
        }
    }

    pub fn get_power_message(mode: PowerModes, channel: u8) -> Message {
        Message {
            prefix: 0,
            control: ControlType::PowerDownComm as u8,
            data: (mode as u16) | (channel as u16 >> 4),
            feature: channel,
            address: 0,
        }
    }

    pub fn get_enable_message(control: ControlType, data: InternalRefCommData, feature: InternalRefCommFeature) -> Message {
        Message {
            prefix: 0,
            control: control as u8,
            data: data as u16,
            feature: feature as u8,
            address: 0,
        }
    }

    pub fn get_write_message(channel: ChannelSelect, value: u16) -> Message {
        Message {
            prefix: 0,
            feature: 0,
            control: ControlType::WriteToChannelAndUpdateSingleRegister as u8,
            address: channel as u8,
            data: value,
        }
    }

    /// For asynchronous communication methods (e.g. Interrupt or DMA), this function
    /// prepares the DAC for the transfer, generates the command and passes it back to the initiator
    /// via the callback parameter
    pub fn prepare_transfer<F: FnMut([u8; 4]) -> ()>(
        &mut self,
        message: Message,
        mut callback: F,
    ) {
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
        message: Message
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
