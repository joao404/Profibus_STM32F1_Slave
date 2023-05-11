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
use super::types::{cmd_type, StreamState, FcRequestHighNibble};
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

pub struct ConnectionParam {
    pub source_addr: u8,
    pub destination_addr: u8,
    pub function_code: u8,
    pub pdu_start: usize,
    pub pdu_end: usize,
    // pub pdu: &'pdu [u8],
}

impl ConnectionParam {
    pub fn new(
        source_addr: u8,
        destination_addr: u8,
        function_code: u8,
        pdu_start: usize,
        pdu_end: usize,
    ) -> Self {
        Self {
            source_addr,
            destination_addr,
            function_code,
            pdu_start,
            pdu_end,
        }
    }
}

pub struct Connection<'pdu> {
    pub source_addr: u8,
    pub destination_addr: u8,
    pub function_code: u8,
    pub sap: Option<(u8, u8)>, //(dsap, ssap)
    pub pdu: &'pdu [u8],
}

impl<'pdu> Connection<'pdu> {
    pub fn new(
        source_addr: u8,
        destination_addr: u8,
        function_code: u8,
        sap: Option<(u8, u8)>, //(dsap, ssap)
        pdu: &'pdu [u8],
    ) -> Self {
        Self {
            source_addr,
            destination_addr,
            function_code,
            sap,
            pdu,
        }
    }
}

const TX_SIZE: usize = 255;

#[allow(dead_code)]
pub struct Codec<SerialInterface> {
    config: CodecConfig,
    hw_interface: SerialInterface,

    tx_buffer: [u8; TX_SIZE],
    tx_len: usize,

    timeout_max_syn_time_in_us: u32,
    timeout_max_rx_time_in_us: u32,
    timeout_max_tx_time_in_us: u32,
    timeout_max_sdr_time_in_us: u32,
    timer_timeout_in_us: u32,

    stream_state: StreamState,

    source_addr: u8,
    fcv_activated: bool,
    fcb_last: bool,
}

const BROADCAST_ADD: u8 = 0x7F;
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
            tx_buffer: [0; TX_SIZE],
            tx_len: 0,
            timeout_max_syn_time_in_us,
            timeout_max_rx_time_in_us,
            timeout_max_tx_time_in_us,
            timeout_max_sdr_time_in_us,
            timer_timeout_in_us,

            stream_state,

            source_addr: 0xFF,
            fcv_activated: false,
            fcb_last: false,
        }
    }

    async fn reset_data_stream(&mut self) {
        self.stream_state = StreamState::WaitSyn;
        self.hw_interface
            .wait_for(self.timeout_max_syn_time_in_us)
            .await;
        self.hw_interface.rx_rs485_enable();
    }

    pub async fn receive<'buf: 'out, 'out>(
        &'_ mut self,
        buffer: &'buf mut [u8],
    ) -> Option<Connection<'out>> {
        self.hw_interface.rx_rs485_disable();
        // wait for syn time
        self.stream_state = StreamState::WaitSyn;
        self.hw_interface
            .wait_for(self.timeout_max_syn_time_in_us)
            .await;
        self.hw_interface.rx_rs485_enable();

        let result = loop {
            match self.receive_and_check(buffer).await {
                Some(connparam) => {
                    break connparam;
                }
                _ => (),
            };
            // reset data stream in case that
            self.reset_data_stream().await;
        };
        self.hw_interface.rx_rs485_disable();
        self.stream_state = StreamState::HandleData;
        let mut sap = None;
        if result.pdu_start != 0xFF {
            if sap_active(result.source_addr) && sap_active(result.destination_addr) {
                sap = Some((result.destination_addr & 0x7F, result.source_addr & 0x7F))
            }
        }
        Some(Connection::new(
            result.source_addr,
            result.destination_addr,
            result.function_code,
            sap,
            &buffer[result.pdu_start..result.pdu_end],
        ))
    }

    async fn receive_and_check<'buf: 'out, 'out>(
        &'_ mut self,
        buffer: &'out mut [u8],
    ) -> Option<ConnectionParam> {
        let mut rx_len = 0;
        self.stream_state = StreamState::WaitData;
        self.hw_interface
            .receive_uart_data(buffer, &mut rx_len)
            .await;
        let result = match Self::check_telegram_format(self.config.t_s, &buffer[0..rx_len]) {
            Some(conn) => {
                if (conn.function_code & 0x30) == FcRequestHighNibble::FCB
                // fcb start
                {
                    self.fcv_activated = true;
                    self.fcb_last = true;
                } else if self.fcv_activated {
                    if conn.source_addr != self.source_addr {
                        // new address so fcv is deactivated
                        self.fcv_activated = false;
                    } else if ((conn.function_code & FcRequestHighNibble::FCB) != 0) == self.fcb_last {
                        // FCB is identical, repeat message
                        self.transmit().await;
                        return None;
                    } else {
                        // save new FCB bit
                        self.fcb_last = !self.fcb_last;
                    }
                } else
                // deactivate fcv
                {
                    self.fcv_activated = false;
                }

                // save last address
                self.source_addr = conn.source_addr;

                Some(conn)
            }
            None => None,
        };
        result

        // let result = match self.hw_interface.receive_uart_data().await {
        //             Some(buffer) => match Self::check_data(self.config.t_s, buffer) {
        //                 Some(conn) => Some(conn),
        //                 None => None,
        //             },
        //             None => None,
        //         };
        //         Self::reset_data_stream(self).await;
        //         result
    }

    // todo!(give SAP to next higher layer)
    // todo!(function code analysis => SDN/SDR inclusive FCB/FCV)
    // todo!(an den FDL layer werden dann die einzelnen PDUs mit SAP übergeben bzw. die zyklischen Daten)
    // todo!(Problem: ich muss das letzte gesendete Telegram hier speichern, was bedeutet, dass ich eine Speicher brauche
    // und damit wieder das Referenzproblem auftritt)
    // todo!(Die FDL hat einzelne SAP Objekte und das Objekt, welches einfach nur für den zyklischen Datenaustausch da ist)
    // todo!(Beachte die tabelle bezüglch des Löschens des FCB)

    fn check_telegram_format<'buf: 'out, 'out>(
        t_s: u8,
        buffer: &'buf [u8],
    ) -> Option<ConnectionParam> {
        let rx_len = buffer.len();
        let buf = buffer;
        match buf[0] {
            cmd_type::SD1 => {
                if 6 == rx_len {
                    if cmd_type::ED == buf[5] {
                        let destination_addr = buf[1];
                        let source_addr = buf[2];
                        let function_code = buf[3];
                        let fcs_data = buf[4]; // Frame Check Sequence

                        if check_destination_addr(t_s, destination_addr) {
                            if fcs_data == calc_checksum(&buf[1..4]) {
                                return Some(ConnectionParam::new(
                                    source_addr,
                                    destination_addr,
                                    function_code,
                                    0xFF,
                                    0,
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
                            let function_code = buf[6];
                            let fcs_data = buf[usize::from(pdu_len + 4)]; // Frame Check Sequence
                            if check_destination_addr(t_s, destination_addr) {
                                if fcs_data == calc_checksum(&buf[4..usize::from(rx_len - 2)]) {
                                    return Some(ConnectionParam::new(
                                        source_addr,
                                        destination_addr,
                                        function_code,
                                        7,
                                        usize::from(rx_len - 2),
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
                        let function_code = buf[3];
                        let fcs_data = buf[12]; // Frame Check Sequence

                        if check_destination_addr(t_s, destination_addr) {
                            if fcs_data == calc_checksum(&buf[1..12]) {
                                return Some(ConnectionParam::new(
                                    source_addr,
                                    destination_addr,
                                    function_code,
                                    4,
                                    12,
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

                    if check_destination_addr(t_s, destination_addr) {
                        //TODO
                        return Some(ConnectionParam::new(
                            source_addr,
                            destination_addr,
                            0,
                            0xFF,
                            0,
                        ));
                    }
                }
            }
            cmd_type::SC => {
                if 1 == rx_len {
                    return Some(ConnectionParam::new(0, 0, 0, 0xFF, 0));
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
    pub async fn transmit_message_sd1<'a>(&mut self, connection: Connection<'a>) {
        let t_s = self.config.t_s;
        self.tx_len = message_sd1(
            &mut self.tx_buffer[..],
            connection.destination_addr,
            t_s,
            connection.function_code,
        );
        self.transmit().await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sd2<'a>(&mut self, connection: Connection<'a>) {
        let t_s = self.config.t_s;
        self.tx_len = message_sd2(
            &mut self.tx_buffer[..],
            connection.destination_addr,
            t_s,
            connection.function_code,
            connection.sap,
            connection.pdu,
        );
        self.transmit().await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sd3<'a>(&mut self, connection: Connection<'a>) {
        let t_s = self.config.t_s;
        self.tx_len = message_sd3(
            &mut self.tx_buffer[..],
            connection.destination_addr,
            t_s,
            connection.function_code,
            connection.sap,
            connection.pdu,
        );
        self.transmit().await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sd4<'a>(&mut self, connection: Connection<'a>) {
        let t_s = self.config.t_s;

        self.tx_len = message_sd4(&mut self.tx_buffer[..], connection.destination_addr, t_s);
        self.transmit().await;
    }

    #[allow(dead_code)]
    pub async fn transmit_message_sc(&mut self) {
        self.tx_len = message_sc(&mut self.tx_buffer[..]);
        self.transmit().await;
    }

    pub async fn transmit(&mut self) {
        if 0 != self.config.t_sdr_min {
            self.stream_state = StreamState::WaitMinTsdr;
            let baudrate = self.hw_interface.get_baudrate();
            self.timer_timeout_in_us = (1_000_000u32 * u32::from(self.config.t_sdr_min)) / baudrate;
            self.hw_interface.wait_for(self.timer_timeout_in_us).await;
        }

        self.stream_state = StreamState::SendData;
        self.hw_interface.wait_for_activ_transmission().await;
        self.hw_interface.tx_rs485_enable();
        let buffer = &self.tx_buffer[0..self.tx_len];
        self.hw_interface.send_uart_data(buffer).await;
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

const SAP_OFFSET: u8 = 0x80;
fn sap_active(address: u8) -> bool {
    (address & SAP_OFFSET) == SAP_OFFSET
}

pub fn message_sd1(
    buffer: &mut [u8],
    destination_addr: u8,
    source_addr: u8,
    function_code: u8,
) -> usize {
    buffer[0] = cmd_type::SD1;
    buffer[1] = destination_addr;
    buffer[2] = source_addr;
    buffer[3] = function_code;
    buffer[4] = calc_checksum(&buffer[1..4]);
    buffer[5] = cmd_type::ED;
    6
}

pub fn message_sd2(
    buffer: &mut [u8],
    destination_addr: u8,
    source_addr: u8,
    function_code: u8,
    sap: Option<(u8, u8)>,
    pdu: &[u8],
) -> usize {
    buffer[0] = cmd_type::SD2;
    buffer[1] = 3 + pdu.len().to_le_bytes()[0];
    buffer[2] = 3 + pdu.len().to_le_bytes()[0];
    buffer[3] = cmd_type::SD2;
    buffer[4] = destination_addr;
    buffer[5] = source_addr;
    buffer[6] = function_code;
    if let Some(sap) = sap {
        buffer[4] += 0x80;
        buffer[5] += 0x80;
        buffer[7] = sap.0;
        buffer[8] = sap.1;
        if pdu.len() > 0 {
            for i in 0..pdu.len() {
                buffer[9 + i] = pdu[i];
            }
        }
        let checksum = calc_checksum(&buffer[4..9 + pdu.len()]);
        buffer[9 + pdu.len()] = checksum;
        buffer[10 + pdu.len()] = cmd_type::ED;
        return 11 + pdu.len();
    } else {
        if pdu.len() > 0 {
            for i in 0..pdu.len() {
                buffer[7 + i] = pdu[i];
            }
        }
        let checksum = calc_checksum(&buffer[4..7 + pdu.len()]);
        buffer[7 + pdu.len()] = checksum;
        buffer[8 + pdu.len()] = cmd_type::ED;
        return 9 + pdu.len();
    }
}

#[allow(dead_code)]
pub fn message_sd3(
    buffer: &mut [u8],
    destination_addr: u8,
    source_addr: u8,
    function_code: u8,
    sap: Option<(u8, u8)>,
    pdu: &[u8],
) -> usize {
    buffer[0] = cmd_type::SD3;
    buffer[1] = destination_addr;
    buffer[2] = source_addr;
    buffer[3] = function_code;
    if let Some(sap) = sap {
        buffer[2] += 0x80;
        buffer[3] += 0x80;
        buffer[4] = sap.0;
        buffer[5] = sap.1;
        for i in 0..pdu.len() {
            buffer[6 + i] = pdu[i];
        }
    } else {
        for i in 0..pdu.len() {
            buffer[4 + i] = pdu[i];
        }
    }
    buffer[12] = calc_checksum(&buffer[1..12]);
    buffer[13] = cmd_type::ED;
    14
}

#[allow(dead_code)]
pub fn message_sd4(buffer: &mut [u8], destination_addr: u8, source_addr: u8) -> usize {
    buffer[0] = cmd_type::SD4;
    buffer[1] = destination_addr;
    buffer[2] = source_addr;
    3
}

pub fn message_sc(buffer: &mut [u8]) -> usize {
    buffer[0] = cmd_type::SC;
    1
}
