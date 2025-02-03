use embedded_hal::digital::OutputPin;

use crate::{mode::Async, Channel, Dac, DacError, Message};

impl<SPI, SYNC> Dac<SPI, SYNC, Async> 
    where SPI: embedded_hal_async::spi::SpiBus, SYNC: OutputPin,

{
    /// Set the specified value to the given channel. This will update the DAC
    /// to output the desired voltage
    pub async fn set_voltage(&mut self, channel: Channel, voltage: u16) -> Result<(), DacError> {
        let message = Message::get_voltage_message(channel, voltage, self.is_inverted);
        self.write(message).await
    }

    /// Configure the DAC to use its internal reference mode of 2.5v rather than using an external
    /// voltage reference
    pub async fn use_internal_reference(&mut self) -> Result<(), DacError> {
        let message = Message::get_internal_reference_message(true);
        self.write(message).await
    }

    /// Configure the DAC to use its external reference mode rather than using the internal reference
    pub async fn use_external_reference(&mut self) -> Result<(), DacError> {
        let message = Message::get_internal_reference_message(false);
        self.write(message).await
    }

    /// Perform a software reset, clearing out all registers
    /// 8.2.10 Software Reset Function
    /// The DAC7568, DAC8168, and DAC8568 contain a software reset feature.
    /// If the software reset feature is executed, all registers inside the device are reset to default settings; that is,
    /// all DAC channels are reset to the power-on reset code (power on reset to zero scale for grades A and C; power on reset to midscale for grades B and D).
    pub async fn reset(&mut self) -> Result<(), DacError> {
        let message = Message::get_software_reset_message();
        self.write(message).await
    }

    /// Write to the DAC via a blocking call on the specified SPI interface
    async fn write(&mut self, message: Message) -> Result<(), DacError> {
        let command: [u8; 4] = message.get_payload_bytes();

        self.sync.set_low().unwrap_or_default();
        let result = self.spi.write(&command).await;
        self.sync.set_high().unwrap_or_default();

        match result {
            Ok(v) => Ok(v),
            Err(_e) => Err(DacError::BusWriteError),
        }
    }
}