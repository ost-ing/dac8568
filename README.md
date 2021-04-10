# dac8568 

A platform agnostic library for the Texas Instruments DAC8568.

- https://crates.io/crates/dac8568

![dac8568](/documentation/dac8568_ssop16.png)

## features

- Support for Texas Instruments DAC8568
- Full no-std support
- Implemented with `embedded-hal` (https://docs.rs/embedded-hal/0.2.3/embedded_hal)
- Blocking and non-blocking support

## dac features

- Synchronous feature support
- Static mode

## wip features

Feel free to create an issue and PR if you would like to add support for the more advanced features of this DAC series.

- Asynchronous modes utilizing the LDAC line
- Flexible mode

## example

Note: Quick example based on the `stm32h7xx-hal`.

```rust
// Initialise NSS for SPI communications
let spi = ...;
let nss = nss.into_push_pull_output();
// Initialize the dac instance
let mut dac = dac8568::Dac::new(nss);
dac.enable();
// Get a "write" message to set the voltage of a given channel
let message = dac8568::Message::get_write_message(dac8568::Channel::A, voltage);
// Now transfer the data either as a blocking call
dac.write_blocking(spi, message).unwrap();
// or prepare the data for a DMA transfer
dac.prepare_transfer(message, |payload| {
    // begin DMA transfer with bytes payload 
});
```