# dac8568 

A platform agnostic library for the Texas Instruments DAC8568.

- https://crates.io/crates/dac8568

![dac8568](/documentation/dac8568_ssop16.png)

## description

The `DAC7568`, `DAC8168`, and `DAC8568` are low-power, voltage-output, eight-channel, 12-, 14- and 16-bit digital-to-analog converters, respectively. These devices include a 2.5V, 2ppm/°C internal reference (disabled by default), giving a full-scale output voltage range of 2.5V or 5V. The internal reference has an initial accuracy of 0.004% and can source up to 20mA at the VREFIN/VREFOUT pin. These devices are monotonic, providing excellent linearity and minimizing undesired code-to-code transient voltages (glitch). They use a versatile 3-wire serial interface that operates at clock rates up to 50MHz. The interface is compatible with standard SPI™, QSPI™, Microwire™, and digital signal processor (DSP) interfaces.

## features

- Support for Texas Instruments DAC8568
- Full no-std support
- Implemented with `embedded-hal` (https://docs.rs/embedded-hal/0.2.3/embedded_hal)
- Blocking and non-blocking support
- Basic feature set including synchronous static mode

## wip features

Feel free to create an issue and PR if you would like to add support for the more advanced features

- Asynchronous modes utilizing the LDAC line
- Flexible mode
- Generic support for DAC7568 (12-Bit) and DAC8168 (14-Bit)

## example

Note: Quick example based on the `stm32h7xx-hal`.

```rust
// Initialise SPI. Ensure correct polarity and phase are respected
let spi: Spi<SPI1, Enabled> = interface.spi(
    (sck, NoMiso, mosi),
    spi::MODE_1, // polarity: Polarity::IdleLow,
                 // phase: Phase::CaptureOnSecondTransition,
    20.mhz(),
    prec,
    clocks,
);
// Initialise SYNC for SPI communications
let sync = sync.into_push_pull_output();
// Initialize the dac instance
let mut dac = dac8568::Dac::new(spi, sync);
// Get a "write" message to set the voltage of a given channel
let message = dac8568::Message::get_write_message(dac8568::Channel::A, voltage);
// Now transfer the data as a blocking call
dac.write(message).unwrap();

// Alternatively, you can maintain ownership of the SPI and SYNC if you need to use
// asynchronous communication such as Interrupts and/or DMA
let (spi, sync) = dac.release();
let transfer = DmaTransfer::new(spi);
// Get the message data-frame that can be transferred manually
let payload = message.get_payload(); 
// And then write it to a DMA RAM buffer...
```