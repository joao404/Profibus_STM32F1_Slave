/*********************************************************************
 * Fdl
 *
 * Copyright (C) 2023 Marcel Maage
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * LICENSE file for more details.
 */

// use super::codec::{Codec, Config, Fdl as FdlTrait, ReceiveHandling, UartAccess};
// use super::data_handling_interface::DataHandlingInterface;
// use super::hw_interface::HwInterface;

// #[allow(dead_code)]
// pub struct Fdl<'a, CodecInterface> {
//     pub codec: Codec<'a, CodecInterface, Fdl<'a, CodecInterface>>,
//     // pub data_handling_interface: DataHandling,
// }

// impl<'a, CodecInterface> Fdl<'a, CodecInterface>
// where
//     CodecInterface: HwInterface,
//     // DataHandling: DataHandlingInterface,
// {
//     pub fn new(
//         mut config: Config,
//         codec_interface: CodecInterface,
//         // data_handling_interface: DataHandling,
//     ) -> Self {
//         let mut instance = Self {
//             codec: Codec::new(codec_interface, config),
//             // data_handling_interface,
//         };
//         instance.codec.set_fdl(&instance);
//         instance
//     }
// }

// impl<'a, CodecInterface> FdlTrait for Fdl<'a, CodecInterface>
// where
//     CodecInterface: HwInterface,
//     // DataHandling: DataHandlingInterface,
// {
//     fn handle_data_receive(
//         &self,
//         source_addr: u8,
//         destination_addr: u8,
//         function_code: u8,
//         pdu: &[u8],
//     ) -> bool {
//         true
//     }
// }
