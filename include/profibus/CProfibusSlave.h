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

//#define F_CPU 16000000UL//Geschwindigkeit ändern

///////////////////////////////////////////////////////////////////////////////////////////////////
constexpr uint32_t counterFrequency = F_CPU;
constexpr uint32_t pbSpeed = 500000;

constexpr uint32_t bitTimeINcycle = counterFrequency / pbSpeed;
///////////////////////////////////////////////////////////////////////////////////////////////////
constexpr uint32_t timeoutMaxSynTime = 33 * bitTimeINcycle; // 33 TBit = TSYN
constexpr uint32_t timeoutMaxRxTime = 15 * bitTimeINcycle;
constexpr uint32_t timeoutMaxTxTime = 15 * bitTimeINcycle;
constexpr uint32_t timeoutMaxSdrTime = 15 * bitTimeINcycle; // 15 Tbit = TSDR
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
#define MAX_BUFFER_SIZE       45

#define INPUT_DATA_SIZE       2    // Anzahl Bytes die vom Master kommen
#define OUTPUT_DATA_SIZE      5    // Anzahl Bytes die an Master gehen
#define MODULE_CNT            5     // Anzahl der Module (Ein- Ausgangsmodule) bei modularer Station

#define USER_PARA_SIZE        0     // Anzahl Bytes fuer Herstellerspezifische Parameterdaten
#define EXT_DIAG_DATA_SIZE    0     // Anzahl Bytes fuer erweiterte Diagnose
#define VENDOR_DATA_SIZE      0     // Anzahl Herstellerspezifische Moduldaten
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// Profibus Ablaufsteuerung
///////////////////////////////////////////////////////////////////////////////////////////////////
#define PROFIBUS_WAIT_SYN     1
#define PROFIBUS_WAIT_DATA    2
#define PROFIBUS_GET_DATA     3
#define PROFIBUS_WAIT_MINTSDR 4
#define PROFIBUS_SEND_DATA    5
///////////////////////////////////////////////////////////////////////////////////////////////////

class CProfibusSlave
{

public:
    void init_Profibus (uint8_t identHigh, uint8_t identLow, void (*func)(volatile uint8_t *outputbuf,volatile uint8_t *inputbuf), void (*printfunc)(uint8_t* buffer, uint8_t len) = nullptr);

    void interruptPbRx(void);

    void interruptPbTx(void);

    void interruptTimer(void);

    void setIdent(uint8_t identHigh, uint8_t identLow){m_identHigh = identHigh; m_identLow = identLow;}

protected:

    virtual void configTimer(void) = 0;

    virtual void runTimer(void) = 0;

    virtual void stopTimer(void) = 0;

    virtual void setTimerCounter(uint32_t value) = 0;

    virtual void setTimerMax(uint32_t value) = 0;

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

    virtual uint32_t millis(void);

    void (*m_printfunc)(uint8_t* buffer, uint8_t len) = nullptr;

private:

    void profibus_RX (void);
    void profibus_send_CMD (uint8_t type, uint8_t function_code, uint8_t sap_offset, volatile uint8_t *pdu, uint8_t length_pdu);
    void profibus_TX (volatile uint8_t *data, uint8_t length);

    uint8_t calc_checksum               (volatile uint8_t *data, uint8_t length);
    uint8_t check_destination_addr      (uint8_t destination);

    uint8_t m_identHigh = 0x00;
    uint8_t m_identLow = 0x2B;

    volatile uint8_t m_pbUartRxBuffer[MAX_BUFFER_SIZE];
    volatile uint8_t m_pbUartTxBuffer[MAX_BUFFER_SIZE];
    volatile uint32_t m_pbUartRxCnt = 0;
    volatile uint32_t m_pbUartTxCnt = 0;
    volatile uint32_t pb_tx_cnt = 0;

    // Profibus Flags und Variablen
    uint8_t stream_status = PROFIBUS_WAIT_SYN;
    uint8_t slave_status;
    uint8_t diagnose_status_1;
    uint8_t slave_addr;
    uint8_t master_addr;
    uint8_t group;

    uint8_t source_add_last=MASTER_ADD_DEFAULT;
    bool fcv_act=false;
    bool fcb_last=false;

    bool freeze=false,sync=false;

    //uint8_t watchdog1=0,watchdog2=0;
    bool watchdog_act=false;
    uint8_t minTSDR=0;

    bool freeze_act=false;
    bool sync_act=false;


    uint32_t last_connection_time;
    uint32_t watchdog_time;

    void (*m_datafunc)(volatile uint8_t *outputbuf,volatile uint8_t *inputbuf);

    

    #if (OUTPUT_DATA_SIZE > 0)
        uint8_t output_register[OUTPUT_DATA_SIZE];
    #endif
    #if (INPUT_DATA_SIZE > 0)
        uint8_t input_register [INPUT_DATA_SIZE];
    #endif
    #if (USER_PARA_SIZE > 0)
        uint8_t User_Para[USER_PARA_SIZE];
    #endif
    #if (EXT_DIAG_DATA_SIZE > 0)
        uint8_t Diag_Data[EXT_DIAG_DATA_SIZE];
    #endif
    #if (VENDOR_DATA_SIZE > 0)
    uint8_t Vendor_Data[VENDOR_DATA_SIZE];
    #endif
    uint8_t User_Para_size;
    //uint8_t Input_Data_size;
    //uint8_t Output_Data_size;
    uint8_t Module_cnt;
    uint8_t Module_Data_size[MODULE_CNT][2]; // [][0] = Anzahl Eingaenge, [][1] = Anzahl Ausgaenge
    uint8_t Vendor_Data_size;   // Anzahl eingelesene Herstellerspezifische Bytes
};
