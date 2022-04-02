/*********************************************************************
 * Can2Lan
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

#pragma once

#include <Arduino.h>
#include <AsyncUDP.h>
#include <AsyncTCP.h>
#include <memory>
#include <list>
#include "trainBoxMaerklin/CanInterface.h"
#include "Helper/Observer.h"

// class is designed as singleton
class Can2Lan : public Observer
{
public:
    static Can2Lan* getCan2Lan();
    virtual ~Can2Lan();
    void begin(CanInterface* canInterface, bool debug, bool canDebug, int localPortUdp = 15731, int localPortTcp = 15731, int destinationPortUdp = 15730);

    static void handleNewTcpClient(void *arg, AsyncClient *client);
    static void handleTcpPacket(void *arg, AsyncClient *client, void *data, size_t len);
    static void handleError(void *arg, AsyncClient *client, int8_t error);
    static void handleDisconnect(void *arg, AsyncClient *client);
    static void handleTimeOut(void *arg, AsyncClient *client, uint32_t time);

private:
    Can2Lan();

    static Can2Lan* m_can2LanInstance;

    bool m_debug;

    bool m_canDebug;

    void update(Observable &observable, void *data);

    void handleUdpPacket(uint8_t *udpframe, size_t size);

    const uint8_t m_canFrameSize{13};

    CanInterface* m_canInterface;

    int m_localPortUdp;
    int m_localPortTcp;
    int m_destinationPortUdp;

    AsyncUDP m_udpInterface;
    std::unique_ptr<AsyncServer> m_tcpInterface;
    std::list<AsyncClient*> m_tcpClients;
};