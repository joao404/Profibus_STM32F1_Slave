/*
*****************************************************************************
*		z21Interface.cpp - library for Roco Z21 LAN protocoll
*		This file is based on code of Philipp Gahtow
*		Copyright (c) 2013-2021 Philipp Gahtow  All right reserved.
*
*
* This library is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
*/

// include this library's description file
#include "z21/z21Interface.h"

// Constructor /////////////////////////////////////////////////////////////////
// Function that handles the creation and setup of instances

z21Interface::z21Interface(HwType hwType, uint32_t swVersion, boolean debug)
	: m_debug(debug)
{
	// initialize this instance's variables
	m_hwType = hwType;
	m_swVersion = swVersion;
	z21InterfaceIPpreviousMillis = 0;
	m_railPower = EnergyState::csTrackVoltageOff;
	clearIPSlots();
}

// Public Methods //////////////////////////////////////////////////////////////
// Functions available in Wiring sketches, this library, and other libraries

//*********************************************************************************************
// Daten ermitteln und Auswerten
void z21Interface::receive(uint8_t client, uint8_t *packet)
{
	addIPToSlot(client, 0);
	// send a reply, to the IP address and port that sent us the packet we received
	uint16_t header = (packet[3] << 8) + packet[2];
	uint8_t data[16]; // z21Interface send storage

	// #if defined(ESP32)
	// 	portMUX_TYPE myMutex = portMUX_INITIALIZER_UNLOCKED;
	// #endif

	switch (static_cast<z21Interface::Header>(header))
	{
	case z21Interface::Header::LAN_GET_SERIAL_NUMBER:
	{
		if (m_debug)
		{
			ZDebug.println("GET_SERIAL_NUMBER");
		}
		uint16_t serialNumber = getSerialNumber();
		data[0] = serialNumber & 0xFF;
		data[1] = (serialNumber >> 8) & 0xFF;
		data[2] = 0x00;
		data[3] = 0x00;
		EthSend(client, 0x08, z21Interface::Header::LAN_GET_SERIAL_NUMBER, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone)); // Seriennummer 32 Bit (little endian)
	}
	break;
	case z21Interface::Header::LAN_GET_HWINFO:
		if (m_debug)
		{
			ZDebug.println("GET_HWINFO");
		}
		data[0] = static_cast<uint8_t>(m_hwType) & 0xFF; // HwType 32 Bit
		data[1] = (static_cast<uint8_t>(m_hwType) >> 8) & 0xFF;
		data[2] = (static_cast<uint8_t>(m_hwType) >> 16) & 0xFF;
		data[3] = (static_cast<uint8_t>(m_hwType) >> 24) & 0xFF;
		data[4] = m_swVersion & 0xFF; // FW Version 32 Bit
		data[5] = (m_swVersion >> 8) & 0xFF;
		data[6] = (m_swVersion >> 16) & 0xFF;
		data[7] = (m_swVersion >> 24) & 0xFF;
		EthSend(client, 0x0C, z21Interface::Header::LAN_GET_HWINFO, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		break;
	case z21Interface::Header::LAN_LOGOFF:
		if (m_debug)
		{
			ZDebug.println("LOGOFF");
		}
		clearIPSlot(client);
		// Antwort von Z21: keine
		break;
	case z21Interface::Header::LAN_GET_CODE: // SW Feature-Umfang der Z21
		/*#define Z21_NO_LOCK        0x00  // keine Features gesperrt
		  #define z21Interface_START_LOCKED   0x01  // �z21Interface start�: Fahren und Schalten per LAN gesperrt
		  #define z21Interface_START_UNLOCKED 0x02  // �z21Interface start�: alle Feature-Sperren aufgehoben */
		data[0] = 0x00; // keine Features gesperrt
		EthSend(client, 0x05, z21Interface::Header::LAN_GET_CODE, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		break;
	case (z21Interface::Header::LAN_X_HEADER):
		//---------------------- LAN X-Header BEGIN ---------------------------
		switch (static_cast<z21Interface::XHeader>(packet[4]))
		{ // X-Header
		case z21Interface::XHeader::LAN_X_GET_SETTING:
			//---------------------- Switch BD0 BEGIN ---------------------------
			switch (packet[5])
			{ // DB0
			case 0x21:
				if (m_debug)
				{
					ZDebug.println("X_GET_VERSION");
				}
				data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_GET_VERSION); // X-Header: 0x63
				data[1] = 0x21;															  // DB0
				data[2] = 0x30;															  // X-Bus Version
				data[3] = 0x12;															  // ID der Zentrale
				EthSend(client, 0x09, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
				break;
			case 0x24:
				data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_STATUS_CHANGED); // X-Header: 0x62
				data[1] = 0x22;																 // DB0
				data[2] = static_cast<uint8_t>(m_railPower);								 // DB1: Status
				// ZDebug.print("X_GET_STATUS ");
				// csEmergencyStop  0x01 // Der Nothalt ist eingeschaltet
				// csTrackVoltageOff  0x02 // Die Gleisspannung ist abgeschaltet
				// csShortCircuit  0x04 // Kurzschluss
				// csProgrammingModeActive 0x20 // Der Programmiermodus ist aktiv
				EthSend(client, 0x08, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
				break;
			case 0x80:
				if (m_debug)
				{
					ZDebug.println("X_SET_TRACK_POWER_OFF");
				}
#ifdef directResponse
				data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_TRACK_POWER);
				data[1] = 0x00;
				EthSend(client, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, Z21bcNone);
#endif
				notifyz21InterfaceRailPower(EnergyState::csTrackVoltageOff);
				break;
			case 0x81:
				if (m_debug)
				{
					ZDebug.println("X_SET_TRACK_POWER_ON");
				}
#ifdef directResponse
				data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_TRACK_POWER);
				data[1] = 0x01;
				EthSend(client, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
#endif
				notifyz21InterfaceRailPower(EnergyState::csNormal);

				break;
			}
			//---------------------- Switch DB0 ENDE ---------------------------
			break; // ENDE DB0
		case z21Interface::XHeader::LAN_X_CV_READ:
			if (packet[5] == 0x11) // LAN_X_CV_READ
			{					   // DB0
				if (m_debug)
				{
					ZDebug.println("X_CV_READ");
				}
				notifyz21InterfaceCVREAD(packet[6], packet[7]); // CV_MSB, CV_LSB
			}
			else if (packet[5] == 0x12) // LAN_X_DCC_WRITE_REGISTER
			{							// DB0
				if (m_debug)
				{
					ZDebug.println("X_DCC_WRITE");
				}
				notifyz21InterfaceDCCWRITE(packet[6], packet[7]);
			}
			break;
		case z21Interface::XHeader::LAN_X_CV_WRITE:
			if (packet[5] == 0x12) // LAN_X_CV_WRITE
			{					   // DB0
				if (m_debug)
				{
					ZDebug.println("X_CV_WRITE");
				}
				notifyz21InterfaceCVWRITE(packet[6], packet[7], packet[8]); // CV_MSB, CV_LSB, value
			}
			else if ((packet[5] == 0xFF) && (packet[6] == 0x00)) // LAN_X_MM_WRITE_BYTE
			{													 // DB0
				if (m_debug)
				{
					ZDebug.println("_X_MM_WRITE_BYTE");
				}
				notifyz21InterfaceMMWRITE(packet[7], packet[8]);
			}
			break;
		case z21Interface::XHeader::LAN_X_DCC_READ_REGISTER:
			if (packet[5] == 0x11)
			{ // DB0
				if (m_debug)
				{
					ZDebug.println("X_DCC_READ");
				}
				notifyz21InterfaceDCCREAD(packet[6]);
			}
			break;
		case z21Interface::XHeader::LAN_X_CV_POM:
			if (packet[5] == 0x30)
			{ // DB0
				uint16_t Adr = ((packet[6] & 0x3F) << 8) + packet[7];
				uint16_t CVAdr = ((packet[8] & B11) << 8) + packet[9];
				uint8_t value = packet[10];
				if ((packet[8] & 0xFC) == 0xEC)
				{
					if (m_debug)
					{
						ZDebug.println("LAN_X_CV_POM_WRITE_BYTE");
					}
					notifyz21InterfaceCVPOMWRITEBYTE(Adr, CVAdr, value); // set Byte
				}
				else if ((packet[8] & 0xFC) == 0xE8)
				{
					if (m_debug)
					{
						ZDebug.println("LAN_X_CV_POM_WRITE_BIT");
					}
					notifyz21InterfaceCVPOMWRITEBIT(Adr, CVAdr, value); // set Bit
				}
				else
				{
					if (m_debug)
					{
						ZDebug.println("LAN_X_CV_POM_READ_BYTE");
					}
					notifyz21InterfaceCVPOMREADBYTE(Adr, CVAdr); // read uint8_t
				}
			}
			else if (packet[5] == 0x31)
			{ // DB0
				if (m_debug)
				{
					ZDebug.println("LAN_X_CV_POM_ACCESSORY");
				}
			}
			break;
		case z21Interface::XHeader::LAN_X_GET_TURNOUT_INFO:
		{
			if (m_debug)
			{
				ZDebug.println("X_GET_TURNOUT_INFO");
			}
			data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_GET_TURNOUT_INFO);
			data[1] = packet[5]; // High
			data[2] = packet[6]; // Low
			notifyz21InterfaceAccessoryInfo((packet[5] << 8) + packet[6], data[3]);
			EthSend(client, 0x09, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll)); // BC new 23.04. !!!(old = 0)

			break;
		}
		case z21Interface::XHeader::LAN_X_SET_TURNOUT:
		{
			// if (m_debug)
			// {
			// 	ZDebug.print("X_SET_TURNOUT Adr.:");
			// 	ZDebug.print((packet[5] << 8) + packet[6]);
			// 	ZDebug.print(":");
			// 	ZDebug.print(bitRead(packet[7], 0));
			// 	ZDebug.print("-");
			// 	ZDebug.println(bitRead(packet[7], 3));
			// }
			// bool TurnOnOff = bitRead(packet[7],3);  //Spule EIN/AUS
			notifyz21InterfaceAccessory((packet[5] << 8) + packet[6], bitRead(packet[7], 0), bitRead(packet[7], 3));
			//	Addresse					Links/Rechts			Spule EIN/AUS
			break;
		}
		case z21Interface::XHeader::LAN_X_SET_EXT_ACCESSORY:
		{
			if (m_debug)
			{
				ZDebug.print("X_SET_EXT_ACCESSORY RAdr.:");
				ZDebug.print((packet[5] << 8) + packet[6]);
				ZDebug.print(":0x");
				ZDebug.println(packet[7], HEX);
			}
			setExtACCInfo((packet[5] << 8) + packet[6], packet[7]);
			break;
		}
		case z21Interface::XHeader::LAN_X_GET_EXT_ACCESSORY_INFO:
		{
			if (m_debug)
			{
				ZDebug.print("X_EXT_ACCESSORY_INFO RAdr.:");
				ZDebug.print((packet[5] << 8) + packet[6]);
				ZDebug.print(":0x");
				ZDebug.println(packet[7], HEX); // DB2 Reserviert f�r zuk�nftige Erweiterungen
			}
			setExtACCInfo((packet[5] << 8) + packet[6], packet[7]);
			break;
		}
		case z21Interface::XHeader::LAN_X_SET_STOP:
			if (m_debug)
			{
				ZDebug.println("X_SET_STOP");
			}
			notifyz21InterfaceRailPower(EnergyState::csEmergencyStop);
			break;
		case z21Interface::XHeader::LAN_X_GET_LOCO_INFO:
			if (packet[5] == 0xF0)
			{ // DB0
				if (m_debug)
				{
					ZDebug.println("X_GET_LOCO_INFO");
				}
				// Antwort: LAN_X_LOCO_INFO  Adr_MSB - Adr_LSB
				returnLocoStateFull(client, word(packet[6] & 0x3F, packet[7]), false);
			}
			break;
		case z21Interface::XHeader::LAN_X_SET_LOCO_DRIVE:
			// setLocoBusy:
			addBusySlot(client, word(packet[6] & 0x3F, packet[7]));

			if (static_cast<z21Interface::XHeader>(packet[5]) == z21Interface::XHeader::LAN_X_SET_LOCO_FUNCTION)
			{ // DB0
				// LAN_X_SET_LOCO_FUNCTION  Adr_MSB        Adr_LSB            Type (00=AUS/01=EIN/10=UM)      Funktion
				notifyz21InterfaceLocoFkt(word(packet[6] & 0x3F, packet[7]), packet[8] >> 6, packet[8] & B00111111);
				// uint16_t Adr, uint8_t type, uint8_t fkt
			}
			else
			{ // DB0
				// ZDebug.print("X_SET_LOCO_DRIVE ");
				notifyz21InterfaceLocoSpeed(word(packet[6] & 0x3F, packet[7]), packet[8], packet[5] & 0x07);
			}
#ifdef directResponse
			returnLocoStateFull(client, word(packet[6] & 0x3F, packet[7]), true);
#endif
			break;
		case z21Interface::XHeader::LAN_X_GET_FIRMWARE_VERSION:
			if (m_debug)
			{
				ZDebug.println("X_GET_FIRMWARE_VERSION");
			}
			data[0] = 0xF3;						 // identify Firmware (not change)
			data[1] = 0x0A;						 // identify Firmware (not change)
			data[2] = (m_swVersion >> 8) & 0xFF; // V_MSB
			data[3] = m_swVersion & 0xFF;		 // V_LSB
			EthSend(client, 0x09, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
			break;
			/*
		  case 0x73:
			//LAN_X_??? WLANmaus periodische Abfrage:
			//0x09 0x00 0x40 0x00 0x73 0x00 0xFF 0xFF 0x00
			//length X-Header	XNet-Msg			  speed?
			if (m_debug)
			{
			ZDebug.println("LAN-X_WLANmaus");
			}
			//set Broadcastflags for WLANmaus:
			if (addIPToSlot(client, 0x00) == 0)
				addIPToSlot(client, Z21bcAll);
			break;
			*/
		default:
			if (m_debug)
			{
				ZDebug.print("UNKNOWN_LAN-X_COMMAND");
				// for (uint8_t i = 0; i < packet[0]; i++) {
				//	ZDebug.print(" 0x");
				//	ZDebug.print(packet[i], HEX);
				// }
				ZDebug.println();
			}
			data[0] = 0x61;
			data[1] = 0x82;
			EthSend(client, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		}
		//---------------------- LAN X-Header ENDE ---------------------------
		break;
	case z21Interface::Header::LAN_SET_BROADCASTFLAGS:
	{
		unsigned long bcflag = packet[7];
		bcflag = packet[6] | (bcflag << 8);
		bcflag = packet[5] | (bcflag << 8);
		bcflag = packet[4] | (bcflag << 8);
		addIPToSlot(client, getLocalBcFlag(bcflag));
		// no inside of the protokoll, but good to have:
		notifyz21InterfaceRailPower(m_railPower); // Zustand Gleisspannung Antworten
		if (m_debug)
		{
			ZDebug.print("SET_BROADCASTFLAGS: ");
			ZDebug.println(addIPToSlot(client, 0x00), BIN);
			// 1=BC Power, Loco INFO, Trnt INFO; 2=BC �nderungen der R�ckmelder am R-Bus
		}
		break;
	}
	case z21Interface::Header::LAN_GET_BROADCASTFLAGS:
	{
		unsigned long flag = getz21InterfaceBcFlag(addIPToSlot(client, 0x00));
		data[0] = flag;
		data[1] = flag >> 8;
		data[2] = flag >> 16;
		data[3] = flag >> 24;
		EthSend(client, 0x08, z21Interface::Header::LAN_GET_BROADCASTFLAGS, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		if (m_debug)
		{
			ZDebug.print("GET_BROADCASTFLAGS: ");
			ZDebug.println(flag, BIN);
		}
		break;
	}
	case z21Interface::Header::LAN_GET_LOCOMODE:
		/*
		In der Z21 kann das Ausgabeformat (DCC, MM) pro Lok-Adresse persistent gespeichert werden.
		Es k�nnen maximal 256 verschiedene Lok-Adressen abgelegt werden. Jede Adresse >= 256 ist automatisch DCC.
		*/
		if (m_debug)
		{
			Serial.println(F("LAN_GET_LOCOMODE"));
		}
		data[0] = packet[4];
		data[1] = packet[5];
		handleGetLocoMode(word(packet[4], packet[5]), data[2]);
		EthSend(client, 0x07, z21Interface::Header::LAN_GET_LOCOMODE, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		break;
	case z21Interface::Header::LAN_SET_LOCOMODE:
		// nothing to replay all DCC Format
		if (m_debug)
		{
			Serial.println(F("LAN_SET_LOCOMODE"));
		}
		handleSetLocoMode(word(packet[4], packet[5]), packet[6]);
		break;
	case z21Interface::Header::LAN_GET_TURNOUTMODE:
		/*
		In der Z21 kann das Ausgabeformat (DCC, MM) pro Funktionsdecoder-Adresse persistent gespeichert werden.
		Es k�nnen maximal 256 verschiedene Funktionsdecoder -Adressen gespeichert werden. Jede Adresse >= 256 ist automatisch DCC.
		*/
		if (m_debug)
		{
			ZDebug.println(F("LAN_SET_TURNOUTMODE"));
		}
		data[0] = packet[4];
		data[1] = packet[5];
		// data[2] = 0; // 0=DCC Format; 1=MM Format
		handleGetTurnOutMode(word(packet[4], packet[5]), data[2]);
		EthSend(client, 0x07, z21Interface::Header::LAN_GET_LOCOMODE, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		break;
	case z21Interface::Header::LAN_SET_TURNOUTMODE:
		if (m_debug)
		{
			ZDebug.println(F("LAN_SET_TURNOUTMODE"));
		}
		handleSetTurnOutMode(word(packet[4], packet[5]), packet[6]);
		break;
	case z21Interface::Header::LAN_RMBUS_GETDATA:
		if (m_debug)
		{
			ZDebug.println("RMBUS_GETDATA");
		}
		// ask for group state 'Gruppenindex'
		notifyz21InterfaceS88Data(packet[4]); // normal Antwort hier nur an den anfragenden Client! (Antwort geht hier an alle!)
		break;
	case z21Interface::Header::LAN_RMBUS_PROGRAMMODULE:
		break;
	case z21Interface::Header::LAN_SYSTEMSTATE_GETDATA:
	{ // System state
		if (m_debug)
		{
			ZDebug.println("LAN_SYS-State");
		}
		notifyz21InterfacegetSystemInfo(client);
		break;
	}
	case z21Interface::Header::LAN_RAILCOM_GETDATA:
	{
		uint16_t Adr = 0;
		if (packet[4] == 0x01)
		{ // RailCom-Daten f�r die gegebene Lokadresse anfordern
			Adr = word(packet[6], packet[5]);
		}
		Adr = notifyz21InterfaceRailcom(); // return global Railcom Adr
		data[0] = Adr >> 8;				   // LocoAddress
		data[1] = Adr & 0xFF;			   // LocoAddress
		data[2] = 0x00;					   // UINT32 ReceiveCounter Empfangsz�hler in Z21
		data[3] = 0x00;
		data[4] = 0x00;
		data[5] = 0x00;
		data[6] = 0x00; // UINT32 ErrorCounter Empfangsfehlerz�hler in Z21
		data[7] = 0x00;
		data[8] = 0x00;
		data[9] = 0x00;
		/*
		data[10] = 0x00;	//UINT8 Reserved1 experimentell, siehe Anmerkung
		data[11] = 0x00;	//UINT8 Reserved2 experimentell, siehe Anmerkung
		data[12] = 0x00;	//UINT8 Reserved3 experimentell, siehe Anmerkung
		*/
		EthSend(client, 0x0E, z21Interface::Header::LAN_RAILCOM_DATACHANGED, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		break;
	}
	case z21Interface::Header::LAN_LOCONET_FROM_LAN:
	{
		if (m_debug)
		{
			ZDebug.println("LOCONET_FROM_LAN");
		}
		uint8_t LNdata[packet[0] - 0x04]; // n Bytes
		for (uint8_t i = 0; i < (packet[0] - 0x04); i++)
			LNdata[i] = packet[0x04 + i];
		notifyz21InterfaceLNSendPacket(LNdata, packet[0] - 0x04);
		// Melden an andere LAN-Client das Meldung auf LocoNet-Bus geschrieben wurde
		EthSend(client, packet[0], z21Interface::Header::LAN_LOCONET_FROM_LAN, packet, false, static_cast<uint16_t>(BcFlagShort::Z21bcLocoNet)); // LAN_LOCONET_FROM_LAN
		break;
	}
	case z21Interface::Header::LAN_LOCONET_DISPATCH_ADDR:
	{
		data[0] = packet[4];
		data[1] = packet[5];
		data[2] = notifyz21InterfaceLNdispatch(word(packet[5], packet[4])); // dispatchSlot
		if (m_debug)
		{
			ZDebug.print("LOCONET_DISPATCH_ADDR ");
			ZDebug.print(word(packet[5], packet[4]));
			ZDebug.print(",");
			ZDebug.println(data[2]);
		}
		EthSend(client, 0x07, z21Interface::Header::LAN_LOCONET_DISPATCH_ADDR, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		break;
	}
	case z21Interface::Header::LAN_LOCONET_DETECTOR:
		// if (m_debug)
		// {
		// 	ZDebug.println("LOCONET_DETECTOR Abfrage");
		// }
		notifyz21InterfaceLNdetector(client, packet[4], word(packet[6], packet[5])); // Anforderung Typ & Reportadresse
		break;
	case z21Interface::Header::LAN_CAN_DETECTOR:
		if (m_debug)
		{
			ZDebug.println("CAN_DETECTOR Abfrage");
		}
		notifyz21InterfaceCANdetector(client, packet[4], word(packet[6], packet[5])); // Anforderung Typ & CAN-ID
		break;

	case z21Interface::Header::LAN_GET_CONFIG1: // configuration read
	{
		// <-- 04 00 12 00
		// 0e 00 12 00 01 00 01 03 01 00 03 00 00 00
		std::array<uint8_t, 10> config;
		if (getConfig1(config))
		{
			uint8_t i = 0;
			for (auto &conf : config)
			{
				data[i++] = conf;
			}
		}
		EthSend(client, 0x0e, z21Interface::Header::LAN_GET_CONFIG1, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		if (m_debug)
		{
			ZDebug.print("Z21 Eins(read) ");
			ZDebug.print("RailCom: ");
			ZDebug.print(data[0], HEX);
			ZDebug.print(", PWR-Button: ");
			ZDebug.print(data[2], HEX);
			ZDebug.print(", ProgRead: ");
			switch (data[3])
			{
			case 0x00:
				ZDebug.print("nothing");
				break;
			case 0x01:
				ZDebug.print("Bit");
				break;
			case 0x02:
				ZDebug.print("Byte");
				break;
			case 0x03:
				ZDebug.print("both");
				break;
			}
			ZDebug.println();
		}
	}
	break;
	case z21Interface::Header::LAN_SET_CONFIG1:
	{ // configuration write
		//<-- 0e 00 13 00 01 00 01 03 01 00 03 00 00 00
		// 0x0e = Length; 0x12 = Header
		// Daten:
		//(0x01) RailCom: 0=aus/off, 1=ein/on
		//(0x00)
		//(0x01) Power-Button: 0=Gleisspannung aus, 1=Nothalt
		//(0x03) Auslese-Modus: 0=Nichts, 1=Bit, 2=Byte, 3=Beides
		if (m_debug)
		{
			ZDebug.print("Z21 Eins(write) ");
			ZDebug.print("RailCom: ");
			ZDebug.print(packet[4], HEX);
			ZDebug.print(", PWR-Button: ");
			ZDebug.print(packet[6], HEX);
			ZDebug.print(", ProgRead: ");
			switch (packet[7])
			{
			case 0x00:
				ZDebug.print("nothing");
				break;
			case 0x01:
				ZDebug.print("Bit");
				break;
			case 0x02:
				ZDebug.print("Byte");
				break;
			case 0x03:
				ZDebug.print("both");
				break;
			}
			ZDebug.println();
		}

		std::array<uint8_t, 10> config;
		uint8_t i = 4;
		for (auto &conf : config)
		{
			conf = packet[i];
		}
		setConfig1(config);
		// Request DCC to change
		notifyz21InterfaceUpdateConf();
	}
	break;
	case z21Interface::Header::LAN_GET_CONFIG2: // configuration read
	{
		//<-- 04 00 16 00
		// 14 00 16 00 19 06 07 01 05 14 88 13 10 27 32 00 50 46 20 4e

		std::array<uint8_t, 16> config;
		if (getConfig2(config))
		{
			uint8_t i = 0;
			for (auto &conf : config)
			{
				data[i++] = conf;
			}
		}
		// check range of MainV:
		if ((word(data[13], data[12]) > 0x59D8) || (word(data[13], data[12]) < 0x2A8F))
		{
			// set to 20V default:
			data[13] = highByte(0x4e20);
			data[12] = lowByte(0x4e20);
		}
		// check range of ProgV:
		if ((word(data[15], data[14]) > 0x59D8) || (word(data[15], data[14]) < 0x2A8F))
		{
			// set to 20V default:
			data[15] = highByte(0x4e20);
			data[14] = lowByte(0x4e20);
		}

		EthSend(client, 0x14, z21Interface::Header::LAN_GET_CONFIG2, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
		if (m_debug)
		{
			ZDebug.print("Z21 Eins(read) ");
			ZDebug.print("RstP(s): ");
			ZDebug.print(data[0]); // EEPROM Adr 60
			ZDebug.print(", RstP(f): ");
			ZDebug.print(data[1]); // EEPROM Adr 61
			ZDebug.print(", ProgP: ");
			ZDebug.print(data[2]); // EEPROM Adr 62
			ZDebug.print(", MainV: ");
			ZDebug.print(word(data[13], data[12])); // Value only: 11000 - 23000
			ZDebug.print(", ProgV: ");
			ZDebug.print(word(data[15], data[14])); // Value only: 11000=0x2A8F - 23000=0x59D8
			ZDebug.println();
		}
	}
	break;
	case z21Interface::Header::LAN_SET_CONFIG2:
	{ // configuration write
		//<-- 14 00 17 00 19 06 07 01 05 14 88 13 10 27 32 00 50 46 20 4e
		// 0x14 = Length; 0x16 = Header(read), 0x17 = Header(write)
		// Daten:
		// (0x19) Reset Packet (starten) (25-255)
		// (0x06) Reset Packet (fortsetzen) (6-64)
		// (0x07) Programmier-Packete (7-64)
		// (0x01) ?
		// (0x05) ?
		// (0x14) ?
		// (0x88) ?
		// (0x13) ?
		// (0x10) ?
		// (0x27) ?
		// (0x32) ?
		// (0x00) ?
		// (0x50) Hauptgleis (LSB) (11-23V)
		// (0x46) Hauptgleis (MSB)
		// (0x20) Programmiergleis (LSB) (11-23V): 20V=0x4e20, 21V=0x5208, 22V=0x55F0
		// (0x4e) Programmiergleis (MSB)
		if (m_debug)
		{
			ZDebug.print("Z21 Eins(write) ");
			ZDebug.print("RstP(s): ");
			ZDebug.print(packet[4]); // EEPROM Adr 60
			ZDebug.print(", RstP(f): ");
			ZDebug.print(packet[5]); // EEPROM Adr 61
			ZDebug.print(", ProgP: ");
			ZDebug.print(packet[6]); // EEPROM Adr 62
			ZDebug.print(", MainV: ");
			ZDebug.print(word(packet[17], packet[16]));
			ZDebug.print(", ProgV: ");
			ZDebug.print(word(packet[19], packet[18]));
			ZDebug.println();
		}
		std::array<uint8_t, 16> config;
		uint8_t i = 4;
		for (auto &conf : config)
		{
			conf = packet[i];
		}
		setConfig2(config);

		// Request DCC to change
		notifyz21InterfaceUpdateConf();
		break;
	}

	default:
		if (m_debug)
		{
			ZDebug.print("UNKNOWN_COMMAND");
			for (uint8_t i = 0; i < packet[0]; i++)
			{
				ZDebug.print(" 0x");
				ZDebug.print(packet[i], HEX);
			}
			ZDebug.println();
		}
		data[0] = 0x61;
		data[1] = 0x82;
		EthSend(client, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone));
	}
	//---------------------------------------------------------------------------------------
	// check if IP is still used:
	unsigned long currentMillis = millis();
	if ((currentMillis - z21InterfaceIPpreviousMillis) > z21InterfaceIPinterval)
	{
		z21InterfaceIPpreviousMillis = currentMillis;
		for (uint8_t i = 0; i < z21InterfaceclientMAX; i++)
		{
			if (ActIP[i].time > 0)
			{
				ActIP[i].time--; // Zeit herrunterrechnen
			}
			else
			{
				clearIP(i); // clear IP DATA
							// send MESSAGE clear Client
			}
		}
	}
}

//--------------------------------------------------------------------------------------------
// Zustand der Gleisversorgung setzten
void z21Interface::setPower(EnergyState state)
{
	uint8_t data[] = {static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_TRACK_POWER), 0x00};
	m_railPower = state;
	switch (state)
	{
	case EnergyState::csNormal:
		data[1] = 0x01;
		break;
	case EnergyState::csTrackVoltageOff:
		data[1] = 0x00;
		break;
	case EnergyState::csServiceMode:
		data[1] = 0x02;
		break;
	case EnergyState::csShortCircuit:
		data[1] = 0x08;
		break;
	case EnergyState::csEmergencyStop:
		data[0] = 0x81;
		data[1] = 0x00;
		break;
	}
	EthSend(0, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll));
	if (m_debug)
	{
		ZDebug.print("set_X_BC_TRACK_POWER ");
		ZDebug.println(static_cast<uint8_t>(state), HEX);
	}
}

//--------------------------------------------------------------------------------------------
// Abfrage letzte Meldung �ber Gleispannungszustand
z21Interface::EnergyState z21Interface::getPower()
{
	return m_railPower;
}

//--------------------------------------------------------------------------------------------
// return request for POM read uint8_t
void z21Interface::setCVPOMBYTE(uint16_t CVAdr, uint8_t value)
{
	uint8_t data[5];
	data[0] = 0x64;				   // X-Header
	data[1] = 0x14;				   // DB0
	data[2] = (CVAdr >> 8) & 0x3F; // CV_MSB;
	data[3] = CVAdr & 0xFF;		   // CV_LSB;
	data[4] = value;
	EthSend(0, 0x0A, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll));
}

//--------------------------------------------------------------------------------------------
// Zustand R�ckmeldung non - Z21 device - Busy!
void z21Interface::setLocoStateExt(int Adr)
{
	uint8_t ldata[6];
	memset(ldata, 0, sizeof(ldata));
	notifyz21InterfaceLocoState(Adr, ldata); // uint8_t Steps[0], uint8_t Speed[1], uint8_t F0[2], uint8_t F1[3], uint8_t F2[4], uint8_t F3[5]

	uint8_t data[9];
	data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_LOCO_INFO); // 0xEF X-HEADER
	data[1] = (Adr >> 8) & 0x3F;
	data[2] = Adr & 0xFF;
	// Fahrstufeninformation: 0=14, 2=28, 4=128
	if ((ldata[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step14))
		data[3] = 0; // 14 steps
	if ((ldata[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step28))
		data[3] = 2; // 28 steps
	if ((ldata[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step128))
		data[3] = 4;		  // 128 steps
	data[3] = data[3] | 0x08; // BUSY!

	data[4] = (char)ldata[1]; // DSSS SSSS
	data[5] = (char)ldata[2]; // F0, F4, F3, F2, F1
	data[6] = (char)ldata[3]; // F5 - F12; Funktion F5 ist bit0 (LSB)
	data[7] = (char)ldata[4]; // F13-F20
	data[8] = (char)ldata[5]; // F21-F28

	reqLocoBusy(Adr);

	EthSend(0, 14, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll) | static_cast<uint16_t>(BcFlagShort::Z21bcNetAll)); // Send Loco Status und Funktions to all active Apps
}

//--------------------------------------------------------------------------------------------
// Gibt aktuellen Lokstatus an Anfragenden Zur�ck
void z21Interface::returnLocoStateFull(uint8_t client, uint16_t Adr, bool bc)
// bc = true => to inform also other client over the change.
// bc = false => just ask about the loco state
{
	uint8_t ldata[6];
	memset(ldata, 0, sizeof(ldata));
	notifyz21InterfaceLocoState(Adr, ldata); // uint8_t Steps[0], uint8_t Speed[1], uint8_t F0[2], uint8_t F1[3], uint8_t F2[4], uint8_t F3[5]

	uint8_t data[9];
	data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_LOCO_INFO); // 0xEF X-HEADER
	data[1] = (Adr >> 8) & 0x3F;
	data[2] = Adr & 0xFF;
	// Fahrstufeninformation: 0=14, 2=28, 4=128
	if ((ldata[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step14))
		data[3] = 0; // 14 steps
	if ((ldata[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step28))
		data[3] = 2; // 28 steps
	if ((ldata[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step128))
		data[3] = 4;		  // 128 steps
	data[3] = data[3] | 0x08; // BUSY!

	data[4] = (char)ldata[1]; // DSSS SSSS
	data[5] = (char)ldata[2]; // F0, F4, F3, F2, F1
	data[6] = (char)ldata[3]; // F5 - F12; Funktion F5 ist bit0 (LSB)
	data[7] = (char)ldata[4]; // F13-F20
	data[8] = (char)ldata[5]; // F21-F28

	// Info to all:
	for (uint8_t i = 0; i < z21InterfaceclientMAX; i++)
	{
		if (ActIP[i].client != client)
		{
			if ((ActIP[i].BCFlag & (static_cast<uint16_t>(BcFlagShort::Z21bcAll) | static_cast<uint16_t>(BcFlagShort::Z21bcNetAll))) > 0)
			{
				if (bc == true)
					EthSend(ActIP[i].client, 14, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone)); // Send Loco status und Funktions to BC Apps
			}
		}
		else
		{ // Info to client that ask:
			if (ActIP[i].adr == Adr)
			{
				data[3] = data[3] & B111; // clear busy flag!
			}
			EthSend(client, 14, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcNone)); // Send Loco status und Funktions to request App
			data[3] = data[3] | 0x08;																							// BUSY!
		}
	}
}

//--------------------------------------------------------------------------------------------
// return state of S88 sensors
void z21Interface::setS88Data(uint8_t *data)
{
	EthSend(0, 0x0F, z21Interface::Header::LAN_RMBUS_DATACHANGED, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcRBus)); // RMBUS_DATACHANED
}

//--------------------------------------------------------------------------------------------
// return state from LN detector
void z21Interface::setLNDetector(uint8_t client, uint8_t *data, uint8_t DataLen)
{
	EthSend(client, 0x04 + DataLen, z21Interface::Header::LAN_LOCONET_DETECTOR, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcLocoNet)); // LAN_LOCONET_DETECTOR
}

//--------------------------------------------------------------------------------------------
// LN Meldungen weiterleiten
void z21Interface::setLNMessage(uint8_t *data, uint8_t DataLen, uint8_t bcType, bool TX)
{
	if (TX)																						   // Send by Z21 or Receive a Packet?
		EthSend(0, 0x04 + DataLen, z21Interface::Header::LAN_LOCONET_Z21_TX, data, false, bcType); // LAN_LOCONET_Z21_TX
	else
		EthSend(0, 0x04 + DataLen, z21Interface::Header::LAN_LOCONET_Z21_RX, data, false, bcType); // LAN_LOCONET_Z21_RX
}

//--------------------------------------------------------------------------------------------
// return state from CAN detector
void z21Interface::setCANDetector(uint16_t NID, uint16_t Adr, uint8_t port, uint8_t typ, uint16_t v1, uint16_t v2)
{
	uint8_t data[10];
	data[0] = NID & 0x08;
	data[1] = NID >> 8;
	data[2] = Adr & 0x08;
	data[3] = Adr >> 8;
	data[4] = port;
	data[5] = typ;
	data[6] = v1 & 0x08;
	data[7] = v1 >> 8;
	data[8] = v2 & 0x08;
	data[9] = v2 >> 8;
	EthSend(0, 0x0E, z21Interface::Header::LAN_CAN_DETECTOR, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcCANDetector)); // CAN_DETECTOR
}

//--------------------------------------------------------------------------------------------
// Return the state of accessory
void z21Interface::setTrntInfo(uint16_t Adr, bool State)
{
	uint8_t data[4];
	data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_TURNOUT_INFO); // 0x43 X-HEADER
	data[1] = Adr >> 8;														   // High
	data[2] = Adr & 0xFF;													   // Low
	data[3] = State + 1;
	//  if (State == true)
	//    data[3] = 2;
	//  else data[3] = 1;
	EthSend(0, 0x09, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll));
}

//--------------------------------------------------------------------------------------------
// Return EXT accessory info
void z21Interface::setExtACCInfo(uint16_t Adr, uint8_t State, bool Status)
{
	uint8_t data[5];
	data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_GET_EXT_ACCESSORY_INFO); // 0x44 X-HEADER
	data[1] = Adr >> 8;																	 // High
	data[2] = Adr & 0xFF;																 // Low
	data[3] = State;
	data[4] = Status; // 0x00 � Data Valid; 0xFF � Data Unknown
	notifyz21InterfaceExtAccessory(Adr, State);
	EthSend(0, 0x0A, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll));
}

//--------------------------------------------------------------------------------------------
// Return CV Value for Programming
void z21Interface::setCVReturn(uint16_t CV, uint8_t value)
{
	if (m_debug)
	{
	  Serial.println("setCVReturn");
	}
	uint8_t data[5];
	data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_CV_RESULT); // 0x64 X-Header
	data[1] = 0x14;															// DB0
	data[2] = CV >> 8;														// CV_MSB;
	data[3] = CV & 0xFF;													// CV_LSB;
	data[4] = value;
	EthSend(0, 0x0A, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll));
}

//--------------------------------------------------------------------------------------------
// Return no ACK from Decoder
void z21Interface::setCVNack()
{
	if (m_debug)
	{
	  Serial.println("setCVNack");
	}
	uint8_t data[2];
	data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_CV_NACK); // 0x61 X-Header
	data[1] = 0x13;														  // DB0
	EthSend(0, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll));
}

//--------------------------------------------------------------------------------------------
// Return Short while Programming
void z21Interface::setCVNackSC()
{
	if (m_debug)
	{
	  Serial.println("setCVNackSC");
	}
	uint8_t data[2];
	data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_CV_NACK_SC); // 0x61 X-Header
	data[1] = 0x12;															 // DB0
	EthSend(0, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll));
}

//--------------------------------------------------------------------------------------------
// Send Changing of SystemInfo
void z21Interface::sendSystemInfo(uint8_t client, uint16_t maincurrent, uint16_t mainvoltage, uint16_t temp)
{
	uint8_t data[16];
	data[0] = maincurrent & 0xFF;				  // MainCurrent mA
	data[1] = maincurrent >> 8;					  // MainCurrent mA
	data[2] = data[0];							  // ProgCurrent mA
	data[3] = data[1];							  // ProgCurrent mA
	data[4] = data[0];							  // FilteredMainCurrent
	data[5] = data[1];							  // FilteredMainCurrent
	data[6] = temp & 0xFF;						  // Temperature
	data[7] = temp >> 8;						  // Temperature
	data[8] = mainvoltage & 0xFF;				  // SupplyVoltage
	data[9] = mainvoltage >> 8;					  // SupplyVoltage
	data[10] = data[8];							  // VCCVoltage
	data[11] = data[9];							  // VCCVoltage
	data[12] = static_cast<uint8_t>(m_railPower); // CentralState
	if (data[12] == static_cast<uint8_t>(EnergyState::csServiceMode))
		data[12] = 0x20;
	/*Bitmasken f�r CentralState:
		#define csEmergencyStop  0x01 // Der Nothalt ist eingeschaltet
		#define csTrackVoltageOff  0x02 // Die Gleisspannung ist abgeschaltet
		#define csShortCircuit  0x04 // Kurzschluss
		#define csProgrammingModeActive 0x20 // Der Programmiermodus ist aktiv
	*/
	data[13] = 0x00; // CentralStateEx
	/* Bitmasken f�r CentralStateEx:
		#define cseHighTemperature  0x01 // zu hohe Temperatur
		#define csePowerLost  0x02 // zu geringe Eingangsspannung
		#define cseShortCircuitExternal 0x04 // am externen Booster-Ausgang
		#define cseShortCircuitInternal 0x08 // am Hauptgleis oder Programmiergleis
		#define cseRCN213 0x20 // Weichenadressierung gem. RCN213
	*/
	data[14] = 0x00; // reserved
	data[15] = 0x01 | 0x02 | 0x10 | 0x20 | 0x40; // Capabilities 
	// Bitmasken für Capabilities:
	// #define capDCC 0x01 // beherrscht DCC
	// #define capMM 0x02 // beherrscht MM
	// //#define capReserved 0x04 // reserviert für zukünftige Erweiterungen
	// #define capRailCom 0x08 // RailCom ist aktiviert
	// #define capLocoCmds 0x10 // akzeptiert LAN-Befehle für Lokdecoder
	// #define capAccessoryCmds 0x20 // akzeptiert LAN-Befehle für Zubehördecoder
	// #define capDetectorCmds 0x40 // akzeptiert LAN-Befehle für Belegtmelder
	// #define capNeedsUnlockCode 0x80 // benötigt Freischaltcode (z21start)

	// only to the request client if or if client = 0 to all that select this message (Abo)!
	if (client > 0)
		EthSend(client, 0x14, Header::LAN_SYSTEMSTATE_DATACHANGED, data, false, static_cast<uint16_t>(BcFlagShort::Z21bcNone)); // only to the request client
	EthSend(0, 0x14, Header::LAN_SYSTEMSTATE_DATACHANGED, data, false, 0);														// static_cast<uint16_t>(BcFlagShort::Z21bcSystemInfo));	// all that select this message (Abo)
}

// Private Methods ///////////////////////////////////////////////////////////////////////////////////////////////////
// Functions only available to other functions in this library *******************************************************

//--------------------------------------------------------------------------------------------
void z21Interface::EthSend(uint8_t client, unsigned int DataLen, z21Interface::Header Header, uint8_t *dataString, boolean withXOR, uint16_t BC)
{
	uint8_t data[24]; // z21Interface send storage

	//--------------------------------------------
	// XOR bestimmen:
	data[0] = DataLen & 0xFF;
	data[1] = DataLen >> 8;
	data[2] = static_cast<uint8_t>(Header) & 0xFF;
	data[3] = static_cast<uint8_t>(static_cast<uint16_t>(Header) >> 8);
	data[DataLen - 1] = 0; // XOR

	for (uint8_t i = 0; i < (DataLen - 5 + !withXOR); i++)
	{ // Ohne Length und Header und XOR
		if (withXOR)
			data[DataLen - 1] = data[DataLen - 1] ^ *dataString;
		data[i + 4] = *dataString;
		dataString++;
	}
	//--------------------------------------------
	if (client > 0)
	{
		notifyz21InterfaceEthSend(client, data);
#ifdef DEBUG_SENDING
		if (m_debug)
		{
			ZDebug.print("CTX ");
			ZDebug.print(client);
			ZDebug.print(" : ");
			for (uint8_t x = 0; x < data[0]; x++)
			{
				ZDebug.print(data[x], HEX);
				ZDebug.print(" ");
			}
			ZDebug.println();
		}
#endif
	}
	else
	{
			// ZDebug.print("BTX ");
			// ZDebug.print(client);
			// ZDebug.print(" : ");
			// for (uint8_t x = 0; x < data[0]; x++)
			// {
			// 	ZDebug.print(data[x], HEX);
			// 	ZDebug.print(" ");
			// }
			// ZDebug.println();
		if (BC == 0) // client and flag is zero => Broadcast
		{
			notifyz21InterfaceEthSend(0, data);
		}
		else
		{

			uint8_t clientOut = client;
			for (uint8_t i = 0; i < z21InterfaceclientMAX; i++)
			{
				if ((ActIP[i].time > 0) && ((BC & ActIP[i].BCFlag) > 0))
				{ // Boradcast & Noch aktiv

					if (BC != 0)
					{
						if (BC == static_cast<uint16_t>(BcFlagShort::Z21bcAll))
							clientOut = 0; // ALL
						else
							clientOut = ActIP[i].client;
					}

					//--------------------------------------------
					notifyz21InterfaceEthSend(clientOut, data);
#ifdef DEBUG_SENDING
					if (m_debug)
					{
						ZDebug.print(i);
						ZDebug.print("BTX ");
						ZDebug.print(clientOut);
						ZDebug.print(" BC:");
						ZDebug.print(BC & ActIP[i].BCFlag, BIN);
						ZDebug.print(" : ");
						for (uint8_t x = 0; x < data[0]; x++)
						{
							ZDebug.print(data[x], HEX);
							ZDebug.print(" ");
						}
						ZDebug.println();
					}
#endif
					if (clientOut == 0)
						return;
				}
			}
		}
	}
}

//--------------------------------------------------------------------------------------------
// Convert local stored flag back into a Z21 Flag
uint32_t z21Interface::getz21InterfaceBcFlag(uint16_t flag)
{
	uint32_t outFlag = 0;
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcAll)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcAll);
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcRBus)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcRBus);
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcSystemInfo)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcSystemInfo);
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcNetAll)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcNetAll);
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcLocoNet)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcLocoNet);
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcLocoNetLocos)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcLocoNetLocos);
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcLocoNetSwitches)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcLocoNetSwitches);
	if ((flag & static_cast<uint16_t>(BcFlagShort::Z21bcLocoNetGBM)) != 0)
		outFlag |= static_cast<uint32_t>(BcFlag::Z21bcLocoNetGBM);
	return outFlag;
}

//--------------------------------------------------------------------------------------------
// Convert Z21 LAN BC flag to local stored flag
uint16_t z21Interface::getLocalBcFlag(uint32_t flag)
{
	uint16_t outFlag = 0;
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcAll)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcAll);
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcRBus)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcRBus);
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcSystemInfo)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcSystemInfo);
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcNetAll)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcNetAll);
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcLocoNet)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcLocoNet);
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcLocoNetLocos)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcLocoNetLocos);
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcLocoNetSwitches)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcLocoNetSwitches);
	if ((flag & static_cast<uint32_t>(BcFlag::Z21bcLocoNetGBM)) != 0)
		outFlag |= static_cast<uint16_t>(BcFlagShort::Z21bcLocoNetGBM);
	return outFlag;
}

//--------------------------------------------------------------------------------------------
// delete the stored IP-Address
void z21Interface::clearIP(uint8_t pos)
{
	ActIP[pos].client = 0;
	ActIP[pos].BCFlag = 0;
	ActIP[pos].time = 0;
	ActIP[pos].adr = 0;
}

//--------------------------------------------------------------------------------------------
void z21Interface::clearIPSlots()
{
	for (int i = 0; i < z21InterfaceclientMAX; i++)
		clearIP(i);
}

//--------------------------------------------------------------------------------------------
void z21Interface::clearIPSlot(uint8_t client)
{
	for (int i = 0; i < z21InterfaceclientMAX; i++)
	{
		if (ActIP[i].client == client)
		{
			clearIP(i);
			return;
		}
	}
}

//--------------------------------------------------------------------------------------------
uint16_t z21Interface::addIPToSlot(uint8_t client, uint16_t BCFlag)
{
	uint8_t Slot = z21InterfaceclientMAX;

	for (uint8_t i = 0; i < z21InterfaceclientMAX; i++)
	{
		if (ActIP[i].client == client)
		{
			ActIP[i].time = z21InterfaceActTimeIP;
			if (BCFlag != 0)
			{ // Falls BC Flag �bertragen wurde diesen hinzuf�gen!
				ActIP[i].BCFlag = BCFlag;
			}
			return ActIP[i].BCFlag; // BC Flag 4. Byte R�ckmelden
		}
		else if (ActIP[i].time == 0 && Slot == z21InterfaceclientMAX)
			Slot = i;
	}
	ActIP[Slot].client = client;
	ActIP[Slot].time = z21InterfaceActTimeIP;
	setPower(m_railPower);	   // inform the client with last power state
	return ActIP[Slot].BCFlag; // BC Flag 4. Byte R�ckmelden
}

//--------------------------------------------------------------------------------------------
// check if there are slots with the same loco, set them to busy
void z21Interface::setOtherSlotBusy(uint8_t slot)
{
	for (uint8_t i = 0; i < z21InterfaceclientMAX; i++)
	{
		if ((i != slot) && (ActIP[slot].adr == ActIP[i].adr))
		{					  // if in other Slot -> set busy
			ActIP[i].adr = 0; // clean slot that informed as busy & let it activ
							  // Inform with busy message:
							  // not used!
		}
	}
}

//--------------------------------------------------------------------------------------------
// Add loco to slot.
void z21Interface::addBusySlot(uint8_t client, uint16_t adr)
{
	for (uint8_t i = 0; i < z21InterfaceclientMAX; i++)
	{
		if (ActIP[i].client == client)
		{
			if (ActIP[i].adr != adr)
			{						 // skip is already used by this client
				ActIP[i].adr = adr;	 // store loco that is used
				setOtherSlotBusy(i); // make other busy
			}
			break;
		}
	}
}

//--------------------------------------------------------------------------------------------
// used by non Z21 client
void z21Interface::reqLocoBusy(uint16_t adr)
{
	for (uint8_t i = 0; i < z21InterfaceclientMAX; i++)
	{
		if (adr == ActIP[i].adr)
		{
			ActIP[i].adr = 0; // clear
		}
	}
}
