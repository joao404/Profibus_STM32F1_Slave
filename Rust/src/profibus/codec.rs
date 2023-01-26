/*********************************************************************
 * Profibus FDL
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

use super::types::cmd_type;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum UartAccess {
    SingleByte,
    Dma,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ReceiveHandling {
    Interrupt,
    Thread,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Config {
    t_s: u8,
    t_sl: u16,
    t_sdr_min: u16,
    rx_handling: UartAccess,
    tx_handling: UartAccess,
    receive_handling: ReceiveHandling,
}

impl Config {
    pub fn t_s(mut self, t_s: u8) -> Self {
        self.t_s = t_s;
        self
    }

    pub fn t_sl(mut self, t_sl: u16) -> Self {
        self.t_sl = t_sl;
        self
    }

    pub fn t_sdr_min(mut self, t_sdr_min: u16) -> Self {
        self.t_sdr_min = t_sdr_min;
        self
    }

    pub fn rx_handling(mut self, rx_handling: UartAccess) -> Self {
        self.rx_handling = rx_handling;
        self
    }

    pub fn tx_handling(mut self, tx_handling: UartAccess) -> Self {
        self.tx_handling = tx_handling;
        self
    }

    pub fn receive_handling(mut self, receive_handling: ReceiveHandling) -> Self {
        self.receive_handling = receive_handling;
        self
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            t_s: 126,
            t_sl: 65000,
            t_sdr_min: 20,
            rx_handling: UartAccess::SingleByte,
            tx_handling: UartAccess::SingleByte,
            receive_handling: ReceiveHandling::Interrupt,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum StreamState {
    WaitSyn,
    WaitData,
    GetData,
    HandleData,
    WaitMinTsdr,
    SendData,
}

const SAP_OFFSET: u8 = 128;
const BROADCAST_ADD: u8 = 127;
const DEFAULT_ADD: u8 = 126;

pub struct CodecVariables {
    pub config: Config,

    pub rx_len: usize,
    pub tx_len: usize,
    pub tx_pos: usize,

    pub stream_state: StreamState,
    pub timeout_max_syn_time_in_us: u32,
    pub timeout_max_rx_time_in_us: u32,
    pub timeout_max_tx_time_in_us: u32,
    pub timeout_max_sdr_time_in_us: u32,

    pub timer_timeout_in_us: u32,
}

impl CodecVariables {
    // fn new() -> Self{
    //     let mut instance = CodecVariables::default();

    //     instance
    // }
}

impl Default for CodecVariables {
    fn default() -> CodecVariables {
        CodecVariables {
            config: Config::default(),
            rx_len: 0,
            tx_len: 0,
            tx_pos: 0,
            stream_state: StreamState::WaitSyn,
            timeout_max_syn_time_in_us: 0xFFFFFFFF,
            timeout_max_rx_time_in_us: 0xFFFFFFFF,
            timeout_max_tx_time_in_us: 0xFFFFFFFF,
            timeout_max_sdr_time_in_us: 0xFFFFFFFF,

            timer_timeout_in_us: 0xFFFFFFFF,
        }
    }
}

#[allow(dead_code)]
pub trait Codec {
    fn codec_config(&mut self, config: &mut Config) {
        let timer_frequency = self.get_timer_frequency();
        let baudrate = self.get_baudrate();
        {
            let codec_variables = self.access_codec_variables();
            codec_variables.timeout_max_syn_time_in_us = (33 * timer_frequency) / baudrate; // 33 TBit = TSYN
            codec_variables.timeout_max_rx_time_in_us = (15 * timer_frequency) / baudrate;
            codec_variables.timeout_max_tx_time_in_us = (15 * timer_frequency) / baudrate;
            codec_variables.timeout_max_sdr_time_in_us = (15 * timer_frequency) / baudrate; // 15 Tbit = TSDR

            codec_variables.timer_timeout_in_us = codec_variables.timeout_max_syn_time_in_us;

            if (0 == config.t_s) || (config.t_s > DEFAULT_ADD) {
                config.t_s = DEFAULT_ADD;
            }

            codec_variables.rx_len = 0;
            codec_variables.tx_len = 0;
            codec_variables.tx_pos = 0;
            codec_variables.stream_state = StreamState::WaitSyn;
        }

        let timer_timeout_in_us = self.access_codec_variables().timer_timeout_in_us;
        // Timer init
        self.config_timer();
        // Pin Init
        self.config_rs485_pin();

        // Uart Init
        self.config_uart();
        self.run_timer(timer_timeout_in_us);
        self.rx_rs485_enable();
        self.activate_rx_interrupt();

        self.debug_write("Codec");
    }

    fn reset_data_stream(&mut self) {
        let codec_variables = self.access_codec_variables();
        codec_variables.rx_len = 0;
        codec_variables.stream_state = StreamState::WaitSyn;
        codec_variables.timer_timeout_in_us = codec_variables.timeout_max_syn_time_in_us;
        let timer_timeout_in_us = codec_variables.timer_timeout_in_us;
        self.run_timer(timer_timeout_in_us);
        self.rx_rs485_enable();
        self.deactivate_tx_interrupt();
    }

    fn serial_interrupt_handler(&mut self) {
        if self.is_rx_received() {
            self.rx_interrupt_handler();
        } else if self.is_tx_done() {
            self.tx_interrupt_handler();
        }
    }

    fn rx_interrupt_handler(&mut self) {
        self.stop_timer();
        loop {
            match self.get_uart_value() {
                Some(data) => {
                    if StreamState::WaitData == self.access_codec_variables().stream_state {
                        self.access_codec_variables().stream_state = StreamState::GetData;
                    }

                    if StreamState::GetData == self.access_codec_variables().stream_state {
                        let mut rx_len = self.access_codec_variables().rx_len;
                        let buf = self.get_rx_buffer();
                            buf[rx_len] = data;
                            if rx_len < buf.len() {
                                rx_len += 1;
                            }
                        self.access_codec_variables().rx_len = rx_len;
                    }
                }
                None => break,
            }
        }
        let timer_timeout_in_us = self.access_codec_variables().timer_timeout_in_us;
        self.run_timer(timer_timeout_in_us);
    }

    fn tx_interrupt_handler(&mut self) {
        self.stop_timer();
        if self.access_codec_variables().config.tx_handling == UartAccess::SingleByte {
            if self.access_codec_variables().tx_pos < self.access_codec_variables().tx_len {
                self.clear_tx_flag();
                let tx_pos = self.access_codec_variables().tx_pos;
                let buf = self.get_tx_buffer();
                    let data = buf[tx_pos];
                    self.set_uart_value(data);
                    self.access_codec_variables().tx_pos += 1;
            } else {
                self.tx_rs485_disable();
                // Alles gesendet, Interrupt wieder aus
                self.deactivate_tx_interrupt();
                // clear Flag because we are not writing to buffer
                self.clear_tx_flag();
            }
        } else if self.access_codec_variables().config.tx_handling == UartAccess::Dma {
            self.tx_rs485_disable();
            // Alles gesendet, Interrupt wieder aus
            self.deactivate_tx_interrupt();
            // clear Flag because we are not writing to buffer
            self.clear_tx_flag();
        }
        let timer_timeout_in_us = self.access_codec_variables().timer_timeout_in_us;
        self.run_timer(timer_timeout_in_us);
    }

    fn timer_interrupt_handler(&mut self) {
        self.stop_timer();
        match self.access_codec_variables().stream_state {
            StreamState::WaitSyn => {
                self.access_codec_variables().stream_state = StreamState::WaitData;
                self.access_codec_variables().rx_len = 0;
                self.rx_rs485_enable(); // Auf Receive umschalten
                self.access_codec_variables().timer_timeout_in_us =
                    self.access_codec_variables().timeout_max_sdr_time_in_us;
            }
            StreamState::GetData => {
                self.access_codec_variables().timer_timeout_in_us =
                    self.access_codec_variables().timeout_max_syn_time_in_us;
                self.deactivate_rx_interrupt();
                if self.access_codec_variables().config.receive_handling
                    == ReceiveHandling::Interrupt
                {
                    self.access_codec_variables().stream_state = StreamState::WaitSyn;
                    self.handle_data_receive();
                } else if self.access_codec_variables().config.receive_handling
                    == ReceiveHandling::Thread
                {
                    self.access_codec_variables().stream_state = StreamState::HandleData;
                    self.schedule_receive_handling();
                }
            }
            StreamState::WaitMinTsdr => {
                self.access_codec_variables().stream_state = StreamState::SendData;
                self.access_codec_variables().timer_timeout_in_us =
                    self.access_codec_variables().timeout_max_tx_time_in_us;
                self.wait_for_activ_transmission();
                self.tx_rs485_enable();
                self.clear_tx_flag();
                if self.access_codec_variables().config.tx_handling == UartAccess::SingleByte {
                    let mut tx_pos = self.access_codec_variables().tx_pos;
                    let buf = self.get_tx_buffer();
                        let data = buf[tx_pos];
                        self.set_uart_value(data);
                        self.activate_tx_interrupt();
                        tx_pos += 1;
                    self.access_codec_variables().tx_pos = tx_pos;
                } else if self.access_codec_variables().config.tx_handling == UartAccess::Dma {
                    let tx_len = self.access_codec_variables().tx_len;
                    self.send_uart_data(tx_len);
                    self.activate_tx_interrupt();
                }
            }
            StreamState::SendData => {
                self.access_codec_variables().stream_state = StreamState::WaitSyn;
                self.access_codec_variables().timer_timeout_in_us =
                    self.access_codec_variables().timeout_max_syn_time_in_us;
                self.rx_rs485_enable();
            }
            _ => (),
        }
        let timer_timeout_in_us = self.access_codec_variables().timer_timeout_in_us;
        self.run_timer(timer_timeout_in_us);
    }

    fn transmit_message_sd1(&mut self, destination_addr: u8, function_code: u8, sap_offset: bool) {
        let t_s = self.access_codec_variables().config.t_s;
        let buf = self.get_tx_buffer();
            buf[0] = cmd_type::SD1;
            buf[1] = destination_addr;
            buf[2] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
            buf[3] = function_code;
            buf[4] = calc_checksum(&buf[1..4]);
            buf[5] = cmd_type::ED;
            self.access_codec_variables().tx_len = 6;
            self.transmit();
    }

    fn transmit_message_sd2(
        &mut self,
        destination_addr: u8,
        function_code: u8,
        sap_offset: bool,
        pdu1: &[u8],
        pdu2: &[u8],
    ) {
        let t_s = self.access_codec_variables().config.t_s;
        let buf = self.get_tx_buffer();
            buf[0] = cmd_type::SD2;
            buf[1] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
            buf[2] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
            buf[3] = cmd_type::SD2;
            buf[4] = destination_addr;
            buf[5] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
            buf[6] = function_code;
            if pdu1.len() > 0 {
                for i in 0..pdu1.len() {
                    buf[7 + i] = pdu1[i];
                }
            }
            if pdu2.len() > 0 {
                for i in 0..pdu2.len() {
                    buf[7 + i + pdu1.len()] = pdu2[i];
                }
            }
            let checksum = calc_checksum(&buf[4..7]) + calc_checksum(pdu1) + calc_checksum(pdu2);
            buf[7 + pdu1.len() + pdu2.len()] = checksum;
            buf[8 + pdu1.len() + pdu2.len()] = cmd_type::ED;
            self.access_codec_variables().tx_len = 9 + pdu1.len() + pdu2.len();
            self.transmit();
    }

    #[allow(dead_code)]
    fn transmit_message_sd3(
        &mut self,
        destination_addr: u8,
        function_code: u8,
        sap_offset: bool,
        pdu: &[u8; 8],
    ) {
        let t_s = self.access_codec_variables().config.t_s;
        let buf = self.get_tx_buffer();
            buf[0] = cmd_type::SD3;
            buf[1] = destination_addr;
            buf[2] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
            buf[3] = function_code;
            for i in 0..pdu.len() {
                buf[4 + i] = pdu[i];
            }
            buf[12] = calc_checksum(&buf[1..12]);
            buf[13] = cmd_type::ED;
            self.access_codec_variables().tx_len = 14;
            self.transmit();
    }

    #[allow(dead_code)]
    fn transmit_message_sd4(&mut self, destination_addr: u8, sap_offset: bool) {
        let t_s = self.access_codec_variables().config.t_s;
        let buf = self.get_tx_buffer();
            buf[0] = cmd_type::SD4;
            buf[1] = destination_addr;
            buf[2] = t_s + if sap_offset { SAP_OFFSET } else { 0 };
            self.access_codec_variables().tx_len = 3;
            self.transmit();
    }

    fn transmit_message_sc(&mut self) {
        let buf = self.get_tx_buffer();
            buf[0] = cmd_type::SC;
            self.access_codec_variables().tx_len = 1;
            self.transmit();
    }

    fn transmit(&mut self) {
        self.stop_timer();
        self.access_codec_variables().tx_pos = 0;
        if 0 != self.access_codec_variables().config.t_sdr_min {
            self.access_codec_variables().stream_state = StreamState::WaitMinTsdr;
            let timer_frequency = self.get_timer_frequency();
            let baudrate = self.get_baudrate();
            self.access_codec_variables().timer_timeout_in_us = (timer_frequency
                * u32::from(self.access_codec_variables().config.t_sdr_min))
                / baudrate
                / 2u32;
            let timer_timeout_in_us = self.access_codec_variables().timer_timeout_in_us;
            self.run_timer(timer_timeout_in_us);
        } else {
            self.access_codec_variables().stream_state = StreamState::SendData;
            self.wait_for_activ_transmission();
            self.access_codec_variables().timer_timeout_in_us =
                self.access_codec_variables().timeout_max_tx_time_in_us;
            // activate Send Interrupt
            self.tx_rs485_enable();
            self.clear_tx_flag();
            if self.access_codec_variables().config.tx_handling == UartAccess::SingleByte {
                let mut tx_pos = self.access_codec_variables().tx_pos;
                let buf = self.get_tx_buffer();
                let data = buf[tx_pos];
                self.set_uart_value(data);
                self.activate_tx_interrupt();
                tx_pos += 1;
                self.access_codec_variables().tx_pos = tx_pos;
                let timer_timeout_in_us = self.access_codec_variables().timer_timeout_in_us;
                self.run_timer(timer_timeout_in_us);
            } else if self.access_codec_variables().config.tx_handling == UartAccess::Dma {
                let tx_len = self.access_codec_variables().tx_len;
                self.send_uart_data(tx_len);
                self.activate_tx_interrupt();
            }
        }
    }

    fn handle_data_receive(&mut self) {
        let mut response = false;

        let mut source_addr: u8 = 0;
        let mut destination_addr: u8 = 0;
        let mut function_code: u8 = 0;
        let mut pdu_len: u8 = 0;

        let rx_len = self.access_codec_variables().rx_len;
        let t_s = self.access_codec_variables().config.t_s;
        let buf = self.get_rx_buffer();
        match buf[0] {
            cmd_type::SD1 => {
                if 6 == rx_len {
                    if cmd_type::ED == buf[5] {
                        destination_addr = buf[1];
                        source_addr = buf[2];
                        function_code = buf[3];
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
                                );
                            }
                        }
                    }
                }
            }

            cmd_type::SD2 => {
                if rx_len > 4 {
                    if rx_len == usize::from(buf[1] + 6) {
                        if cmd_type::ED == buf[rx_len - 1] {
                            pdu_len = buf[1]; // DA+SA+FC+Nutzdaten
                            destination_addr = buf[4];
                            source_addr = buf[5];
                            function_code = buf[6];
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
                                    );
                                }
                            }
                        }
                    }
                }
            }

            cmd_type::SD3 => {
                if 14 == rx_len {
                    if cmd_type::ED == buf[13] {
                        pdu_len = 11; // DA+SA+FC+Nutzdaten
                        destination_addr = buf[1];
                        source_addr = buf[2];
                        function_code = buf[3];
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
                                );
                            }
                        }
                    }
                }
            }

            cmd_type::SD4 => {
                if 3 == rx_len {
                    destination_addr = buf[1];
                    source_addr = buf[2];

                    if check_destination_addr(
                        self.access_codec_variables().config.t_s,
                        destination_addr,
                    ) {
                        //TODO
                    }
                }
            }

            _ => (),
        } // match self.buffer[0]
        if !response {
            self.reset_data_stream();
        }
        self.activate_rx_interrupt();
    }

    fn fdl_handle_data(
        &mut self,
        source_addr: u8,
        destination_addr: u8,
        function_code: u8,
        pdu: &[u8],
    ) -> bool;

    fn access_codec_variables(&mut self) -> &mut CodecVariables;

    fn config_timer(&mut self);

    fn run_timer(&mut self, _timeout_in_us: u32);

    fn stop_timer(&mut self);

    fn clear_overflow_flag(&mut self);

    fn config_uart(&mut self);

    fn activate_tx_interrupt(&mut self);

    fn deactivate_tx_interrupt(&mut self);

    fn activate_rx_interrupt(&mut self);

    fn deactivate_rx_interrupt(&mut self);

    fn activate_idle_interrupt(&mut self);

    fn deactivate_idle_interrupt(&mut self);

    fn set_tx_flag(&mut self);

    fn clear_tx_flag(&mut self);

    fn clear_rx_flag(&mut self);

    fn clear_idle_flag(&mut self);

    fn wait_for_activ_transmission(&mut self);

    fn is_rx_received(&mut self) -> bool;

    fn is_rx_idle(&mut self) -> bool;

    fn is_tx_done(&mut self) -> bool;

    fn tx_rs485_enable(&mut self);

    fn tx_rs485_disable(&mut self);

    fn rx_rs485_enable(&mut self);

    fn config_rs485_pin(&mut self);

    fn get_uart_value(&mut self) -> Option<u8>;

    fn set_uart_value(&mut self, _value: u8);

    fn send_uart_data(&mut self, _len: usize);

    fn get_uart_data(&mut self) -> usize;

    fn schedule_receive_handling(&mut self);

    fn get_rx_buffer(&mut self) -> &mut [u8];

    fn get_tx_buffer(&mut self) -> &mut [u8];

    fn get_timer_frequency(&self) -> u32;

    fn get_baudrate(&self) -> u32;

    fn debug_write(&mut self, _debug: &str);
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

extern "Rust" {
    fn fdl_handle_data(
        source_addr: u8,
        destination_addr: u8,
        function_code: u8,
        pdu: &[u8],
    ) -> bool;
}
