/*********************************************************************
 * Profibus Codec
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

use super::codec_hw_interface::HwInterface;
use super::slave::PbDpSlave;
use super::types::{cmd_type, StreamState};

use super::data_handling_interface::DataHandlingInterface;

pub struct CodecConfig {
    pub(super) t_s: u8,
    pub(super) t_sl: u16,
    pub(super) t_sdr_min: u16,
}

impl CodecConfig {
    #[allow(dead_code)]
    pub fn t_s(mut self, t_s: u8) -> Self {
        self.t_s = t_s;
        self
    }

    #[allow(dead_code)]
    pub fn t_sl(mut self, t_sl: u16) -> Self {
        self.t_sl = t_sl;
        self
    }

    #[allow(dead_code)]
    pub fn t_sdr_min(mut self, t_sdr_min: u16) -> Self {
        self.t_sdr_min = t_sdr_min;
        self
    }
}

impl Default for CodecConfig {
    fn default() -> CodecConfig {
        CodecConfig {
            t_s: 126,
            t_sl: 65000,
            t_sdr_min: 20,
        }
    }
}

pub(super) struct Codec {
    pub(super) config: CodecConfig,

    pub(super) tx_len: usize,

    pub(super) stream_state: StreamState,
    pub(super) timeout_max_syn_time_in_us: u32,
    pub(super) timeout_max_rx_time_in_us: u32,
    pub(super) timeout_max_tx_time_in_us: u32,
    pub(super) timeout_max_sdr_time_in_us: u32,

    pub(super) timer_timeout_in_us: u32,
}

impl Codec {
    // fn new() -> Self{
    //     let mut instance = CodecVariables::default();

    //     instance
    // }
}

impl Default for Codec {
    fn default() -> Codec {
        Codec {
            config: CodecConfig::default(),
            tx_len: 0,
            stream_state: StreamState::WaitSyn,
            timeout_max_syn_time_in_us: 0xFFFFFFFF,
            timeout_max_rx_time_in_us: 0xFFFFFFFF,
            timeout_max_tx_time_in_us: 0xFFFFFFFF,
            timeout_max_sdr_time_in_us: 0xFFFFFFFF,

            timer_timeout_in_us: 0xFFFFFFFF,
        }
    }
}

const SAP_OFFSET: u8 = 128;
const BROADCAST_ADD: u8 = 127;
const DEFAULT_ADD: u8 = 126;

impl<
        Serial,
        DataHandling,
        const BUF_SIZE: usize,
        const INPUT_DATA_SIZE: usize,
        const OUTPUT_DATA_SIZE: usize,
        const USER_PARA_SIZE: usize,
        const EXTERN_DIAG_PARA_SIZE: usize,
        const MODULE_CONFIG_SIZE: usize,
    >
    PbDpSlave<
        Serial,
        DataHandling,
        BUF_SIZE,
        INPUT_DATA_SIZE,
        OUTPUT_DATA_SIZE,
        USER_PARA_SIZE,
        EXTERN_DIAG_PARA_SIZE,
        MODULE_CONFIG_SIZE,
    >
where
    Serial: HwInterface,
    DataHandling: DataHandlingInterface,
{
    pub(super) fn codec_init(
        codec: &mut Codec,
        hw_interface: &mut Serial,
        data_handling_interface: &mut DataHandling,
        config: CodecConfig,
    ) {
        let baudrate = hw_interface.get_baudrate();

        codec.config = config;
        codec.timeout_max_syn_time_in_us = 33_000_000u32 / baudrate; // 33 TBit = TSYN
        codec.timeout_max_rx_time_in_us = 15_000_000u32 / baudrate;
        codec.timeout_max_tx_time_in_us = 15_000_000u32 / baudrate;
        codec.timeout_max_sdr_time_in_us = 15_000_000u32 / baudrate; // 15 Tbit = TSDR

        codec.timer_timeout_in_us = codec.timeout_max_syn_time_in_us;

        if (0 == codec.config.t_s) || (codec.config.t_s > DEFAULT_ADD) {
            codec.config.t_s = DEFAULT_ADD;
        }

        codec.stream_state = StreamState::WaitSyn;

        // Timer init
        hw_interface.config_timer();
        // LED Status
        data_handling_interface.config_error_led();
        // Pin Init
        hw_interface.config_rs485_pin();

        // Uart Init
        hw_interface.config_uart();
        hw_interface.rx_rs485_enable();

        data_handling_interface.debug_write("Profi");
    }

    fn reset_data_stream(&mut self) {
        self.codec.stream_state = StreamState::WaitSyn;
        self.hw_interface
            .wait_for(self.codec.timeout_max_syn_time_in_us);
        self.hw_interface.rx_rs485_enable();
    }

    pub(super) async fn run_codec(&mut self) {
        self.hw_interface.rx_rs485_disable();
        self.hw_interface
            .wait_for(self.codec.timeout_max_syn_time_in_us);
        let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];
        self.hw_interface.rx_rs485_enable();
        let mut rx_len = 0;
        self.hw_interface.receive_uart_data(&mut buffer, &mut rx_len);
        self.hw_interface.rx_rs485_disable();
        self.handle_codec_data(&buffer[0..rx_len]).await;
    }

    // self.fdl_timer_call();

    pub(super) async fn transmit_message_sd1(
        &mut self,
        destination_addr: u8,
        function_code: u8,
        sap_offset: bool,
    ) {
        let t_s = self.codec.config.t_s;
        self.codec.tx_len = message_sd1(&mut self.tx_buffer, destination_addr, function_code, sap_offset, t_s);
        self.transmit().await;
    }

    pub(super) async fn transmit_message_sd2(
        &mut self,
        destination_addr: u8,
        function_code: u8,
        sap_offset: bool,
        pdu1: &[u8],
        pdu2: &[u8],
    ) {
        let t_s = self.codec.config.t_s;
        self.codec.tx_len = message_sd2(&mut self.tx_buffer, destination_addr, function_code, sap_offset, t_s, pdu1, pdu2);
        self.transmit().await;
    }

    #[allow(dead_code)]
    pub(super) async fn transmit_message_sd3(
        &mut self,
        destination_addr: u8,
        function_code: u8,
        sap_offset: bool,
        pdu: &[u8; 8],
    ) {
        let t_s = self.codec.config.t_s;
        self.codec.tx_len = message_sd3(&mut self.tx_buffer, destination_addr, function_code, sap_offset, t_s, pdu);
        self.transmit().await;
    }

    #[allow(dead_code)]
    pub(super) async fn transmit_message_sd4(&mut self, destination_addr: u8, sap_offset: bool) {
        let t_s = self.codec.config.t_s;
        
        self.codec.tx_len = message_sd4(&mut self.tx_buffer, destination_addr, sap_offset, t_s);
        self.transmit().await;
    }

    pub(super) async fn transmit_message_sc(&mut self) {
        self.codec.tx_len = message_sc(&mut self.tx_buffer);
        self.transmit().await;
    }

    pub(super) async fn transmit(&mut self) {
        if 0 != self.codec.config.t_sdr_min {
            self.codec.stream_state = StreamState::WaitMinTsdr;
            let baudrate = self.hw_interface.get_baudrate();
            self.codec.timer_timeout_in_us =
                (1_000_000u32 * u32::from(self.codec.config.t_sdr_min)) / baudrate;
            self.hw_interface.wait_for(self.codec.timer_timeout_in_us);
        }

        self.codec.stream_state = StreamState::SendData;
        self.hw_interface.wait_for_activ_transmission();
        self.hw_interface.tx_rs485_enable();
        self.hw_interface
            .send_uart_data(&self.tx_buffer[0..self.codec.tx_len]).await;
    }

    pub async fn handle_codec_data(&mut self, rx_buffer: &[u8]) {
        let mut response = false;

        let rx_len = rx_buffer.len();
        let t_s = self.codec.config.t_s;
        let buf = rx_buffer;
        match buf[0] {
            cmd_type::SD1 => {
                if 6 == rx_len {
                    if cmd_type::ED == buf[5] {
                        let destination_addr = buf[1];
                        let source_addr = buf[2];
                        let mut function_code = buf[3];
                        let fcs_data = buf[4]; // Frame Check Sequence

                        if check_destination_addr(t_s, destination_addr) {
                            if fcs_data == calc_checksum(&buf[1..4]) {
                                // FCV und FCB loeschen, da vorher überprüft
                                function_code &= 0xCF;
                                response = self.fdl_handle_data(
                                    source_addr,
                                    destination_addr,
                                    function_code,
                                    &[0; 0],
                                ).await;
                            }
                        }
                    }
                }
            }

            cmd_type::SD2 => {
                if rx_len > 4 {
                    if rx_len == usize::from(buf[1] + 6) {
                        if cmd_type::ED == buf[rx_len - 1] {
                            let pdu_len = buf[1]; // DA+SA+FC+Nutzdaten
                            let destination_addr = buf[4];
                            let source_addr = buf[5];
                            let mut function_code = buf[6];
                            let fcs_data = buf[usize::from(pdu_len + 4)]; // Frame Check Sequence
                            if check_destination_addr(t_s, destination_addr) {
                                if fcs_data == calc_checksum(&buf[4..usize::from(rx_len - 2)]) {
                                    // FCV und FCB loeschen, da vorher überprüft
                                    function_code &= 0xCF;
                                    response = self.fdl_handle_data(
                                        source_addr,
                                        destination_addr,
                                        function_code,
                                        &buf[7..usize::from(rx_len - 2)],
                                    ).await;
                                }
                            }
                        }
                    }
                }
            }

            cmd_type::SD3 => {
                if 14 == rx_len {
                    if cmd_type::ED == buf[13] {
                        let destination_addr = buf[1];
                        let source_addr = buf[2];
                        let mut function_code = buf[3];
                        let fcs_data = buf[12]; // Frame Check Sequence

                        if check_destination_addr(t_s, destination_addr) {
                            if fcs_data == calc_checksum(&buf[1..12]) {
                                // FCV und FCB loeschen, da vorher überprüft
                                function_code &= 0xCF;
                                response = self.fdl_handle_data(
                                    source_addr,
                                    destination_addr,
                                    function_code,
                                    &buf[4..12],
                                ).await;
                            }
                        }
                    }
                }
            }

            cmd_type::SD4 => {
                if 3 == rx_len {
                    let destination_addr = buf[1];
                    let _source_addr = buf[2];

                    if check_destination_addr(self.codec.config.t_s, destination_addr) {
                        //TODO
                    }
                }
            }

            _ => (),
        } // match self.buffer[0]
        if !response {
            self.reset_data_stream();
        }
    }
}

fn calc_checksum(data: &[u8]) -> u8 {
    let mut checksum: u8 = 0;
    for x in data {
        checksum += *x;
    }
    checksum
}

fn check_destination_addr(address: u8, destination: u8) -> bool {
    if ((destination & 0x7F) != address) &&  // Slave
           ((destination & 0x7F) != BROADCAST_ADD)
    // Broadcast
    {
        false
    } else {
        true
    }
}

pub(super) fn message_sd1(
    buffer: &mut [u8],
    destination_addr: u8,
    function_code: u8,
    sap_offset: bool,
    t_s: u8,
) -> usize {
    buffer[0] = cmd_type::SD1;
    buffer[1] = destination_addr;
    buffer[2] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
    buffer[3] = function_code;
    buffer[4] = calc_checksum(&buffer[1..4]);
    buffer[5] = cmd_type::ED;
    6
}

pub(super) fn message_sd2(
    buffer: &mut [u8],
    destination_addr: u8,
    function_code: u8,
    sap_offset: bool,
    t_s: u8,
    pdu1: &[u8],
    pdu2: &[u8],
) -> usize {
    buffer[0] = cmd_type::SD2;
    buffer[1] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
    buffer[2] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
    buffer[3] = cmd_type::SD2;
    buffer[4] = destination_addr;
    buffer[5] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
    buffer[6] = function_code;
    if pdu1.len() > 0 {
        for i in 0..pdu1.len() {
            buffer[7 + i] = pdu1[i];
        }
    }
    if pdu2.len() > 0 {
        for i in 0..pdu2.len() {
            buffer[7 + i + pdu1.len()] = pdu2[i];
        }
    }
    let checksum = calc_checksum(&buffer[4..7]) + calc_checksum(pdu1) + calc_checksum(pdu2);
    buffer[7 + pdu1.len() + pdu2.len()] = checksum;
    buffer[8 + pdu1.len() + pdu2.len()] = cmd_type::ED;
    9 + pdu1.len() + pdu2.len()
}

#[allow(dead_code)]
pub(super) fn message_sd3(
    buffer: &mut [u8],
    destination_addr: u8,
    function_code: u8,
    sap_offset: bool,
    t_s: u8,
    pdu: &[u8; 8],
) -> usize {
    buffer[0] = cmd_type::SD3;
    buffer[1] = destination_addr;
    buffer[2] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
    buffer[3] = function_code;
    for i in 0..pdu.len() {
        buffer[4 + i] = pdu[i];
    }
    buffer[12] = calc_checksum(&buffer[1..12]);
    buffer[13] = cmd_type::ED;
    14
}

#[allow(dead_code)]
pub(super) fn message_sd4(
    buffer: &mut [u8],
    destination_addr: u8,
    sap_offset: bool,
    t_s: u8,
) -> usize {
    buffer[0] = cmd_type::SD4;
    buffer[1] = destination_addr;
    buffer[2] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
    3
}

pub(super) fn message_sc(buffer: &mut [u8]) -> usize {
    buffer[0] = cmd_type::SC;
    1
}
