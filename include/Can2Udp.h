/*********************************************************************
 * Can2Udp
 *
 * Copyright (C) 2022 Marcel Maage
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

#pragma once

#include <Arduino.h>
#include <AsyncUDP.h>
#include "trainBoxMaerklin/CanInterface.h"
#include "Helper/Observer.h"

class Can2Udp : public Observer
{
    public:
        Can2Udp(CanInterface& canInterface, bool debug);
        ~Can2Udp();
        void begin(int localPort = 15731, int destinationPort = 15730);
    private:
        bool m_debug;

        void update(Observable& observable, void* data);

        void handleUdpPacket(uint8_t *udpframe, size_t size);

        CanInterface& m_canInterface;

        int m_localPort;
    int m_destinationPort;

        AsyncUDP m_udpInterface;
};