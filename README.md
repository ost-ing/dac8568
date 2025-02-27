# dac8568

A platform agnostic library for the Texas Instruments DAC8568.

[![crates.io](https://img.shields.io/crates/v/dac8568.svg)](https://crates.io/crates/dac8568)
[![Documentation](https://docs.rs/dac8568/badge.svg)](https://docs.rs/dac8568)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.46+-blue.svg)

![dac8568](https://github.com/ostenning/images/blob/main/ssop16.png?raw=true)

## description

The `DAC7568`, `DAC8168`, and `DAC8568` are low-power, voltage-output, eight-channel, 12-, 14- and 16-bit digital-to-analog converters, respectively. These devices include a 2.5V, 2ppm/°C internal reference (disabled by default), giving a full-scale output voltage range of 2.5V or 5V. The internal reference has an initial accuracy of 0.004% and can source up to 20mA at the VREFIN/VREFOUT pin. These devices are monotonic, providing excellent linearity and minimizing undesired code-to-code transient voltages (glitch). They use a versatile 3-wire serial interface that operates at clock rates up to 50MHz. The interface is compatible with standard SPI™, QSPI™, Microwire™, and digital signal processor (DSP) interfaces.

## features

- Support for Texas Instruments DAC8568
- Heapless & no-std compatible
- Implemented with [embedded-hal](https://crates.io/crates/embedded-hal) and [embedded-async-hal](https://crates.io/crates/embedded-hal-async)
- Asynchronous and blocking support
- Basic feature set including synchronous static mode

## wip features

Feel free to create an issue and PR if you would like to add support for the more advanced features

- Asynchronous modes utilizing the LDAC line
- Flexible mode
- Generic support for DAC7568 (12-Bit) and DAC8168 (14-Bit)

## example

```rust
// The following example is compatible with embassy.rs and its asynchronous SPI

// Set LDAC low to update DAC immedaitely after writing
let _ldac = Output::new(ldac, embassy_stm32::gpio::Level::Low, Speed::Medium);
// Set CLR high for normal operation
let _clear = Output::new(clear, embassy_stm32::gpio::Level::High, Speed::Medium);
// Initialize asynchronous SPI with DMA. Ensure correct polarity and phase
let mut spi_config = spi::Config::default();
{
    spi_config.frequency = mhz(1);
    spi_config.mode = Mode {
        phase: spi::Phase::CaptureOnSecondTransition,
        polarity: spi::Polarity::IdleLow,
    };
    spi_config.rise_fall_speed = Speed::High;
    spi_config.bit_order = BitOrder::MsbFirst;
}
let spi = spi::Spi::new_txonly(interface, sck, mosi, p.DMA2_CH4, spi_config);
// Initilize the sync line
let sync = Output::new(sync, embassy_stm32::gpio::Level::High, Speed::High);
// Initialize the asynchronous dac instance
let mut dac = dac8568::Dac::new(spi, sync);
// Perform a software reset of DAC8568 to clear all registers
dac.reset().await;
// Configure the DAC to use the internal 2.5v reference
dac.use_internal_reference().await.unwrap();
// Optionally, invert the output signal
dac.set_inverted_output(true);
// Now transfer the data to update the DAC as a blocking call
dac.set_voltage(dac8568::Channel::A, voltage).await.unwrap();

// Alternatively, you can maintain ownership of the SPI and SYNC if you need to use
// custom transfer mechanisms like circular DMA on non-async drivers.
let (spi, sync) = dac.release();
// And then access the desired message directly
let message = dac8568::Message::get_voltage_message(dac8568::Channel::A, voltage, false);
// Get the message data that can be transferred manually
let payload = message.get_payload_bytes();
// And then write the message bytes to a DMA RAM buffer
```
