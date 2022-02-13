/*
  z21Interface.h - library for Z21 mobile protocoll
  This file is based on code of Philipp Gahtow
  Copyright (c) 2013-2021 Philipp Gahtow  All right reserved.

  ROCO Z21 LAN Protocol for Arduino.

  Notice:
	- analyse the data and give back the content and a answer

 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

  Grundlage: Z21 LAN Protokoll Spezifikation V1.11
*/
// include types & constants of Wiring core API
#include <Arduino.h>
#include <array>

//**************************************************************
#define ZDebug Serial // Port for the Debugging
// #define DEBUG_SENDING

// #define directResponse
//--------------------------------------------------------------
#define z21InterfaceclientMAX 30	// Speichergroesse for IP-Adressen
#define z21InterfaceActTimeIP 20	// Aktivhaltung einer IP for (sec./2)
#define z21InterfaceIPinterval 2000 // interval at milliseconds

struct TypeActIP
{
	uint8_t client; // Byte client
	uint16_t BCFlag; // BroadCastFlag
	uint8_t time;	// Zeit
	uint16_t adr;	// Loco control Adr
};

// library interface description
class z21Interface
{
public:
	enum class HwType : uint16_t
	{
		Z21_OLD = 0x00000200,			 // schwarze Z21 (Hardware-Variante ab 2012)
		Z21_NEW = 0x00000201,			 // schwarze Z21(Hardware-Variante ab 2013)
		SMARTRAIL = 0x00000202,			 // SmartRail (ab 2012)
		Z21Interface_SMALL = 0x00000203, // weisse z21Interface Starterset-Variante (ab 2013)
		Z21Interface_START = 0x00000204, // z21Interface start Starterset-Variante (ab 2016)
		Z21_XL = 0x00000211,			 // 10870 Z21 XL Series (ab 2020)
		SINGLE_BOOSTER = 0x00000205,	 // 10806 Z21 Single Booster (zLink)
		DUAL_BOOSTER = 0x00000206,		 // 10807 Z21 Dual Booster (zLink)
		Z21_SWITCH_DECODER = 0x00000301, // 10836 Z21 SwitchDecoder (zLink)
		Z21_SIGNAL_DECODER = 0x00000302	 // 10836 Z21 SignalDecoder (zLink)
	};

protected:
	enum class Header : uint16_t
	{
		LAN_GET_SERIAL_NUMBER = 0x10,
		LAN_LOGOFF = 0x30,
		LAN_X_HEADER = 0x40,
		LAN_SET_BROADCASTFLAGS = 0x50,
		LAN_GET_BROADCASTFLAGS = 0x51,
		LAN_SYSTEMSTATE_DATACHANGED = 0x84,
		LAN_SYSTEMSTATE_GETDATA = 0x85, // AW: LAN_SYSTEMSTATE_DATACHANGED
		LAN_GET_HWINFO = 0x1A,
		LAN_GET_CODE = 0x18,

		LAN_GET_CONFIG1 = 0x12,
		LAN_SET_CONFIG1 = 0x13,
		LAN_GET_CONFIG2 = 0x16,
		LAN_SET_CONFIG2 = 0x17,

		LAN_GET_LOCOMODE = 0x60,
		LAN_SET_LOCOMODE = 0x61,
		LAN_GET_TURNOUTMODE = 0x70,
		LAN_SET_TURNOUTMODE = 0x71,

		LAN_RMBUS_DATACHANGED = 0x80,
		LAN_RMBUS_GETDATA = 0x81,
		LAN_RMBUS_PROGRAMMODULE = 0x82,

		LAN_RAILCOM_DATACHANGED = 0x88,
		LAN_RAILCOM_GETDATA = 0x89,

		LAN_LOCONET_Z21_RX = 0xA0,
		LAN_LOCONET_Z21_TX = 0xA1,
		LAN_LOCONET_FROM_LAN = 0xA2,
		LAN_LOCONET_DISPATCH_ADDR = 0xA3,
		LAN_LOCONET_DETECTOR = 0xA4,

		LAN_CAN_DETECTOR = 0xC4
	};

	enum class XHeader : uint8_t
	{
		LAN_X_GET_SETTING = 0x21,
		LAN_X_BC_TRACK_POWER = 0x61,
		LAN_X_UNKNOWN_COMMAND = 0x61,
		LAN_X_STATUS_CHANGED = 0x62,
		LAN_X_GET_VERSION = 0x63, // AW: X-Bus Version 090040006321301260
		LAN_X_SET_STOP = 0x80,	  // AW: LAN_X_BC_STOPPED
		LAN_X_BC_STOPPED = 0x81,
		LAN_X_GET_FIRMWARE_VERSION = 0xF1, // AW: 0xF3

		LAN_X_GET_LOCO_INFO = 0xE3,
		LAN_X_SET_LOCO_DRIVE = 0xE4,	// X-Header
		LAN_X_SET_LOCO_FUNCTION = 0xF8, // DB0
		LAN_X_LOCO_INFO = 0xEF,

		LAN_X_GET_TURNOUT_INFO = 0x43,
		LAN_X_SET_TURNOUT = 0x53,
		LAN_X_TURNOUT_INFO = 0x43,

		LAN_X_SET_EXT_ACCESSORY = 0x54,		 // new: 1.10
		LAN_X_GET_EXT_ACCESSORY_INFO = 0x44, // new: 1.10

		LAN_X_CV_READ = 0x23,
		LAN_X_CV_WRITE = 0x24,
		LAN_X_CV_NACK_SC = 0x61,
		LAN_X_CV_NACK = 0x61,
		LAN_X_CV_RESULT = 0x64,

		LAN_X_CV_POM = 0xE6, // X-Header

		// ab Z21 FW Version 1.23
		LAN_X_MM_WRITE_BYTE = 0x24,

		// ab Z21 FW Version 1.25
		LAN_X_DCC_READ_REGISTER = 0x22,
		LAN_X_DCC_WRITE_REGISTER = 0x23
	};

	enum class BcFlag : uint32_t
	{
		// Z21 BC Flags
		Z21bcNone = 0x00000000,
		Z21bcAll = 0x00000001,
		Z21bcRBus = 0x00000002,
		Z21bcRailcom = 0x00000004, // RailCom-Daten für Abo Loks
		Z21bcRailcom_s = 0x100,

		Z21bcSystemInfo = 0x00000100, // LAN_SYSTEMSTATE_DATACHANGED

		// ab FW Version 1.20:
		Z21bcNetAll = 0x00010000, // Alles, auch alle Loks ohne vorher die Lokadresse abonnieren zu müssen (für PC Steuerung)

		Z21bcLocoNet = 0x01000000,		   // LocoNet Meldungen an LAN Client weiterleiten (ohne Loks und Weichen)
		Z21bcLocoNetLocos = 0x02000000,	   // Lok-spezifische LocoNet Meldungen an LAN Client weiterleiten
		Z21bcLocoNetSwitches = 0x04000000, // Weichen-spezifische LocoNet Meldungen an LAN Client weiterleiten

		// ab FW Version 1.22:
		Z21bcLocoNetGBM = 0x08000000, // Status-Meldungen von Gleisbesetztmeldern am LocoNet-Bus

		// ab FW Version 1.29:
		Z21bcRailComAll = 0x00040000, // alles: Änderungen bei RailCom-Daten ohne Lok Abo! -> LAN_RAILCOM_DATACHANGED

		// ab FW Version 1.30:
		Z21bcCANDetector = 0x00080000, // Meldungen vom Gelisbesetztmeldern am CAN-Bus
	};

	enum class BcFlagShort : uint16_t
	{
		// Z21 BC Flags
		Z21bcNone = 0x0000,
		Z21bcAll = 0x0001,
		Z21bcRBus = 0x0002,
		Z21bcRailcom = 0x0004, // RailCom-Daten für Abo Loks

		Z21bcSystemInfo = 0x0008, // LAN_SYSTEMSTATE_DATACHANGED

		// ab FW Version 1.20:
		Z21bcNetAll = 0x0010, // Alles, auch alle Loks ohne vorher die Lokadresse abonnieren zu müssen (für PC Steuerung)

		Z21bcLocoNet = 0x0020,		   // LocoNet Meldungen an LAN Client weiterleiten (ohne Loks und Weichen)
		Z21bcLocoNetLocos = 0x0040,	   // Lok-spezifische LocoNet Meldungen an LAN Client weiterleiten
		Z21bcLocoNetSwitches = 0x0080, // Weichen-spezifische LocoNet Meldungen an LAN Client weiterleiten

		// ab FW Version 1.22:
		Z21bcLocoNetGBM = 0x0100, // Status-Meldungen von Gleisbesetztmeldern am LocoNet-Bus

		// ab FW Version 1.29:
		Z21bcRailComAll = 0x0200, // alles: Änderungen bei RailCom-Daten ohne Lok Abo! -> LAN_RAILCOM_DATACHANGED

		// ab FW Version 1.30:
		Z21bcCANDetector = 0x0400, // Meldungen vom Gelisbesetztmeldern am CAN-Bus
	};

	enum class StepConfig : uint8_t
	{
		Step14 = 0x01,
		Step28 = 0x02,
		Step128 = 0x03
	};

	enum class EnergyState : uint8_t
	{
		csNormal = 0x00,		   	// Normal Operation Resumed ist eingeschaltet
		csEmergencyStop = 0x01,   	// Der Nothalt ist eingeschaltet
		csTrackVoltageOff = 0x02, 	// Die Gleisspannung ist abgeschaltet
		csShortCircuit = 0x04,	   	// Kurzschluss
		csServiceMode = 0x08	   	// Der Programmiermodus ist aktiv - Service Mode
	};

	enum class CentralState : uint8_t
	{
		cseHighTemperature = 0x01,		 // zu hohe Temperatur
		csePowerLost = 0x02,			 // zu geringe Eingangsspannung
		cseShortCircuitExternal = 0x04,  // am externen Booster-Ausgang
		cseShortCircuitInternal = 0x08	 // am Hauptgleis oder Programmiergleis
	};

	// user-accessible "public" interface
public:
	z21Interface(HwType hwType, uint16_t swVersion, boolean debug); // Constuctor

	void receive(uint8_t client, uint8_t *packet); // Pr�fe auf neue Ethernet Daten

	void setPower(EnergyState state); // Zustand Gleisspannung Melden
	EnergyState getPower();			  // Zusand Gleisspannung ausgeben

	void setCVPOMBYTE(uint16_t CVAdr, uint8_t value); // POM write uint8_t return

	void setLocoStateExt(int Adr);					   // send Loco state to BC
	uint32_t getz21InterfaceBcFlag(uint16_t flag); // Convert local stored flag back into a Z21 Flag

	void setS88Data(uint8_t *data); // return state of S88 sensors

	void setLNDetector(uint8_t client, uint8_t *data, uint8_t DataLen);			// return state from LN detector
	void setLNMessage(uint8_t *data, uint8_t DataLen, uint8_t bcType, bool TX); // return LN Message

	void setCANDetector(uint16_t NID, uint16_t Adr, uint8_t port, uint8_t typ, uint16_t v1, uint16_t v2); // state from CAN detector

	void setTrntInfo(uint16_t Adr, bool State); // Return the state of accessory

	void setExtACCInfo(uint16_t Adr, uint8_t State, bool Status = 0x00); // Return EXT Accessory INFO

	void setCVReturn(uint16_t CV, uint8_t value); // Return CV Value for Programming
	void setCVNack();							  // Return no ACK from Decoder
	void setCVNackSC();							  // Return Short while Programming

	void sendSystemInfo(uint8_t client, uint16_t maincurrent, uint16_t mainvoltage, uint16_t temp); // Send to all clients that request via BC the System Information

	// library-accessible "private" interface
private:
	HwType m_hwType;

	uint16_t m_swVersion;

	// Variables:
	EnergyState m_railPower;					// state of the railpower
	long z21InterfaceIPpreviousMillis;		// will store last time of IP decount updated
	TypeActIP ActIP[z21InterfaceclientMAX]; // Speicherarray for IPs

	// Functions:
	void returnLocoStateFull(uint8_t client, uint16_t Adr, bool bc); // Antwort auf Statusabfrage
	uint16_t getLocalBcFlag(uint32_t flag);							 // Convert Z21 LAN BC flag to local stored flag
	void clearIP(uint8_t pos);										 // delete the stored client
	void clearIPSlots();											 // delete all stored clients
	void clearIPSlot(uint8_t client);								 // delete a client
	uint16_t addIPToSlot(uint8_t client, uint16_t BCFlag);

	void setOtherSlotBusy(uint8_t slot);
	void addBusySlot(uint8_t client, uint16_t adr);
	void reqLocoBusy(uint16_t adr);

protected:
	boolean m_debug;

	void EthSend(uint8_t client, unsigned int DataLen, z21Interface::Header Header, uint8_t *dataString, boolean withXOR, uint16_t BC);

	virtual uint16_t getSerialNumber() = 0;

	virtual bool getConfig1(std::array<uint8_t, 10>& config){return true;};

	virtual void setConfig1(std::array<uint8_t, 10>& config){};

	virtual bool getConfig2(std::array<uint8_t, 16>& config){return true;};

	virtual void setConfig2(std::array<uint8_t, 16>& config){};

	virtual void handleGetLocoMode(uint16_t adr, uint8_t &mode){};
	virtual void handleSetLocoMode(uint16_t adr, uint8_t mode){};
	virtual void handleGetTurnOutMode(uint16_t adr, uint8_t &mode){};
	virtual void handleSetTurnOutMode(uint16_t adr, uint8_t mode){};

	virtual void notifyz21InterfacegetSystemInfo(uint8_t client){};

	virtual void notifyz21InterfaceEthSend(uint8_t client, uint8_t *data) = 0;

	virtual void notifyz21InterfaceLNdetector(uint8_t client, uint8_t typ, uint16_t Adr){};
	virtual uint8_t notifyz21InterfaceLNdispatch(uint16_t Adr) { return 0; };
	virtual void notifyz21InterfaceLNSendPacket(uint8_t *data, uint8_t length){};

	virtual void notifyz21InterfaceCANdetector(uint8_t client, uint8_t typ, uint16_t ID){};

	virtual void notifyz21InterfaceRailPower(EnergyState State){};

	virtual void notifyz21InterfaceCVREAD(uint8_t cvAdrMSB, uint8_t cvAdrLSB){};
	virtual void notifyz21InterfaceCVWRITE(uint8_t cvAdrMSB, uint8_t cvAdrLSB, uint8_t value){};
	virtual void notifyz21InterfaceCVPOMWRITEBYTE(uint16_t Adr, uint16_t cvAdr, uint8_t value){};
	virtual void notifyz21InterfaceCVPOMWRITEBIT(uint16_t Adr, uint16_t cvAdr, uint8_t value){};
	virtual void notifyz21InterfaceCVPOMREADBYTE(uint16_t Adr, uint16_t cvAdr){};

	virtual void notifyz21InterfaceMMWRITE(uint8_t regAdr, uint8_t value){};
	virtual void notifyz21InterfaceDCCWRITE(uint8_t regAdr, uint8_t value){};
	virtual void notifyz21InterfaceDCCREAD(uint8_t regAdr){};

	virtual void notifyz21InterfaceAccessoryInfo(uint16_t Adr, uint8_t &position){};
	virtual void notifyz21InterfaceAccessory(uint16_t Adr, bool state, bool active){};

	virtual void notifyz21InterfaceExtAccessory(uint16_t Adr, uint8_t state){};

	virtual void notifyz21InterfaceLocoState(uint16_t Adr, uint8_t data[]){};
	virtual void notifyz21InterfaceLocoFkt(uint16_t Adr, uint8_t type, uint8_t fkt){};
	virtual void notifyz21InterfaceLocoSpeed(uint16_t Adr, uint8_t speed, uint8_t stepConfig){};

	virtual void notifyz21InterfaceS88Data(uint8_t gIndex){}; // return last state S88 Data for the Client!

	virtual uint16_t notifyz21InterfaceRailcom() { return 0; }; // return global Railcom Adr

	virtual void notifyz21InterfaceUpdateConf(){}; // information for DCC via EEPROM (RailCom, ProgMode,...)
};
