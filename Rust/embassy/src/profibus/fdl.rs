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

use defmt::*;

use super::codec::{Codec, CodecConfig, Connection};
use super::codec_hw_interface::CodecHwInterface;

use super::types::FcRequestLowNibble;

#[derive(Default)]
pub struct FdlConfig {
    pub codec_config: CodecConfig,
}

#[derive(PartialEq, Eq)]
#[allow(dead_code)]
pub enum FdlService {
    SdaLow,
    SdnLow,
    SrdLow,
    SdaHigh,
    SdnHigh,
    SrdHigh,
    Msrd,
    Csrd,
    Cs,
    None,
}

#[allow(dead_code)]
pub enum FdlState {
    PowerOff,
    Idle,
}

#[allow(dead_code)]
pub enum FdlType {
    Passiv,
    Active,
}

pub struct Service<'pdu> {
    pub connection: Connection<'pdu>,
    pub service: FdlService,
}

impl<'pdu> Service<'pdu> {
    pub fn new(connection: Connection<'pdu>, service: FdlService) -> Self {
        Self {
            connection,
            service,
        }
    }
}

#[allow(dead_code)]
pub struct Fdl<CodecInterface> {
    codec: Codec<CodecInterface>,
    fdl_type: FdlType,
    fdl_state: FdlState,
}

impl<CodecInterface> Fdl<CodecInterface>
where
    CodecInterface: CodecHwInterface,
{
    pub fn new(codec_interface: CodecInterface, config: FdlConfig, fdl_type: FdlType) -> Self {
        info!("codec start");
        let codec = Codec::<CodecInterface>::new(codec_interface, config.codec_config);
        Self {
            codec,
            fdl_type,
            fdl_state: FdlState::Idle, //FdlState::PowerOff
        }
    }

    pub async fn run<'buf: 'out, 'out>(
        &'_ mut self,
        buffer: &'buf mut [u8],
    ) -> Option<Service<'out>> {
        match self.codec.receive(buffer).await {
            Some(conn) => {
                let service: FdlService = match conn.function_code & 0x0F {
                    FcRequestLowNibble::SDA_LOW => FdlService::SdaLow,
                    FcRequestLowNibble::SDN_LOW => FdlService::SdnLow,
                    FcRequestLowNibble::SRD_LOW => FdlService::SrdLow,
                    FcRequestLowNibble::SDA_HIGH => FdlService::SdaHigh,
                    FcRequestLowNibble::SDN_HIGH => FdlService::SdnHigh,
                    FcRequestLowNibble::SRD_HIGH => FdlService::SrdHigh,
                    FcRequestLowNibble::MSRD => FdlService::Msrd,
                    _ => FdlService::None,
                };
                Some(Service::new(conn, service))
            }
            None => None,
        }
    }

    pub async fn response_sdn()
    {
        
    }
}
