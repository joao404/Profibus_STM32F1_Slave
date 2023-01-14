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

use super::hw_interface::HwInterface;
use super::types::{cmd_type, fc_request, fc_response};

#[derive(PartialEq, Eq)]
pub enum UartAccess {
    SingleByte,
    Dma,
}

#[derive(PartialEq, Eq)]
pub enum ReceiveHandling {
    Interrupt,
    Thread,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct Fdl<Serial, const BUF_SIZE: usize> {
    config: Config,
    interface: Serial,
    tx_buffer: [u8; BUF_SIZE],
    rx_buffer: [u8; BUF_SIZE],

    rx_len: usize,
    tx_len: usize,
    tx_pos: usize,

    stream_state: StreamState,
    timeout_max_syn_time_in_us: u32,
    timeout_max_rx_time_in_us: u32,
    timeout_max_tx_time_in_us: u32,
    timeout_max_sdr_time_in_us: u32,

    timer_timeout_in_us: u32,
}

impl<Serial, const BUF_SIZE: usize> Fdl<Serial, BUF_SIZE>
where
    Serial: HwInterface,
{
    pub fn new(mut interface: Serial, mut config: Config) -> Self {
        let timeout_max_syn_time_in_us =
            (33 * interface.get_timer_frequency()) / interface.get_baudrate(); // 33 TBit = TSYN
        let timeout_max_rx_time_in_us =
            (15 * interface.get_timer_frequency()) / interface.get_baudrate();
        let timeout_max_tx_time_in_us =
            (15 * interface.get_timer_frequency()) / interface.get_baudrate();
        let timeout_max_sdr_time_in_us =
            (15 * interface.get_timer_frequency()) / interface.get_baudrate(); // 15 Tbit = TSDR

        if (0 == config.t_s) || (config.t_s > DEFAULT_ADD) {
            config.t_s = DEFAULT_ADD;
        }

        // Timer init
        interface.config_timer();
        // Pin Init
        interface.config_rs485_pin();

        // Uart Init
        interface.config_uart();
        interface.run_timer(timeout_max_syn_time_in_us);
        interface.rx_rs485_enable();
        interface.activate_rx_interrupt();

        interface.debug_write("FDL");

        Self {
            config,
            interface,
            tx_buffer: [0; BUF_SIZE],
            rx_buffer: [0; BUF_SIZE],
            rx_len: 0,
            tx_len: 0,
            tx_pos: 0,
            stream_state: StreamState::WaitSyn,
            timeout_max_syn_time_in_us,
            timeout_max_rx_time_in_us,
            timeout_max_tx_time_in_us,
            timeout_max_sdr_time_in_us,
            timer_timeout_in_us: timeout_max_syn_time_in_us,
        }
    }

    pub fn get_TS(&self) -> u8 {
        self.config.t_s
    }

    pub fn set_TS(&mut self, TS: u8) {
        if TS < BROADCAST_ADD {
            self.config.t_s = TS;
        } else {
            self.interface.debug_write("E:TS>126");
        }
    }

    pub fn get_baudrate(&self) -> u32 {
        self.interface.get_baudrate()
    }

    pub fn get_t_sl(&self) -> u16 {
        self.config.t_sl
    }

    pub fn set_t_sl(&mut self, t_sl: u16) {
        if 52 >= t_sl {
            self.config.t_sl = t_sl;
        } else {
            self.interface.debug_write("E:Tsl<52");
        }
    }

    pub fn get_t_sdr_min(&self) -> u16 {
        self.config.t_sdr_min
    }

    pub fn set_t_sdr_min(&mut self, t_sdr_min: u16) {
        if 20 >= t_sdr_min {
            self.config.t_sdr_min = t_sdr_min;
        } else {
            self.interface.debug_write("E:Tsdr<20");
        }
    }

    fn reset_data_stream(&mut self) {
        self.rx_len = 0;
        self.stream_state = StreamState::WaitSyn;
        self.timer_timeout_in_us = self.timeout_max_syn_time_in_us;
        self.interface.run_timer(self.timer_timeout_in_us);
        self.interface.rx_rs485_enable();
        self.interface.deactivate_tx_interrupt();
    }

    pub fn serial_interrupt_handler(&mut self) {
        if self.interface.is_rx_received() {
            self.rx_interrupt_handler();
        } else if self.interface.is_tx_done() {
            self.tx_interrupt_handler();
        }
    }

    pub fn rx_interrupt_handler(&mut self) {
        self.interface.stop_timer();
        loop {
            match self.interface.get_uart_value() {
                Some(data) => {
                    if StreamState::WaitData == self.stream_state {
                        self.stream_state = StreamState::GetData;
                    }

                    if StreamState::GetData == self.stream_state {
                        self.rx_buffer[self.rx_len] = data;
                        if self.rx_len < self.rx_buffer.len() {
                            self.rx_len += 1;
                        }
                    }
                }
                None => break,
            }
        }
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn tx_interrupt_handler(&mut self) {
        self.interface.stop_timer();
        if self.config.tx_handling == UartAccess::SingleByte {
            if self.tx_pos < self.tx_len {
                self.interface.clear_tx_flag();
                self.interface.set_uart_value(self.tx_buffer[self.tx_pos]);
                self.tx_pos += 1;
            } else {
                self.interface.tx_rs485_disable();
                // Alles gesendet, Interrupt wieder aus
                self.interface.deactivate_tx_interrupt();
                // clear Flag because we are not writing to buffer
                self.interface.clear_tx_flag();
            }
        } else if self.config.tx_handling == UartAccess::Dma {
            self.interface.tx_rs485_disable();
            // Alles gesendet, Interrupt wieder aus
            self.interface.deactivate_tx_interrupt();
            // clear Flag because we are not writing to buffer
            self.interface.clear_tx_flag();
        }
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    pub fn timer_interrupt_handler(&mut self) {
        self.interface.stop_timer();
        match self.stream_state {
            StreamState::WaitSyn => {
                self.stream_state = StreamState::WaitData;
                self.rx_len = 0;
                self.interface.rx_rs485_enable(); // Auf Receive umschalten
                self.timer_timeout_in_us = self.timeout_max_sdr_time_in_us;
            }
            StreamState::GetData => {
                self.timer_timeout_in_us = self.timeout_max_syn_time_in_us;
                self.interface.deactivate_rx_interrupt();
                if self.config.receive_handling == ReceiveHandling::Interrupt {
                    self.stream_state = StreamState::WaitSyn;
                    self.handle_data_receive();
                } else if self.config.receive_handling == ReceiveHandling::Thread {
                    self.stream_state = StreamState::HandleData;
                    self.interface.schedule_receive_handling();
                }
            }
            StreamState::WaitMinTsdr => {
                self.stream_state = StreamState::SendData;
                self.timer_timeout_in_us = self.timeout_max_tx_time_in_us;
                self.interface.wait_for_activ_transmission();
                self.interface.tx_rs485_enable();
                self.interface.clear_tx_flag();
                if self.config.tx_handling == UartAccess::SingleByte {
                    self.interface.set_uart_value(self.tx_buffer[self.tx_pos]);
                    self.interface.activate_tx_interrupt();
                    self.tx_pos += 1;
                    self.interface.run_timer(self.timer_timeout_in_us);
                } else if self.config.tx_handling == UartAccess::Dma {
                    self.interface.send_uart_data(&self.tx_buffer);
                    self.interface.activate_tx_interrupt();
                }
            }
            StreamState::SendData => {
                self.stream_state = StreamState::WaitSyn;
                self.timer_timeout_in_us = self.timeout_max_syn_time_in_us;
                self.interface.rx_rs485_enable();
            }
            _ => (),
        }
        self.interface.run_timer(self.timer_timeout_in_us);
    }

    fn transmit_message_sd1(&mut self, source_addr: u8, function_code: u8, sap_offset: bool) {
        self.tx_buffer[0] = cmd_type::SD1;
        self.tx_buffer[1] = source_addr;
        self.tx_buffer[2] = self.config.t_s + if sap_offset { SAP_OFFSET } else { 0 };
        self.tx_buffer[3] = function_code;
        let checksum = self.calc_checksum(&self.tx_buffer[1..4]);
        self.tx_buffer[4] = checksum;
        self.tx_buffer[5] = cmd_type::ED;
        self.tx_len = 6;
        self.transmit();
    }

    fn transmit_message_sd2(
        &mut self,
        source_addr: u8,
        function_code: u8,
        sap_offset: bool,
        pdu1: &[u8],
        pdu2: &[u8],
    ) {
        self.tx_buffer[0] = cmd_type::SD2;
        self.tx_buffer[1] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
        self.tx_buffer[2] = 3 + pdu1.len().to_le_bytes()[0] + pdu2.len().to_le_bytes()[0];
        self.tx_buffer[3] = cmd_type::SD2;
        self.tx_buffer[4] = source_addr;
        self.tx_buffer[5] = self.config.t_s + if sap_offset { SAP_OFFSET } else { 0 };
        self.tx_buffer[6] = function_code;
        if pdu1.len() > 0 {
            for i in 0..pdu1.len() {
                self.tx_buffer[7 + i] = pdu1[i];
            }
        }
        if pdu2.len() > 0 {
            for i in 0..pdu2.len() {
                self.tx_buffer[7 + i + pdu1.len()] = pdu2[i];
            }
        }
        let checksum = self.calc_checksum(&self.tx_buffer[4..7])
            + self.calc_checksum(pdu1)
            + self.calc_checksum(pdu2);
        self.tx_buffer[7 + pdu1.len() + pdu2.len()] = checksum;
        self.tx_buffer[8 + pdu1.len() + pdu2.len()] = cmd_type::ED;
        self.tx_len = 9 + pdu1.len() + pdu2.len();
        self.transmit();
    }

    #[allow(dead_code)]
    fn transmit_message_sd3(&mut self, source_addr: u8, function_code: u8, sap_offset: bool, pdu: &[u8; 8]) {
        self.tx_buffer[0] = cmd_type::SD3;
        self.tx_buffer[1] = source_addr;
        self.tx_buffer[2] = self.config.t_s + if sap_offset { SAP_OFFSET } else { 0 };
        self.tx_buffer[3] = function_code;
        for i in 0..pdu.len() {
            self.tx_buffer[4 + i] = pdu[i];
        }
        let checksum = self.calc_checksum(&self.tx_buffer[1..12]);
        self.tx_buffer[12] = checksum;
        self.tx_buffer[13] = cmd_type::ED;
        self.tx_len = 14;
        self.transmit();
    }

    #[allow(dead_code)]
    fn transmit_message_sd4(&mut self, source_addr : u8, sap_offset: bool) {
        self.tx_buffer[0] = cmd_type::SD4;
        self.tx_buffer[1] = source_addr;
        self.tx_buffer[2] = self.config.t_s + if sap_offset { SAP_OFFSET } else { 0 };
        self.tx_len = 3;
        self.transmit();
    }

    fn transmit_message_sc(&mut self) {
        self.tx_buffer[0] = cmd_type::SC;
        self.tx_len = 1;
        self.transmit();
    }

    fn transmit(&mut self) {
        self.interface.stop_timer();
        self.tx_pos = 0;
        if 0 != self.config.t_sdr_min {
            self.stream_state = StreamState::WaitMinTsdr;
            self.timer_timeout_in_us = (self.interface.get_timer_frequency()
                * u32::from(self.config.t_sdr_min))
                / self.interface.get_baudrate()
                / 2u32;
            self.interface.run_timer(self.timer_timeout_in_us);
        } else {
            self.stream_state = StreamState::SendData;
            self.interface.wait_for_activ_transmission();
            self.timer_timeout_in_us = self.timeout_max_tx_time_in_us;
            // activate Send Interrupt
            self.interface.tx_rs485_enable();
            self.interface.clear_tx_flag();
            if self.config.tx_handling == UartAccess::SingleByte {
                self.interface.set_uart_value(self.tx_buffer[self.tx_pos]);
                self.interface.activate_tx_interrupt();
                self.tx_pos += 1;
                self.interface.run_timer(self.timer_timeout_in_us);
            } else if self.config.tx_handling == UartAccess::Dma {
                self.interface.send_uart_data(&self.tx_buffer);
                self.interface.activate_tx_interrupt();
            }
        }
    }

    fn calc_checksum(&self, data: &[u8]) -> u8 {
        let mut checksum: u8 = 0;
        for x in data {
            checksum += *x;
        }
        checksum
    }

    fn check_destination_addr(&self, destination: u8) -> bool {
        if ((destination & 0x7F) != self.config.t_s) &&  // Slave
       ((destination & 0x7F) != BROADCAST_ADD)
        // Broadcast
        {
            false
        } else {
            true
        }
    }

    pub fn handle_data_receive(&mut self) {
        let mut process_data = false;
        let mut response = false;

        // Profibus Datentypen
        let mut destination_addr: u8 = 0;
        let mut source_addr: u8 = 0;
        let mut function_code: u8 = 0;
        let mut pdu_len: u8 = 0; // PDU Groesse

        match self.rx_buffer[0] {
            cmd_type::SD1 => {
                if 6 == self.rx_len {
                    if cmd_type::ED == self.rx_buffer[5] {
                        destination_addr = self.rx_buffer[1];
                        source_addr = self.rx_buffer[2];
                        function_code = self.rx_buffer[3];
                        let fcs_data = self.rx_buffer[4]; // Frame Check Sequence

                        if self.check_destination_addr(destination_addr) {
                            if fcs_data == self.calc_checksum(&self.rx_buffer[1..4]) {
                                // FCV und FCB loeschen, da vorher überprüft
                                function_code &= 0xCF;
                                process_data = true;
                            }
                        }
                    }
                }
            }

            cmd_type::SD2 => {
                if self.rx_len > 4 {
                    if self.rx_len == usize::from(self.rx_buffer[1] + 6) {
                        if cmd_type::ED == self.rx_buffer[self.rx_len - 1] {
                            pdu_len = self.rx_buffer[1]; // DA+SA+FC+Nutzdaten
                            destination_addr = self.rx_buffer[4];
                            source_addr = self.rx_buffer[5];
                            function_code = self.rx_buffer[6];
                            let fcs_data = self.rx_buffer[usize::from(pdu_len + 4)]; // Frame Check Sequence
                            if self.check_destination_addr(destination_addr) {
                                if fcs_data
                                    == self.calc_checksum(
                                        &self.rx_buffer[4..usize::from(self.rx_len - 2)],
                                    )
                                {
                                    // FCV und FCB loeschen, da vorher überprüft
                                    function_code &= 0xCF;
                                    process_data = true;
                                }
                            }
                        }
                    }
                }
            }

            cmd_type::SD3 => {
                if 14 == self.rx_len {
                    if cmd_type::ED == self.rx_buffer[13] {
                        pdu_len = 11; // DA+SA+FC+Nutzdaten
                        destination_addr = self.rx_buffer[1];
                        source_addr = self.rx_buffer[2];
                        function_code = self.rx_buffer[3];
                        let fcs_data = self.rx_buffer[12]; // Frame Check Sequence

                        if self.check_destination_addr(destination_addr) {
                            if fcs_data == self.calc_checksum(&self.rx_buffer[1..12]) {
                                // FCV und FCB loeschen, da vorher überprüft
                                function_code &= 0xCF;
                                process_data = true;
                            }
                        }
                    }
                }
            }

            cmd_type::SD4 => {
                if 3 == self.rx_len {
                    destination_addr = self.rx_buffer[1];
                    source_addr = self.rx_buffer[2];

                    if self.check_destination_addr(destination_addr) {
                        //TODO
                    }
                }
            }

            _ => (),
        } // match self.buffer[0]

        if process_data {}
        // data could not be handled
        if !response {
            self.reset_data_stream();
        }
        self.interface.activate_rx_interrupt();
    }
}
