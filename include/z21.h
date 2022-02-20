/*********************************************************************
 * z21
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

#include "trainBoxMaerklin/TrainBoxMaerklinEsp32.h"
#include "z21/z21InterfaceEsp32.h"
#include <unordered_map>
#include <list>
#include "Preferences.h"

class z21 : public TrainBoxMaerklinEsp32, private z21InterfaceEsp32
{
public:
    z21(CanInterface& canInterface, HwType hwType, uint16_t swVersion, int16_t port, uint16_t hash, bool debug);
    ~z21();
    void begin();
    void cyclic();

    struct DataLoco
    {
        uint16_t adr;
        uint8_t mode;
        bool isActive;
        unsigned long lastSpeedCmdTimeINms;
        std::array<uint8_t, 6> data;
    };

    struct ConfigLoco
    {
        uint16_t adr;
        uint8_t mode;
        uint8_t steps;
    };

private:
    const uint32_t z21Uid{0xBADEAFFE};

    uint32_t m_trainBoxUid;

    uint16_t m_hwIdent;

    Preferences m_preferences;

    const char* m_namespaceZ21 {"z21"};

    const char* m_keyLocoMode {"locomode"};

    const char* m_keyTurnOutMode {"turnoutmode"};

    const char* m_keyConfig1 {"config1"};

    const char* m_keyConfig2 {"config2"};

    const size_t m_maxNumberOfLoco{256};

    const uint16_t m_longDccAddressStart {128};

    std::list<DataLoco> m_locos;
    
    const size_t m_maxNumberOfTurnout{1024};

    const uint16_t m_startAdressAccDCC {1000};

    std::unordered_map<uint16_t, uint8_t> m_turnouts;

    //1024 adresses => 1024/8
    //std::array<uint8_t, 32> turnOutMode;
    uint8_t m_turnOutMode[128];

    bool directProgramming;

    const unsigned long minimumCmdIntervalINms {100};


    uint16_t m_currentINmA = 0;
    uint16_t m_voltageINmV = 0;
    uint16_t m_tempIN10_2deg = 0;

    void saveLocoConfig();

    // true if emergency stop is activ
    bool calcSpeedZ21toTrainbox(uint8_t data, uint8_t speedConfig, uint8_t& speed);

    void calcSpeedTrainboxToZ21(uint8_t speed, uint8_t speedConfig, uint8_t &data);

    void notifyLocoState(uint8_t client, uint16_t Adr, std::array<uint8_t, 6> &locoData);

    bool getConfig1(std::array<uint8_t, 10>& config) override;

	void setConfig1(std::array<uint8_t, 10>& config) override;

	bool getConfig2(std::array<uint8_t, 16>& config) override;

	void setConfig2(std::array<uint8_t, 16>& config) override;

    uint16_t getSerialNumber() override;

    // onCallback
    bool onSystemStop(uint32_t id) override;

    bool onSystemGo(uint32_t id) override;

    bool onSystemHalt(uint32_t id) override;

    bool onLocoStop(uint32_t id) override;

    bool onLocoRemoveCycle(uint32_t id) override;

    bool onLocoDataProtocol(uint32_t id, ProtocolLoco protocol) override;

    bool onAccTime(uint32_t id, uint16_t accTimeIN10ms) override;

    bool onFastReadMfx(uint32_t id, uint16_t mfxSid) override;

    bool onTrackProtocol(uint32_t id, uint8_t param) override;

    bool onMfxCounter(uint32_t id, uint16_t counter) override;

    bool onSystemOverLoad(uint32_t id, uint8_t channel) override;

    bool onSystemStatus(uint32_t id, uint8_t channel, bool valid) override;

    bool onSystemStatus(uint32_t id, uint8_t channel, uint16_t value) override;

    bool onSystemIdent(uint32_t id, uint16_t feedbackId) override;

    bool onSystemReset(uint32_t id, uint8_t target) override;

    bool onLocoSpeed(uint32_t id) override;

    bool onLocoSpeed(uint32_t id, uint16_t speed) override;

    // 0 = Fahrtrichtung bleibt
    // 1 = Fahrtrichtung vorwärts
    // 2 = Fahrtrichtung rückwärts
    // 3 = Fahrtrichtung umschalten
    bool onLocoDir(uint32_t id, uint8_t dir) override;

    bool onLocoFunc(uint32_t id, uint8_t function, uint8_t value) override;

    bool onReadConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool readSuccessful) override;

    bool onWriteConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool writeSuccessful, bool verified) override;

    bool onAccSwitch(uint32_t id, uint8_t position, uint8_t current) override;

    bool onPing(uint32_t id, uint16_t swVersion, uint16_t hwIdent) override;
    // Z21

    void handleGetLocoMode(uint16_t adr, uint8_t& mode) override;
	void handleSetLocoMode(uint16_t adr, uint8_t mode) override;
    void handleGetTurnOutMode(uint16_t adr, uint8_t& mode) override;
	void handleSetTurnOutMode(uint16_t adr, uint8_t mode) override;

    void notifyz21InterfacegetSystemInfo(uint8_t client) override;

    // void notifyz21InterfaceLNdetector(uint8_t client, uint8_t typ, uint16_t Adr) override;
    uint8_t notifyz21InterfaceLNdispatch(uint16_t Adr) override;
    void notifyz21InterfaceLNSendPacket(uint8_t *data, uint8_t length) override;

    // void notifyz21InterfaceCANdetector(uint8_t client, uint8_t typ, uint16_t ID) override;

    void notifyz21InterfaceRailPower(EnergyState State) override;

    void notifyz21InterfaceCVREAD(uint8_t cvAdrMSB, uint8_t cvAdrLSB) override;
    void notifyz21InterfaceCVWRITE(uint8_t cvAdrMSB, uint8_t cvAdrLSB, uint8_t value) override;
    void notifyz21InterfaceCVPOMWRITEBYTE(uint16_t Adr, uint16_t cvAdr, uint8_t value) override;
    void notifyz21InterfaceCVPOMWRITEBIT(uint16_t Adr, uint16_t cvAdr, uint8_t value) override;
    void notifyz21InterfaceCVPOMREADBYTE (uint16_t Adr, uint16_t cvAdr) override;

    void notifyz21InterfaceMMWRITE(uint8_t regAdr, uint8_t value) override;
	void notifyz21InterfaceDCCWRITE(uint8_t regAdr, uint8_t value) override;
	void notifyz21InterfaceDCCREAD(uint8_t regAdr) override;

    void notifyz21InterfaceAccessoryInfo(uint16_t Adr, uint8_t& position) override;
    void notifyz21InterfaceAccessory(uint16_t Adr, bool state, bool active) override;

    // void notifyz21InterfaceExtAccessory(uint16_t Adr, byte state) override;

    void notifyz21InterfaceLocoState(uint16_t Adr, uint8_t data[]) override;
    void notifyz21InterfaceLocoFkt(uint16_t Adr, uint8_t type, uint8_t fkt) override;
    void notifyz21InterfaceLocoSpeed(uint16_t Adr, uint8_t speed, uint8_t stepConfig) override;

    void notifyz21InterfaceS88Data(uint8_t gIndex) override; // return last state S88 Data for the Client!
};