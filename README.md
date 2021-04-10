# dac8568

A platform agnostic library for the Texas Instruments DAC8568.

- https://crates.io/crates/dac8568

## features

- Support for Texas Instruments DAC8568
- Limited subset of DAC8568 features supported
- Full no-std support
- Implemented with `embedded-hal` (https://docs.rs/embedded-hal/0.2.3/embedded_hal)
- Blocking and non-blocking support

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