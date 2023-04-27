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
#include <stdio.h>

enum class DpSlaveState : uint8_t
{
    POR = 1,  // Power on Reset
    WPRM = 2, // Wait for Parameter
    WCFG = 3, // Wait for Config
    DXCHG = 4// Dataexchange
};

///////////////////////////////////////////////////////////////////////////////////////////////////
// Adressen
///////////////////////////////////////////////////////////////////////////////////////////////////
//#define MASTER_ADD            2     // SPS Adresse
#define SAP_OFFSET 128 // Service Access Point Adress Offset
#define BROADCAST_ADD 127
#define DEFAULT_ADD 126 // Auslieferungsadresse
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// Kommandotypen
///////////////////////////////////////////////////////////////////////////////////////////////////

enum class CmdType: uint8_t
{
    SD1 = 0x10, // Telegramm ohne Datenfeld
    SD2 = 0x68, // Daten Telegramm variabel
    SD3 = 0xA2, // Daten Telegramm fest
    SD4 = 0xDC, // Token
    SC = 0xE5,  // Kurzquittung
    ED = 0x16   // Ende
};

///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// Function Codes
///////////////////////////////////////////////////////////////////////////////////////////////////
/* FC Request */
#define FDL_STATUS 0x09 // SPS: Status Abfrage
#define SRD_LOW 0x0C    // SPS: Ausgaenge setzen, Eingaenge lesen
#define SRD_HIGH 0x0D   // SPS: Ausgaenge setzen, Eingaenge lesen
#define FCV_ 0x10
#define FCB_ 0x20
#define REQUEST_ 0x40

/* FC Response */
#define FDL_STATUS_OK 0x00 // SLA: OK
#define DATA_LOW 0x08      // SLA: (Data low) Daten Eingaenge senden
#define DATA_HIGH 0x0A     // SLA: (Data high) Diagnose anstehend
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// Service Access Points (DP Slave) MS0
///////////////////////////////////////////////////////////////////////////////////////////////////
#define SAP_SET_SLAVE_ADR 55   // Master setzt Slave Adresse, Slave Anwortet mit SC
#define SAP_RD_INP 56          // Master fordert Input Daten, Slave sendet Input Daten
#define SAP_RD_OUTP 57         // Master fordert Output Daten, Slave sendet Output Daten
#define SAP_GLOBAL_CONTROL 58  // Master Control, Slave Antwortet nicht
#define SAP_GET_CFG 59         // Master fordert Konfig., Slave sendet Konfiguration
#define SAP_SLAVE_DIAGNOSIS 60 // Master fordert Diagnose, Slave sendet Diagnose Daten
#define SAP_SET_PRM 61         // Master sendet Parameter, Slave sendet SC
#define SAP_CHK_CFG 62         // Master sendet Konfuguration, Slave sendet SC
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// SAP: Global Control (Daten Master)
///////////////////////////////////////////////////////////////////////////////////////////////////
#define CLEAR_DATA_ 0x02
#define UNFREEZE_ 0x04
#define FREEZE_ 0x08
#define UNSYNC_ 0x10
#define SYNC_ 0x20
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// SAP: Diagnose (Antwort Slave)
///////////////////////////////////////////////////////////////////////////////////////////////////
/* Status Byte 1 */
#define STATUS_1_DEFAULT 0x00
#define STATION_NOT_EXISTENT_ 0x01
#define STATION_NOT_READY_ 0x02
#define CFG_FAULT_ 0x04
#define EXT_DIAG_ 0x08 // Erweiterte Diagnose vorhanden
#define NOT_SUPPORTED_ 0x10
#define INV_SLAVE_RESPONSE_ 0x20
#define PRM_FAULT_ 0x40
#define MASTER_LOCK 0x80

/* Status Byte 2 */
#define STATUS_2_DEFAULT 0x04
#define PRM_REQ_ 0x01
#define STAT_DIAG_ 0x02
#define WD_ON_ 0x08
#define FREEZE_MODE_ 0x10
#define SYNC_MODE_ 0x20
//#define free                  0x40
#define DEACTIVATED_ 0x80

/* Status Byte 3 */
#define DIAG_SIZE_OK 0x00
#define DIAG_SIZE_ERROR 0x80

/* Adresse */
#define MASTER_ADD_DEFAULT 0xFF

/* Erweiterte Diagnose (EXT_DIAG_ = 1) */
#define EXT_DIAG_TYPE_ 0xC0     // Bit 6-7 ist Diagnose Typ
#define EXT_DIAG_BYTE_CNT_ 0x3F // Bit 0-5 sind Anzahl der Diagnose Bytes

#define EXT_DIAG_GERAET 0x00  // Wenn Bit 7 und 6 = 00, dann Geraetebezogen
#define EXT_DIAG_KENNUNG 0x40 // Wenn Bit 7 und 6 = 01, dann Kennungsbezogen
#define EXT_DIAG_KANAL 0x80   // Wenn Bit 7 und 6 = 10, dann Kanalbezogen
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// SAP: Set Parameters Request (Daten Master)
///////////////////////////////////////////////////////////////////////////////////////////////////
/* Befehl */
#define LOCK_SLAVE_ 0x80   // Slave fuer andere Master gesperrt
#define UNLOCK_SLAVE_ 0x40 // Slave fuer andere Master freigegeben
#define ACTIVATE_SYNC_ 0x20
#define ACTIVATE_FREEZE_ 0x10
#define ACTIVATE_WATCHDOG_ 0x08

/* DPV1 Status Byte 1 */
#define DPV1_MODE_ 0x80
#define FAIL_SAVE_MODE_ 0x40
#define PUBLISHER_MODE_ 0x20
#define WATCHDOG_TB_1MS 0x04

/* DPV1 Status Byte 2 */
#define PULL_PLUG_ALARM_ 0x80
#define PROZESS_ALARM_ 0x40
#define DIAGNOSE_ALARM_ 0x20
#define VENDOR_ALARM_ 0x10
#define STATUS_ALARM_ 0x08
#define UPDATE_ALARM_ 0x04
#define CHECK_CONFIG_MODE_ 0x01

/* DPV1 Status Byte 3 */
#define PARAMETER_CMD_ON_ 0x80
#define ISOCHRON_MODE_ON_ 0x10
#define PARAMETER_BLOCK_ 0x08
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// SAP: Check Config Request (Daten Master)
///////////////////////////////////////////////////////////////////////////////////////////////////
#define CFG_DIRECTION_ 0x30   // Bit 4-5 ist Richtung. 01 =  Eingang, 10 = Ausgang, 11 = Eingang/Ausgang
#define CFG_INPUT 0x10        // Eingang
#define CFG_OUTPUT 0x20       // Ausgang
#define CFG_INPUT_OUTPUT 0x30 // Eingang/Ausgang
#define CFG_SPECIAL 0x00      // Spezielles Format wenn mehr als 16/32Byte uebertragen werden sollen

#define CFG_KONSISTENZ_ 0x80    // Bit 7 ist Konsistenz. 0 = Byte oder Wort, 1 = Ueber gesamtes Modul
#define CFG_KONS_BYTE_WORT 0x00 // Byte oder Wort
#define CFG_KONS_MODUL 0x80     // Modul

#define CFG_WIDTH_ 0x40 // Bit 6 ist IO Breite. 0 = Byte (8bit), 1 = Wort (16bit)
#define CFG_BYTE 0x00   // Byte
#define CFG_WORD 0x40   // Wort

/* Kompaktes Format */
#define CFG_BYTE_CNT_ 0x0F // Bit 0-3 sind Anzahl der Bytes oder Worte. 0 = 1 Byte, 1 = 2 Byte usw.

/* Spezielles Format */
#define CFG_SP_DIRECTION_ 0xC0   // Bit 6-7 ist Richtung. 01 =  Eingang, 10 = Ausgang, 11 = Eingang/Ausgang
#define CFG_SP_VOID 0x00         // Leerplatz
#define CFG_SP_INPUT 0x40        // Eingang
#define CFG_SP_OUTPUT 0x80       // Ausgang
#define CFG_SP_INPUT_OUTPUT 0xC0 // Eingang/Ausgang

#define CFG_SP_VENDOR_CNT_ 0x0F // Bit 0-3 sind Anzahl der herstellerspezifischen Bytes. 0 = keine

/* Spezielles Format / Laengenbyte */
#define CFG_SP_BYTE_CNT_ 0x3F // Bit 0-5 sind Anzahl der Bytes oder Worte. 0 = 1 Byte, 1 = 2 Byte usw.
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
#define TIMEOUT_MAX_SYN_BIT 33
#define TIMEOUT_MAX_RX_BIT 15
#define TIMEOUT_MAX_TX_BIT 15
#define TIMEOUT_MAX_SDR_BIT 15
///////////////////////////////////////////////////////////////////////////////////////////////////