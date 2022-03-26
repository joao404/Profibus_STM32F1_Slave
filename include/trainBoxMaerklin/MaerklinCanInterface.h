/*********************************************************************
 * TrainBox Maerklin 
 *
 * Copyright (C) 2022 Marcel Maage
 * 
 * based on code by Joerg Pleumann
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
#include <Printable.h>
#include <String>
#include <array>

class TrackMessage : public Printable {

  public:
    /**
   * The message prio.
   */
  uint8_t prio;
  /**
   * The command number.
   */
  uint8_t command;
  
  /**
   * The hash that is used for avoiding device/message collisions.
   */
  uint16_t hash;

  /**
   * Whether this is a response to a request.
   */
  bool response;

  /**
   * The number of data bytes in the payload.
   */
  uint8_t length;

  /**
   * The actual message data bytes.
   */
  uint8_t data[8];

  /**
   * Clears the message, setting all values to zero. Provides for
   * easy recycling of TrackMessage objects.
   */
  void clear();

  /**
   * Prints the message to the given Print object, which could be a
   * Serial object, for instance. The message format looks like this
   *
   * HHHH R CC L DD DD DD DD DD DD DD DD
   *
   * with all numbers being hexadecimals and the data bytes being
   * optional beyond what the message length specifies. Exactly one
   * whitespace is inserted between different fields as a separator.
   */
  virtual size_t printTo(Print &p) const;

  /**
   * Parses the message from the given String. Returns true on
   * success, false otherwise. The message must have exactly the
   * format that printTo creates. This includes each and every
   * whitespace. If the parsing fails the state of the object is
   * undefined afterwards, and a clear() is recommended.
   */
  bool parseFrom(String &s);

};


class MaerklinCanInterface {

    public:
        enum class ProtocolLoco : uint8_t
        {
            MM2_2040 = 0, // Ansteuerung Loks mit 20kHz und FDEC mit 40 kHz
            MM2_LOK_20 = 1, // Nur Ansteuerung Loks mit 20 kHz
            MM2_FKT_40 = 2, // Nur Ansteuerung FDEC mit 40 kHz
            DCC_SHORT_28 = 0, // DCC Kurze Adresse, 28 Fahrstufen [=DCC-FS-Default]
            DCC_SHORT_14 = 1, // DCC Kurze Adresse, 14 Fahrstufen
            DCC_SHORT_128 = 2, // DCC Kurze Adresse, 126 Fahrstufen
            DCC_LONG_28 = 3, // DCC Lange Adresse, 28 Fahrstufen
            DCC_LONG_128 = 4 // DCC Lange Adresse, 126 Fahrstufen
        };

        enum class AddrOffset : uint16_t
        {
            MM2 = 0x0000, // MM2 locomotive
            SX1 = 0x0800, // Selectrix (old) locomotive
            MFX = 0x4000, // MFX locomotive
            SX2 = 0x8000, // Selectrix (new) locomotive
            DCC = 0xC000, // DCC locomotive
            SX1Acc = 0x2000, // Selectrix (old) magnetic accessory
            MM2Acc = 0x3000, // MM2 magnetic accessory
            DCCAcc = 0x3800 // DCC magnetic accessory
        };

        enum class MessagePrio : uint8_t
        {
            max = 0, // should not be used
            system = 1, // Stopp / Go / Kurzschluss-Meldung
            feedback = 2, // Rueckmeldungen
            locoStop = 3, // Lok anhalten
            locoAccCommand = 4, // loco /acc command
            noPrio = 5 // no prio
        };


        enum class Cmd : uint8_t
        {
            systemCmd = 0x00,
            locoDetection = 0x01,//TODO
            mfxBind = 0x02,//TODO
            mfxVerify = 0x03,//TODO
            locoSpeed = 0x04,
            locoDir = 0x05,
            locoFunc = 0x06,
            readConfig = 0x07,//TODO
            writeConfig = 0x08,//TODO
            accSwitch = 0x0B,
            accConfig = 0x0C,
            s88Pol = 0x10,//TODO
            s88Event = 0x11,//TODO
            sX1Event = 0x12,//TODO
            ping = 0x18,
            updateOffer = 0x19,//TODO
            readConfigData = 0x1A,//TODO
            bootloaderCanBind = 0x1B,//TODO
            bootloaderTrackBind = 0x1C,//TODO
            statusDataConfig = 0x1D,
            requestConfigData = 0x20,
            configDataSteam = 0x21,
            connect6021 = 0x22//TODO
        };

        enum class SubCmd : uint8_t
        {
            systemStop = 0x00,
            systemGo = 0x01,
            systemHalt = 0x02,
            locoStop = 0x03,
            locoRemoveCycle = 0x04,
            locoDataProtocol = 0x05,
            accTime = 0x06,
            fastReadMfx = 0x07,
            setTrackProtocol = 0x08,
            setMfxCounter = 0x09,
            systemOverLoad = 0x0A,
            systemStatus = 0x0B,
            systemIdent = 0x0C,
            //mfxSeek = 0x30,
            systemReset = 0x18
        };

        enum class valueChannel : uint8_t
        {
            current = 1,
            voltage = 3,
            temp = 4
        };

    protected:

        MaerklinCanInterface(word hash, bool debug);

        virtual ~MaerklinCanInterface();

	    uint16_t m_hash;

	    bool m_debug;

	    virtual void begin();

        void generateHash();

        uint16_t getHash();

        bool isDebug();

        virtual bool sendMessage(TrackMessage &message) = 0;

        virtual bool receiveMessage(TrackMessage &message) = 0;

        virtual void end() = 0;

        void handleReceivedMessage(TrackMessage &message);


        //onCallback
        virtual bool onSystemStop(uint32_t id){return false;}

        virtual bool onSystemGo(uint32_t id){return false;}

        virtual bool onSystemHalt(uint32_t id){return false;}

        virtual bool onLocoStop(uint32_t id){return false;}

        virtual bool onLocoRemoveCycle(uint32_t id){return false;}

        virtual bool onLocoDataProtocol(uint32_t id, ProtocolLoco protocol){return false;}

        virtual bool onAccTime(uint32_t id, uint16_t accTimeIN10ms){return false;}

        virtual bool onFastReadMfx(uint32_t id, uint16_t mfxSid){return false;}

        virtual bool onTrackProtocol(uint32_t id, uint8_t param){return false;}

        virtual bool onMfxCounter(uint32_t id, uint16_t counter){return false;}

        virtual bool onSystemOverLoad(uint32_t id, uint8_t channel){return false;}

        virtual bool onSystemStatus(uint32_t id, uint8_t channel, bool valid){return false;}
        
        virtual bool onSystemStatus(uint32_t id, uint8_t channel, uint16_t value){return false;}

        virtual bool onSystemIdent(uint32_t id, uint16_t feedbackId){return false;}

        virtual bool onSystemReset(uint32_t id, uint8_t target){return false;}



        virtual bool onLocoSpeed(uint32_t id){return false;}

        virtual bool onLocoSpeed(uint32_t id, uint16_t speed){return false;}

        //0 = Fahrtrichtung bleibt
        //1 = Fahrtrichtung vorwärts
        //2 = Fahrtrichtung rückwärts
        //3 = Fahrtrichtung umschalten
        virtual bool onLocoDir(uint32_t id, uint8_t dir){return false;}

        virtual bool onLocoFunc(uint32_t id, uint8_t function, uint8_t value){return false;}

        virtual bool onReadConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool readSuccessful){return false;};

        virtual bool onWriteConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool writeSuccessful, bool verified){return false;};

        virtual bool onAccSwitch(uint32_t id, uint8_t position, uint8_t current){return false;}

        virtual bool onPing(uint32_t id, uint16_t swVersion, uint16_t hwIdent){return false;}

        virtual bool onStatusDataConfig(uint16_t hash, std::array<uint8_t, 8>& data){return false;}

        virtual bool onStatusDataConfig(uint16_t hash, uint32_t uid, uint8_t index, uint8_t length){return false;}

        virtual bool onConfigData(std::array<uint8_t, 8> data){return false;}

        virtual bool onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc){return false;}

        virtual bool onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc, uint8_t res){return false;}

        virtual bool onConfigDataStream(uint16_t hash, std::array<uint8_t, 8>& data){return false;}

        virtual bool onConfigDataSteamError(uint16_t hash){return false;}

    public:
        void messageSystemStop(TrackMessage& message, uint32_t uid = 0);

        void messageSystemGo(TrackMessage& message, uint32_t uid = 0);

        void messageSystemHalt(TrackMessage& message, uint32_t uid = 0);

        void messageLocoStop(TrackMessage& message, uint32_t uid = 0);

        void messageLocoRemoveCycle(TrackMessage& message, uint32_t uid = 0);

        void messageLocoDataProtocol(TrackMessage& message, uint32_t uid, ProtocolLoco protocol);

        void messageAccTime(TrackMessage& message, uint16_t accTimeIN10ms, uint32_t uid = 0);

        void messageFastReadMfx(TrackMessage& message, uint16_t mfxSid, uint32_t uid = 0);

        void messageSetTrackProtocol(TrackMessage& message, uint8_t protocols, uint32_t uid = 0);

        void messageSetMfxCounter(TrackMessage& message, uint16_t counter, uint32_t uid = 0);

        void messageSystemStatus(TrackMessage& message, uint8_t channelNumber, uint32_t uid = 0);

        void messageSystemStatus(TrackMessage& message, uint8_t channelNumber, uint16_t configuration, uint32_t uid = 0);

        void messageSetSystemIdent(TrackMessage& message, uint16_t systemIdent, uint32_t uid = 0);

        void messageSystemReset(TrackMessage& message, uint8_t resetTarget, uint32_t uid = 0);

        void messageLocoSpeed(TrackMessage& message, uint32_t uid);

        void messageLocoSpeed(TrackMessage& message, uint32_t uid, uint16_t speed);

        void messageLocoDir(TrackMessage& message, uint32_t uid);

        void messageLocoDir(TrackMessage& message, uint32_t uid, uint8_t dir);

        void messageLocoFunc(TrackMessage& message, uint32_t uid, uint8_t function);

        void messageLocoFunc(TrackMessage& message, uint32_t uid, uint8_t function, uint8_t value);

        void messageReadConfig(TrackMessage& message, uint32_t id, uint16_t cvAdr, uint8_t number);

        void messageWriteConfig(TrackMessage& message, uint32_t id, uint16_t cvAdr, uint8_t value, bool directProc, bool writeByte);

        void messageAccSwitch(TrackMessage& message, uint32_t uid, uint8_t position, uint8_t current);

        void messageAccSwitch(TrackMessage& message, uint32_t uid, uint8_t position, uint8_t current, uint16_t switchTimeIN10ms);

        void messagePing(TrackMessage &message);

        void messagePing(TrackMessage& message, uint32_t uid, uint16_t swVersion, uint16_t hwIdent);

        void messageStatusDataConfig(TrackMessage &message, uint32_t uid, uint8_t index);

        void messageConfigData(TrackMessage& message, std::array<uint8_t, 8>& request);


        bool exchangeMessage(TrackMessage &out, TrackMessage &in,  word timeout);

        bool sendSystemStop(uint32_t uid = 0);

        bool sendSystemGo(uint32_t uid = 0);

        bool sendSystemHalt(uint32_t uid = 0);

        bool sendLocoStop(uint32_t uid = 0);

        bool sendLocoRemoveCycle(uint32_t uid = 0);

        bool sendLocoDataProtocol(uint32_t uid, ProtocolLoco protocol);

        bool sendAccTime(uint16_t accTimeIN10ms, uint32_t uid = 0);

        // mfxSid must be checked by mfx verify before usage. mfxSid must be inside mfx range starting with 0x7F
        bool sendFastReadMfx(uint16_t mfxSid, uint32_t uid = 0);

        bool sendSetTrackProtocol(uint8_t protocols, uint32_t uid = 0);

        bool sendSetMfxCounter(uint16_t counter, uint32_t uid = 0);

        bool sendSystemStatus(uint8_t channelNumber, uint32_t uid = 0);

        bool sendSystemStatus(uint8_t channelNumber, uint16_t configuration, uint32_t uid = 0);

        bool sendSetSystemIdent(uint16_t systemIdent, uint32_t uid = 0);

        bool sendSystemReset(uint8_t resetTarget, uint32_t uid = 0);

        bool requestLocoSpeed(uint32_t uid);

        // value speed between 0 and 1024
        bool setLocoSpeed(uint32_t uid, uint16_t speed);

        bool requestLocoDir(uint32_t uid);

        //if direction changes, speed is set to zero
        //0 = Fahrtrichtung bleibt
        //1 = Fahrtrichtung vorwärts
        //2 = Fahrtrichtung rückwärts
        //3 = Fahrtrichtung umschalten
        bool setLocoDir(uint32_t uid, uint8_t dir);

        bool requestLocoFunc(uint32_t uid, uint8_t function);

        bool setLocoFunc(uint32_t uid, uint8_t function, uint8_t value);

        bool sendReadConfig(uint32_t id, uint16_t cvAdr, uint8_t number);

        bool sendWriteConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool directProc, bool writeByte);

        bool setAccSwitch(uint32_t uid, uint8_t position, uint8_t current);

        bool setAccSwitch(uint32_t uid, uint8_t position, uint8_t current, uint16_t switchTimeIN10ms);

        bool sendPing();

        bool sendPing(uint32_t uid, uint16_t swVersion, uint16_t hwIdent);

        bool requestStatusDataConfig(uint32_t uid, uint8_t index);

        bool requestConfigData(std::array<uint8_t, 8>& request);
};