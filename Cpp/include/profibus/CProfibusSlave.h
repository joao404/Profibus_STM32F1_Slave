/*********************************************************************
 * Profibus Stm32f1 Slave
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

#include "profibus/ProfibusConstants.h"
#include <stdio.h>
#include <vector>
#include <memory>

///////////////////////////////////////////////////////////////////////////////////////////////////
// Profibus Ablaufsteuerung
///////////////////////////////////////////////////////////////////////////////////////////////////
enum class StreamStatus : uint8_t
{
    WaitSyn,
    WaitData,
    GetData,
    WaitMinTsdr,
    SendData
};

///////////////////////////////////////////////////////////////////////////////////////////////////

class CProfibusSlave
{
public:
    struct Config
    {
        uint8_t identHigh{0};
        uint8_t identLow{0};
        uint32_t counterFrequency{0};
        uint32_t speed{0};
        uint8_t bufSize{0};
        uint8_t inputDataSize{0};
        uint8_t outputDataSize{0};
        uint8_t moduleCount{0};
        uint8_t userParaSize{0};
        uint8_t externDiagParaSize{0};
        uint8_t vendorDataSize{0};
    };

public:
    CProfibusSlave();
    virtual ~CProfibusSlave();

    void init_Profibus(Config &config , void (*func)(std::vector<uint8_t>& outputbuf, std::vector<uint8_t>& inputbuf), void (*printfunc)(const char *, ...) = nullptr);

    void interruptPbRx(void);

    void interruptPbTx(void);

    void interruptTimer(void);

    void setIdent(uint8_t identHigh, uint8_t identLow)
    {
        m_config.identHigh = identHigh;
        m_config.identLow = identLow;
    }

protected:
    virtual void configTimer(void) = 0;

    virtual void runTimer(void) = 0;

    virtual void stopTimer(void) = 0;

    virtual void setTimerCounter(uint16_t value) = 0;

    virtual void setTimerMax(uint16_t value) = 0;

    virtual void clearOverflowFlag(void) = 0;

    virtual void configUart(void) = 0;

    virtual void activateTxInterrupt(void) = 0;

    virtual void deactivateTxInterrupt(void) = 0;

    virtual void activateRxInterrupt(void) = 0;

    virtual void deactivateRxInterrupt(void) = 0;

    virtual void setTxFlag(void) = 0;

    virtual void clearTxFlag(void) = 0;

    virtual void clearRxFlag(void) = 0;

    virtual void waitForActivTransmission(void) = 0;

    virtual void TxRs485Enable(void) = 0;

    virtual void TxRs485Disable(void) = 0;

    virtual void RxRs485Enable(void) = 0;

    virtual void configRs485Pin(void) = 0;

    virtual uint8_t getUartValue(void) = 0;

    virtual void setUartValue(uint8_t value) = 0;

    virtual void configErrorLed(void) = 0;

    virtual void errorLedOn(void) = 0;

    virtual void errorLedOff(void) = 0;

    virtual uint32_t millis(void) = 0;

    void (*m_printfunc)(const char *, ...) = nullptr;

private:
    void rxFunc(void);
    void sendCmd(CmdType type, uint8_t function_code, uint8_t sap_offset, volatile uint8_t *pdu, uint8_t length_pdu);
    void txFunc(volatile uint8_t *data, uint8_t length);

    uint8_t calcChecksum(volatile uint8_t *data, uint8_t length);
    uint8_t checkDestinationAdr(uint8_t destination);

    Config m_config;

    // uint8_t m_identHigh = 0x00;
    // uint8_t m_identLow = 0x2B;

    uint16_t m_bitTimeINcycle {0};
    uint16_t m_timeoutMaxSynTime {0};
    uint16_t m_timeoutMaxRxTime {0};
    uint16_t m_timeoutMaxTxTime {0};
    uint16_t m_timeoutMaxSdrTime {0};

    std::unique_ptr<volatile uint8_t[]> m_rxBuffer;
    std::unique_ptr<volatile uint8_t[]> m_txBuffer;
    volatile uint32_t m_rxBufCnt{0};
    volatile uint32_t m_txBufCnt{0};
    volatile uint32_t m_txCnt{0};

    // Profibus Flags und Variablen
    StreamStatus stream_status {StreamStatus::WaitSyn};
    DpSlaveState slave_status{DpSlaveState::POR};
    uint8_t diagnose_status_1;
    uint8_t slave_addr;
    uint8_t master_addr;
    uint8_t group;

    uint8_t source_add_last = MASTER_ADD_DEFAULT;
    bool fcv_act = false;
    bool fcb_last = false;

    bool freeze = false, sync = false;

    // uint8_t watchdog1=0,watchdog2=0;
    bool watchdog_act = false;
    uint8_t minTSDR = 0;

    bool freeze_act = false;
    bool sync_act = false;

    uint32_t last_connection_time;
    uint32_t watchdog_time;

    void (*m_datafunc)(std::vector<uint8_t>& outputbuf, std::vector<uint8_t>& inputbuf);

    std::vector<uint8_t> m_outputReg;
    std::vector<uint8_t> m_inputReg;
    std::vector<uint8_t> m_userPara;
    std::vector<uint8_t> m_diagData;
    std::vector<uint8_t> m_VendorData;

    uint8_t User_Para_size;
    // uint8_t Input_Data_size;
    // uint8_t Output_Data_size;
    uint8_t Module_cnt;
    
    typedef struct
    {
        uint8_t data[2];
    }ModulData;
    std::vector<ModulData> m_moduleData; // [][0] = Anzahl Eingaenge, [][1] = Anzahl Ausgaenge
    uint8_t Vendor_Data_size;                // Anzahl eingelesene Herstellerspezifische Bytes
};
