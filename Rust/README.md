# Profibus DP Stm32f1 slave
Example code for usage of stm32f1 as profibus dp slave based on cortex-m-rtic 1.0.0 and stm32f1xx-hal 0.10.0

Example for handling and flashing stm32f1 can be found at https://jonathanklimt.de/electronics/programming/embedded-rust/rust-on-stm32-2/.
cargo build --release
cargo flash --chip stm32f103C8 --release
