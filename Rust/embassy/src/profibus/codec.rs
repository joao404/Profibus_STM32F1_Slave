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

use super::codec_hw_interface::CodecHwInterface;
use super::types::{cmd_type, StreamState};
use defmt::*;

pub struct CodecConfig {
    pub t_s: u8,
    pub t_sl: u16,
    pub t_sdr_min: u16,
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

pub struct Connection<'a> {
    pub source_addr: u8,
    pub destination_addr: u8,
    pub function_code: u8,
    pub pdu: &'a [u8],
}

impl<'a> Connection<'a> {
    pub fn new(source_addr: u8, destination_addr: u8, function_code: u8, pdu: &'a [u8]) -> Self {
        Self {
            source_addr,
            destination_addr,
            function_code,
            pdu,
        }
    }
}

#[allow(dead_code)]
pub struct Codec<SerialInterface> {
    config: CodecConfig,
    hw_interface: SerialInterface,

    stream_state: StreamState,
    timeout_max_syn_time_in_us: u32,
    timeout_max_rx_time_in_us: u32,
    timeout_max_tx_time_in_us: u32,
    timeout_max_sdr_time_in_us: u32,
    timer_timeout_in_us: u32,
}

const SAP_OFFSET: u8 = 128;
const BROADCAST_ADD: u8 = 127;
const DEFAULT_ADD: u8 = 126;

impl<SerialInterface> Codec<SerialInterface>
where
    SerialInterface: CodecHwInterface,
{
    pub fn new(mut hw_interface: SerialInterface, mut config: CodecConfig) -> Self {
        let baudrate = hw_interface.get_baudrate();

        let timeout_max_syn_time_in_us = 33_000_000u32 / baudrate; // 33 TBit = TSYN
        let timeout_max_rx_time_in_us = 15_000_000u32 / baudrate;
        let timeout_max_tx_time_in_us = 15_000_000u32 / baudrate;
        let timeout_max_sdr_time_in_us = 15_000_000u32 / baudrate; // 15 Tbit = TSDR

        let timer_timeout_in_us = timeout_max_syn_time_in_us;

        if (0 == config.t_s) || (config.t_s > DEFAULT_ADD) {
            config.t_s = DEFAULT_ADD;
        }

        let stream_state = StreamState::WaitSyn;

        // Timer init
        hw_interface.config_timer();
        // Pin Init
        hw_interface.config_rs485_pin();

        // Uart Init
        hw_interface.config_uart();
        hw_interface.rx_rs485_enable();

        info!("codec start");
        Self {
            config,
            hw_interface,
            stream_state,
            timeout_max_syn_time_in_us,
            timeout_max_rx_time_in_us,
            timeout_max_tx_time_in_us,
            timeout_max_sdr_time_in_us,
            timer_timeout_in_us,
        }
    }

    async fn reset_data_stream(&mut self) {
        self.stream_state = StreamState::WaitSyn;
        self.hw_interface
            .wait_for(self.timeout_max_syn_time_in_us)
            .await;
        self.hw_interface.rx_rs485_enable();
    }

    pub async fn receive<'a>(&mut self, buffer: &'a mut [u8]) -> Option<Connection<'a>> {
        self.hw_interface.rx_rs485_disable();
        self.hw_interface
            .wait_for(self.timeout_max_syn_time_in_us)
            .await;
        self.hw_interface.rx_rs485_enable();

        loop {
            match self.receive_and_check(buffer).await {
                Some(conn) => {
                    self.hw_interface.rx_rs485_disable();
                    return Some(conn);
                }
                None => {
                    self.reset_data_stream().await;
                }
            }
        }
    }

    async fn receive_and_check<'a>(&mut self, buffer: &'a mut [u8]) -> Option<Connection<'a>> {
        let mut rx_len = 0;
        self.hw_interface
            .receive_uart_data(buffer, &mut rx_len)
            .await;
        match self.check_data(&buffer[0..rx_len]) {
            Some(conn) => return Some(conn),
            None => {
                self.reset_data_stream().await;
                None
            }
        }
    }

    pub fn check_data<'a>(&mut self, buffer: &'a [u8]) -> Option<Connection<'a>> {
        let rx_len = buffer.len();
        let t_s = self.config.t_s;
        let buf = buffer;
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
                                return Some(Connection::new(
                                    source_addr,
                                    destination_addr,
                                    function_code,
                                    &[0; 0],
                                ));
                            }
                        }
                    }
                }
            }

            cmd_type::SD2 => {
                if rx_len > 4 {
                    if rx_len == usize::from(buf[1] + 6) {
                        if cmd_type::ED == buf[rx_len - 1] {
                            let pdu_len = buf[1]; // DA+SA+FC+PDU
                            let destination_addr = buf[4];
                            let source_addr = buf[5];
                            let mut function_code = buf[6];
                            let fcs_data = buf[usize::from(pdu_len + 4)]; // Frame Check Sequence
                            if check_destination_addr(t_s, destination_addr) {
                                if fcs_data == calc_checksum(&buf[4..usize::from(rx_len - 2)]) {
                                    return Some(Connection::new(
                                        source_addr,
                                        destination_addr,
                                        function_code,
                                        &buf[7..usize::from(rx_len - 2)],
                                    ));
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
                                return Some(Connection::new(
                                    source_addr,
                                    destination_addr,
                                    function_code,
                                    &buf[4..12],
                                ));
                            }
                        }
                    }
                }
            }

            cmd_type::SD4 => {
                if 3 == rx_len {
                    let destination_addr = buf[1];
                    let source_addr = buf[2];

                    if check_destination_addr(self.config.t_s, destination_addr) {
                        //TODO
                        return Some(Connection::new(source_addr, destination_addr, 0, &[0; 0]));
                    }
                }
            }
            cmd_type::SC => {
                if 1 == rx_len {
                    return Some(Connection::new(0, 0, 0, &[0; 0]));
                }
            }
            _ => (),
        } // match self.buffer[0]
        return None;
        // if !response {
        //     self.reset_data_stream();
        // }
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sd1<'a>(
        &mut self,
        buffer: &'a mut [u8],
        connection: Connection<'a>,
        sap_offset: bool,
    ) {
        let t_s = self.config.t_s;
        let tx_len = message_sd1(
            buffer,
            connection.destination_addr,
            connection.function_code,
            sap_offset,
            t_s,
        );
        self.transmit(&buffer[0..tx_len]).await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sd2<'a>(
        &mut self,
        buffer: &mut [u8],
        connection: Connection<'a>,
        sap_offset: bool,
    ) {
        let t_s = self.config.t_s;
        let tx_len = message_sd2(
            buffer,
            connection.destination_addr,
            connection.function_code,
            sap_offset,
            t_s,
            connection.pdu,
        );
        self.transmit(&buffer[0..tx_len]).await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sd3<'a>(
        &mut self,
        buffer: &mut [u8],
        connection: Connection<'a>,
        sap_offset: bool,
    ) {
        let t_s = self.config.t_s;
        let tx_len = message_sd3(
            buffer,
            connection.destination_addr,
            connection.function_code,
            sap_offset,
            t_s,
            connection.pdu,
        );
        self.transmit(&buffer[0..tx_len]).await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sd4<'a>(
        &mut self,
        buffer: &mut [u8],
        connection: Connection<'a>,
        sap_offset: bool,
    ) {
        let t_s = self.config.t_s;

        let tx_len = message_sd4(buffer, connection.destination_addr, sap_offset, t_s);
        self.transmit(&buffer[0..tx_len]).await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sc(&mut self, buffer: &mut [u8]) {
        let tx_len = message_sc(buffer);
        self.transmit(&buffer[0..tx_len]).await;
    }

    pub async fn transmit(&mut self, buffer: &[u8]) {
        if 0 != self.config.t_sdr_min {
            self.stream_state = StreamState::WaitMinTsdr;
            let baudrate = self.hw_interface.get_baudrate();
            self.timer_timeout_in_us = (1_000_000u32 * u32::from(self.config.t_sdr_min)) / baudrate;
            self.hw_interface.wait_for(self.timer_timeout_in_us).await;
        }

        self.stream_state = StreamState::SendData;
        self.hw_interface.wait_for_activ_transmission().await;
        self.hw_interface.tx_rs485_enable();
        self.hw_interface.send_uart_data(&buffer).await;
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

pub fn message_sd1(
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

pub fn message_sd2(
    buffer: &mut [u8],
    destination_addr: u8,
    function_code: u8,
    sap_offset: bool,
    t_s: u8,
    pdu: &[u8],
) -> usize {
    buffer[0] = cmd_type::SD2;
    buffer[1] = 3 + pdu.len().to_le_bytes()[0];
    buffer[2] = 3 + pdu.len().to_le_bytes()[0];
    buffer[3] = cmd_type::SD2;
    buffer[4] = destination_addr;
    buffer[5] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
    buffer[6] = function_code;
    if pdu.len() > 0 {
        for i in 0..pdu.len() {
            buffer[7 + i] = pdu[i];
        }
    }
    let checksum = calc_checksum(&buffer[4..7]) + calc_checksum(pdu);
    buffer[7 + pdu.len()] = checksum;
    buffer[8 + pdu.len()] = cmd_type::ED;
    9 + pdu.len()
}

#[allow(dead_code)]
pub fn message_sd3(
    buffer: &mut [u8],
    destination_addr: u8,
    function_code: u8,
    sap_offset: bool,
    t_s: u8,
    pdu: &[u8],
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
pub fn message_sd4(buffer: &mut [u8], destination_addr: u8, sap_offset: bool, t_s: u8) -> usize {
    buffer[0] = cmd_type::SD4;
    buffer[1] = destination_addr;
    buffer[2] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
    3
}

pub fn message_sc(buffer: &mut [u8]) -> usize {
    buffer[0] = cmd_type::SC;
    1
}
