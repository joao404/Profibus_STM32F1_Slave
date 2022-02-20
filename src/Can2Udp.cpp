/*********************************************************************
 * Can2Udp
 *
 * Copyright (C) 2022 Marcel Maage
 * 
 * based on https://github.com/GBert/railroad/blob/master/can2udp/src/can2udp.c
 * by Gerhard Bertelsmann 
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

#include "Can2Udp.h"

Can2Udp::Can2Udp(CanInterface &canInterface, bool debug)
    : m_debug(debug),
      m_canInterface(canInterface),
      m_localPort(15731),
      m_destinationPort(15730)

{
}

Can2Udp::~Can2Udp()
{
}

void Can2Udp::begin(int localPort, int destinationPort)
{
    m_localPort = localPort;
    m_destinationPort = destinationPort;
    // Udp.begin(local_port);
    if (m_udpInterface.listen(m_localPort))
    {
        m_udpInterface.onPacket([this](AsyncUDPPacket packet)
                                { handleUdpPacket(packet.data(), packet.length()); });
    }

    m_canInterface.attach(*this);
    // send magic start frame
    can_message_t frame;
    frame.identifier = 0x360301UL;
    frame.flags = CAN_MSG_FLAG_EXTD | CAN_MSG_FLAG_SS;
    frame.data_length_code = 5;
    for (int i = 0; i < frame.data_length_code; i++)
    {
        frame.data[i] = 0;
    }
    frame.data[4] = 0x11;
    if (!m_canInterface.transmit(frame, 1000u))
    {
        Serial.println("CAN magic start write error");
    }
    Serial.println("Can2Udp setup finished");
}

void Can2Udp::update(Observable &observable, void *data)
{
    if (&observable == &m_canInterface)
    {
        if (nullptr != data)
        {
            can_message_t *frame = static_cast<can_message_t *>(data);

            uint8_t udpframe[128];
            uint32_t canid = htonl(frame->identifier);
            memcpy(udpframe, &canid, 4);
            udpframe[4] = frame->data_length_code;
            memcpy(&udpframe[5], &frame->data, frame->data_length_code);

            if (m_debug)
            {
                Serial.print("CAN ");
                Serial.print(frame->identifier, HEX);
                Serial.print(" ");
                Serial.print(frame->data_length_code, HEX);
                Serial.print(" ");
                for (int i = 0; i < (frame->data_length_code); i++)
                {
                    Serial.print(udpframe[i], HEX);
                    Serial.print(" ");
                }
                Serial.print("\n");
            }
            m_udpInterface.broadcastTo(udpframe, 13, m_destinationPort); //, TCPIP_ADAPTER_IF_AP);
        }
    }
}

void Can2Udp::handleUdpPacket(uint8_t *udpframe, size_t size)
{
    /* Maerklin UDP Format: always 13 bytes
     *   byte 0-3 TWAI ID
     *   byte 4 DLC
     *   byte 5-12 TWAI data
     */
    if (size == 13)
    {
        can_message_t tx_frame;
        // Serial.println("Udp 13");
        uint32_t canid = 0;
        memcpy(&canid, &udpframe[0], 4);
        /* TWAI is stored in network Big Endian format */
        tx_frame.identifier = ntohl(canid);
        tx_frame.flags = CAN_MSG_FLAG_EXTD | CAN_MSG_FLAG_SS;
        tx_frame.data_length_code = udpframe[4];
        memcpy(&tx_frame.data, &udpframe[5], 8);
        /* send TWAI frame */
        memcpy(&tx_frame.data, &udpframe[5], 8);

        // Serial.println(frame.MsgID, HEX);
        // Serial.print(frame.data.u8[0], HEX);
        // Serial.print(frame.data.u8[1], HEX);
        // Serial.print(frame.data.u8[2], HEX);
        // Serial.print(frame.data.u8[3], HEX);
        // Serial.println(frame.data.u8[4], HEX);

        /* answer to TWAI ping from LAN to LAN */
        if (((tx_frame.identifier & 0x00FF0000UL) == 0x00310000UL) &&
            (udpframe[11] == 0xEE) && (udpframe[12] == 0xEE))
        {
            if (m_debug)
            {
                Serial.println("CAN ping");
            }
            uint8_t udpframe_reply[16];
            memcpy(udpframe_reply, udpframe, 13);
            udpframe_reply[0] = 0x00;
            udpframe_reply[1] = 0x30;
            udpframe_reply[2] = 0x00;
            udpframe_reply[3] = 0x00;
            udpframe_reply[4] = 0x00;
            // Udp.beginPacket(broadcastIp, destination_port);
            // Udp.write(udpframe_reply, 13);
            // Udp.endPacket();
            m_udpInterface.broadcastTo(udpframe_reply, 13, m_destinationPort); //, TCPIP_ADAPTER_IF_AP);
        }
        if (m_debug)
        {
            Serial.print("UDP ");
            Serial.print(tx_frame.identifier, HEX);
            Serial.print(" ");
            Serial.print(tx_frame.data_length_code, HEX);
            Serial.print(" ");
            for (int i = 0; i < (tx_frame.data_length_code); i++)
            {
                Serial.print(tx_frame.data[i], HEX);
                Serial.print(" ");
            }
            Serial.print("\n");
        }

        if (!m_canInterface.transmit(tx_frame, 1000u))
        {
            if (m_debug)
            {
                Serial.println("CAN write error");
            }
        }
    }
}