# Stm32f1_rust_can
Short example for usage of can with cortex-m-rtic 1.0.0 and stm32f1xx-hal 0.9.0

Example for handling and flashing stm32f1 can be found at https://jonathanklimt.de/electronics/programming/embedded-rust/rust-on-stm32-2/.
cargo build --release
cargo flash --chip stm32f103C8 --release
