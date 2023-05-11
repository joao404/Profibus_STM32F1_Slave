/*********************************************************************
 * Profibus SAP
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

use defmt::*;

use super::codec_hw_interface::CodecHwInterface;
use super::fdl::{Fdl, FdlConfig, FdlType, Service};
use super::types::{
    sap_codes, sap_diagnose_byte1, sap_diagnose_byte2, sap_diagnose_byte3, sap_diagnose_ext,
    sap_global_control, sap_set_parameter_request, DeviceState,
};

const RX_SIZE: usize = 255;

#[derive(Default)]
pub struct DeviceConfig {
    pub fdl_config: FdlConfig,
}

pub struct Device<
    CodecInterface,
    const BUF_SIZE: usize,
    const INPUT_DATA_SIZE: usize,
    const OUTPUT_DATA_SIZE: usize,
    const USER_PARA_SIZE: usize,
    const EXTERN_DIAG_PARA_SIZE: usize,
    const MODULE_CONFIG_SIZE: usize,
> {
    fdl: Fdl<CodecInterface>,
    //  pub tx_buffer: [u8; BUF_SIZE],
    device_state: DeviceState,

    input_data: [u8; INPUT_DATA_SIZE],
    input_data_buffer: [u8; INPUT_DATA_SIZE],
    output_data: [u8; OUTPUT_DATA_SIZE],
    output_data_buffer: [u8; OUTPUT_DATA_SIZE],
    user_para: [u8; USER_PARA_SIZE],
    extern_diag_para: [u8; EXTERN_DIAG_PARA_SIZE],
    module_config: [u8; MODULE_CONFIG_SIZE],

    diagnose_status_1: u8,
    master_addr: u8,
    group: u8,

    freeze: bool,
    sync: bool,
    watchdog_act: bool,

    freeze_configured: bool,
    sync_configured: bool,

    watchdog_time: u32,
}

impl<
        CodecInterface,
        const BUF_SIZE: usize,
        const INPUT_DATA_SIZE: usize,
        const OUTPUT_DATA_SIZE: usize,
        const USER_PARA_SIZE: usize,
        const EXTERN_DIAG_PARA_SIZE: usize,
        const MODULE_CONFIG_SIZE: usize,
    >
    Device<
        CodecInterface,
        BUF_SIZE,
        INPUT_DATA_SIZE,
        OUTPUT_DATA_SIZE,
        USER_PARA_SIZE,
        EXTERN_DIAG_PARA_SIZE,
        MODULE_CONFIG_SIZE,
    >
where
    CodecInterface: CodecHwInterface,
{
    pub fn new(
        codec_interface: CodecInterface,
        config: DeviceConfig,
        module_config: [u8; MODULE_CONFIG_SIZE],
    ) -> Self {
        let input_data = [0; INPUT_DATA_SIZE];
        let input_data_buffer = [0; INPUT_DATA_SIZE];
        let output_data = [0; OUTPUT_DATA_SIZE];
        let output_data_buffer = [0; OUTPUT_DATA_SIZE];
        let user_para = [0; USER_PARA_SIZE];
        let extern_diag_para = [0; EXTERN_DIAG_PARA_SIZE];

        let fdl = Fdl::<CodecInterface>::new(codec_interface, config.fdl_config, FdlType::Passiv);

        Self {
            // tx_buffer: [0; BUF_SIZE],
            fdl,
            device_state: DeviceState::Por,
            input_data,
            input_data_buffer,
            output_data,
            output_data_buffer,
            user_para,
            extern_diag_para,
            module_config,
            diagnose_status_1: sap_diagnose_byte1::STATION_NOT_READY,
            master_addr: 0xFF,
            group: 0,
            freeze: false,
            sync: false,
            watchdog_act: false,
            freeze_configured: false,
            sync_configured: false,
            watchdog_time: 0xFFFFFF,
        }
    }

    pub async fn run(&mut self) -> bool {
        let mut buffer: [u8; RX_SIZE] = [0; RX_SIZE];

        match self.fdl.run(&mut buffer[..]).await {
            Some(service) => {
                // service.connection.
                self.handle_message(service);
            }
            None => (),
        }
        true
    }

    //TODO: handle message and send reply afterwards
    //FDL needs interface for sending back as SDN etc.

    fn handle_message(&mut self, service: Service) {
        match service.connection.sap {
            Some((dsap, ssap)) => {
                // SAP detected
                match dsap {
                    54 => self.handle_sap_54(),
                    55 => self.handle_sap_55(),
                    _ => (),
                }
            }
            None => {
                // NIL
                self.handle_sap_nil();
            }
        }
        // service.connection.destination_addr
        // match dsap
        // {
        //     ....
        // }
    }

    fn handle_sap_nil(&mut self) {}

    fn handle_sap_54(&mut self) {}

    fn handle_sap_55(&mut self)
    {
        // // Set Slave Address (SSAP 62 -> DSAP 55)
        //                     // Siehe Felser 8/2009 Kap. 4.2
        //                     if DpSlaveState::Wrpm == self.slave_state {
        //                         if 6 == pdu.len() {
        //                             self.codec.config.t_s = pdu[2];
        //                             self.fdl.ident_high = pdu[3];
        //                             self.fdl.ident_low = pdu[4];
        //                             //TODO
        //                             // if (pb_uart_buffer[12] & 0x01) adress_aenderung_sperren = true;
        //                             // trigger value saving
        //                         }
        //                     }
        //                     response = true;
        //                     self.transmit_message_sc();
    }
}
