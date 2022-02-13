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

#include "trainBoxMaerklin/TrainBoxMaerklin.h"

size_t printHex(Print &p, unsigned long hex, int digits);
int parseHex(String &s, int start, int end, bool *ok);

TrainBoxMaerklin::TrainBoxMaerklin(word hash, bool debug)
	: m_hash(hash),
	  m_debug(debug)
{
	if (m_debug)
	{
		Serial.println(F("Creating TrainBoxMaerklin"));
	}
}

TrainBoxMaerklin::~TrainBoxMaerklin()
{
	if (m_debug)
	{
		Serial.println(F("Destroying TrainBoxMaerklin"));
	}
}

void TrainBoxMaerklin::begin()
{

	// send init message
	TrackMessage message;
	message.clear();
	message.command = 0x1b;
	message.length = 0x05;
	message.data[4] = 0x11;
	sendMessage(message);

	if (m_hash == 0)
	{
		generateHash();
	}
}

uint16_t TrainBoxMaerklin::getHash()
{
	return m_hash;
}

bool TrainBoxMaerklin::isDebug()
{
	return m_debug;
}

void TrainBoxMaerklin::generateHash()
{
	TrackMessage message;

	bool ok = false;

	while (!ok)
	{
		m_hash = (random(0x10000) & 0xff7f) | 0x0300;

		if (m_debug)
		{
			Serial.print(F("### Trying new hash "));
			printHex(Serial, m_hash, 4);
			Serial.println();
		}

		message.clear();
		message.command = 0x18;

		sendMessage(message);

		delay(500);

		ok = true;
		unsigned long time = millis();
		while (receiveMessage(message))
		{
			if (message.hash == m_hash)
			{
				ok = false;
				break;
			}
			if (millis() < time + 2000)
			{
				break;
			}
		}
	}

	if (m_debug)
	{
		Serial.print(F("### New hash "));
		Serial.print(m_hash, HEX);
		Serial.println(F(" looks good"));
	}
}

bool TrainBoxMaerklin::exchangeMessage(TrackMessage &out, TrackMessage &in, word timeout)
{
	int command = out.command;

	if (!sendMessage(out))
	{
		if (m_debug)
		{
			Serial.println(F("!!! Send error"));
			Serial.println(F("!!! Emergency stop"));
			// setPower(false);
		}
		return false;
	}

	unsigned long time = millis();

	// TrackMessage response;

	while (millis() < time + timeout)
	{
		in.clear();
		bool result = receiveMessage(in);

		if (result && in.command == command && in.response)
		{
			return true;
		}
		else if (result)
		{
			handleReceivedMessage(in);
		}
	}

	if (m_debug)
	{
		Serial.println(F("!!! Receive timeout"));
	}

	return false;
}

void TrainBoxMaerklin::handleReceivedMessage(TrackMessage &message)
{
	// Serial.print("==> ");
	// Serial.println(message);
	bool messageHandled{false};
	// check message if it is a response or not and call callbacks
	if (message.response)
	{
		switch (static_cast<TrainBoxMaerklin::Cmd>(message.command))
		{
		case TrainBoxMaerklin::Cmd::systemCmd:
			switch (static_cast<TrainBoxMaerklin::SubCmd>(message.data[4]))
			{
			case TrainBoxMaerklin::SubCmd::systemStop:
				if (5 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onSystemStop(id);
				}
				break;
			case TrainBoxMaerklin::SubCmd::systemGo:
				if (5 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onSystemGo(id);
				}
				break;
			case TrainBoxMaerklin::SubCmd::systemHalt:
				if (5 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onSystemHalt(id);
				}
				break;
			case TrainBoxMaerklin::SubCmd::locoStop:
				if (5 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onLocoStop(id);
				}
				break;
			case TrainBoxMaerklin::SubCmd::locoRemoveCycle:
				if (5 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onLocoRemoveCycle(id);
				}
				break;
			case TrainBoxMaerklin::SubCmd::locoDataProtocol:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onLocoDataProtocol(id, static_cast<ProtocolLoco>(message.data[5]));
				}
				break;
			case TrainBoxMaerklin::SubCmd::accTime:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t accTimeIN10ms = (message.data[5] << 8) + message.data[6];
					messageHandled = onAccTime(id, accTimeIN10ms);
				}
				break;
			case TrainBoxMaerklin::SubCmd::fastReadMfx:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t mfxSid = (message.data[5] << 8) + message.data[6];
					messageHandled = onFastReadMfx(id, mfxSid);
				}
				break;
			case TrainBoxMaerklin::SubCmd::setTrackProtocol:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onTrackProtocol(id, message.data[5]);
				}
				break;
			case TrainBoxMaerklin::SubCmd::setMfxCounter:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t counter = (message.data[5] << 8) + message.data[6];
					messageHandled = onMfxCounter(id, counter);
				}
				break;
			case TrainBoxMaerklin::SubCmd::systemOverLoad:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onSystemOverLoad(id, message.data[5]);
				}
				break;
			case TrainBoxMaerklin::SubCmd::systemStatus:
				if (7 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint8_t channel = message.data[5];
					bool valid = (0x01 == message.data[6]);
					messageHandled = onSystemStatus(id, channel, valid);
				}
				else if (8 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint8_t channel = message.data[5];
					uint16_t value = (message.data[6] << 8) + message.data[7];
					messageHandled = onSystemStatus(id, channel, value);
				}
				break;
			case TrainBoxMaerklin::SubCmd::systemIdent:
				if (7 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t feedbackId = (message.data[5] << 8) + message.data[6];
					messageHandled = onSystemIdent(id, feedbackId);
				}
				break;
			case TrainBoxMaerklin::SubCmd::systemReset:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onSystemReset(id, message.data[5]);
				}
				break;
			default:
				break;
			}
			break;
		case TrainBoxMaerklin::Cmd::locoSpeed:
			if (4 == message.length) // locomotive is not known
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				messageHandled = onLocoSpeed(id);
			}
			else if (6 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint16_t speed = (message.data[4] << 8) + message.data[5];
				messageHandled = onLocoSpeed(id, speed);
			}
			break;
		case TrainBoxMaerklin::Cmd::locoDir:
			if (5 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint8_t dir = message.data[4];
				messageHandled = onLocoDir(id, dir);
			}
			break;
		case TrainBoxMaerklin::Cmd::locoFunc:
			if (6 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint8_t function = message.data[4];
				uint8_t value = message.data[5];
				messageHandled = onLocoFunc(id, function, value);
			}
			break;
		case TrainBoxMaerklin::Cmd::writeConfig:
			if (8 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint16_t cvAdr = (message.data[4] << 8) + message.data[5];
				uint8_t value = message.data[6];
				bool writeSuccessful = message.data[7] & 0x80;
				bool verified = message.data[7] & 0x40;
				messageHandled = onWriteConfig(id, cvAdr, value, writeSuccessful, verified);
			}
			break;

		case TrainBoxMaerklin::Cmd::accSwitch:
			if (6 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint8_t position = message.data[4];
				uint8_t current = message.data[5];
				messageHandled = onAccSwitch(id, position, current);
			}
			else if (8 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint8_t position = message.data[4];
				uint8_t current = message.data[5];
				messageHandled = onAccSwitch(id, position, current);
			}
			break;
		case TrainBoxMaerklin::Cmd::ping:
			if (8 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint16_t swVersion = (static_cast<uint16_t>(message.data[4]) << 8) + message.data[5];
				uint16_t hwIdent = (static_cast<uint16_t>(message.data[6]) << 8) + message.data[7];
				messageHandled = onPing(id, swVersion, hwIdent);
			}
			break;

		default:

			break;
		}
	}

	else
	{
		switch (static_cast<TrainBoxMaerklin::Cmd>(message.command))
		{
		case TrainBoxMaerklin::Cmd::systemCmd:
			switch (static_cast<TrainBoxMaerklin::SubCmd>(message.data[4]))
			{
			case TrainBoxMaerklin::SubCmd::systemOverLoad:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onSystemOverLoad(id, message.data[5]);
				}
				break;
			default:
				break;
			}
			break;

		default:

			break;
		}
	}

	if (messageHandled)
	{
	}
}

void TrainBoxMaerklin::messageSystemStop(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::systemStop);
}

void TrainBoxMaerklin::messageSystemGo(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::systemGo);
}

void TrainBoxMaerklin::messageSystemHalt(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::systemHalt);
}

void TrainBoxMaerklin::messageLocoStop(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoStop);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::locoStop);
}

void TrainBoxMaerklin::messageLocoRemoveCycle(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::locoRemoveCycle);
}

void TrainBoxMaerklin::messageLocoDataProtocol(TrackMessage &message, uint32_t uid, ProtocolLoco protocol)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::locoDataProtocol);
	message.data[5] = static_cast<uint8_t>(protocol);
}

void TrainBoxMaerklin::messageAccTime(TrackMessage &message, uint16_t accTimeIN10ms, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::accTime);
	message.data[5] = highByte(accTimeIN10ms);
	message.data[6] = lowByte(accTimeIN10ms);
}

void TrainBoxMaerklin::messageFastReadMfx(TrackMessage &message, uint16_t mfxSid, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::fastReadMfx);
	message.data[5] = highByte(mfxSid);
	message.data[6] = lowByte(mfxSid);
}

void TrainBoxMaerklin::messageSetTrackProtocol(TrackMessage &message, uint8_t protocols, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::setTrackProtocol);
	message.data[5] = protocols;
}

void TrainBoxMaerklin::messageSetMfxCounter(TrackMessage &message, uint16_t counter, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::setMfxCounter);
	message.data[5] = highByte(counter);
	message.data[6] = lowByte(counter);
}

void TrainBoxMaerklin::messageSystemStatus(TrackMessage &message, uint8_t channelNumber, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::systemStatus);
	message.data[5] = channelNumber;
}

void TrainBoxMaerklin::messageSystemStatus(TrackMessage &message, uint8_t channelNumber, uint16_t configuration, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x08;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::systemStatus);
	message.data[5] = channelNumber;
	message.data[6] = highByte(configuration);
	message.data[7] = lowByte(configuration);
}

void TrainBoxMaerklin::messageSetSystemIdent(TrackMessage &message, uint16_t systemIdent, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::systemIdent);
	message.data[5] = highByte(systemIdent);
	message.data[6] = lowByte(systemIdent);
}

void TrainBoxMaerklin::messageSystemReset(TrackMessage &message, uint8_t resetTarget, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(TrainBoxMaerklin::SubCmd::systemReset);
	message.data[5] = resetTarget;
}

// ===================================================================
// === LocoCmd =======================================================
// ===================================================================

void TrainBoxMaerklin::messageLocoSpeed(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::locoSpeed);
	message.length = 0x04;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
}

void TrainBoxMaerklin::messageLocoSpeed(TrackMessage &message, uint32_t uid, uint16_t speed)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::locoSpeed);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = highByte(speed);
	message.data[5] = lowByte(speed);
}

void TrainBoxMaerklin::messageLocoDir(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::locoDir);
	message.length = 0x04;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
}

void TrainBoxMaerklin::messageLocoDir(TrackMessage &message, uint32_t uid, uint8_t dir)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::locoDir);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = dir;
}

void TrainBoxMaerklin::messageLocoFunc(TrackMessage &message, uint32_t uid, uint8_t function)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::locoFunc);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = function;
}

void TrainBoxMaerklin::messageLocoFunc(TrackMessage &message, uint32_t uid, uint8_t function, uint8_t value)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::locoFunc);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = function;
	message.data[5] = value;
}

void TrainBoxMaerklin::messageWriteConfig(TrackMessage &message, uint32_t id, uint16_t cvAdr, uint8_t value, bool directProc, bool writeByte)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::locoFunc);
	message.length = 0x08;
	message.data[0] = 0xFF & (id >> 24);
	message.data[1] = 0xFF & (id >> 16);
	message.data[2] = 0xFF & (id >> 8);
	message.data[3] = 0xFF & id;
	message.data[4] = highByte(cvAdr);
	message.data[5] = lowByte(cvAdr);
	message.data[6] = value;
	message.data[7] = directProc ? 0 : (writeByte ? 1 : 2);
}

void TrainBoxMaerklin::messageAccSwitch(TrackMessage &message, uint32_t uid, uint8_t position, uint8_t current)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::accSwitch);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = position;
	message.data[5] = current;
}

void TrainBoxMaerklin::messageAccSwitch(TrackMessage &message, uint32_t uid, uint8_t position, uint8_t current, uint16_t switchTimeIN10ms)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::accSwitch);
	message.length = 0x08;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = position;
	message.data[5] = current;
	message.data[6] = highByte(switchTimeIN10ms);
	message.data[7] = lowByte(switchTimeIN10ms);
}

void TrainBoxMaerklin::messagePing(TrackMessage &message)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::ping);
	message.length = 0x00;
}

void TrainBoxMaerklin::messagePing(TrackMessage &message, uint32_t uid, uint16_t swVersion, uint16_t hwIdent)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(TrainBoxMaerklin::Cmd::ping);
	message.length = 0x08;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = highByte(swVersion);
	message.data[5] = lowByte(swVersion);
	message.data[6] = highByte(hwIdent);
	message.data[7] = lowByte(hwIdent);
}

// ===================================================================
// === SystemCmd =====================================================
// ===================================================================

bool TrainBoxMaerklin::sendSystemStop(uint32_t uid)
{
	TrackMessage message;
	messageSystemStop(message, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSystemGo(uint32_t uid)
{
	TrackMessage message;
	messageSystemGo(message, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSystemHalt(uint32_t uid)
{
	TrackMessage message;
	messageSystemHalt(message, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendLocoStop(uint32_t uid)
{
	TrackMessage message;
	messageLocoStop(message, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendLocoRemoveCycle(uint32_t uid)
{
	TrackMessage message;
	messageLocoRemoveCycle(message, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendLocoDataProtocol(uint32_t uid, ProtocolLoco protocol)
{
	TrackMessage message;
	messageLocoDataProtocol(message, uid, protocol);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendAccTime(uint16_t accTimeIN10ms, uint32_t uid)
{
	TrackMessage message;
	messageAccTime(message, accTimeIN10ms, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendFastReadMfx(uint16_t mfxSid, uint32_t uid)
{
	TrackMessage message;
	messageFastReadMfx(message, mfxSid, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSetTrackProtocol(uint8_t protocols, uint32_t uid)
{
	TrackMessage message;
	messageSetTrackProtocol(message, protocols, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSetMfxCounter(uint16_t counter, uint32_t uid)
{
	TrackMessage message;
	messageSetMfxCounter(message, counter, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSystemStatus(uint8_t channelNumber, uint32_t uid)
{
	TrackMessage message;
	messageSystemStatus(message, channelNumber, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSystemStatus(uint8_t channelNumber, uint16_t configuration, uint32_t uid)
{
	TrackMessage message;
	messageSystemStatus(message, channelNumber, configuration, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSetSystemIdent(uint16_t systemIdent, uint32_t uid)
{
	TrackMessage message;
	messageSetSystemIdent(message, systemIdent, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendSystemReset(uint8_t resetTarget, uint32_t uid)
{
	TrackMessage message;
	messageSystemReset(message, resetTarget, uid);
	return sendMessage(message);
}

// ===================================================================
// === LocoCmd =======================================================
// ===================================================================

bool TrainBoxMaerklin::requestLocoSpeed(uint32_t uid)
{
	TrackMessage message;
	messageLocoSpeed(message, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::setLocoSpeed(uint32_t uid, uint16_t speed)
{
	TrackMessage message;
	messageLocoSpeed(message, uid, speed);
	return sendMessage(message);
}

bool TrainBoxMaerklin::requestLocoDir(uint32_t uid)
{
	TrackMessage message;
	messageLocoDir(message, uid);
	return sendMessage(message);
}

bool TrainBoxMaerklin::setLocoDir(uint32_t uid, uint8_t dir)
{
	TrackMessage message;
	messageLocoDir(message, uid, dir);
	return sendMessage(message);
}

bool TrainBoxMaerklin::requestLocoFunc(uint32_t uid, uint8_t function)
{
	TrackMessage message;
	messageLocoFunc(message, uid, function);
	return sendMessage(message);
}

bool TrainBoxMaerklin::setLocoFunc(uint32_t uid, uint8_t function, uint8_t value)
{
	TrackMessage message;
	messageLocoFunc(message, uid, function, value);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendWriteConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool directProc, bool writeByte)
{
	TrackMessage message;
	messageWriteConfig(message, id, cvAdr, value, directProc, writeByte);
	return sendMessage(message);
}

bool TrainBoxMaerklin::setAccSwitch(uint32_t uid, uint8_t position, uint8_t current)
{
	TrackMessage message;
	messageAccSwitch(message, uid, position, current);
	return sendMessage(message);
}

bool TrainBoxMaerklin::setAccSwitch(uint32_t uid, uint8_t position, uint8_t current, uint16_t switchTimeIN10ms)
{
	TrackMessage message;
	messageAccSwitch(message, uid, position, current, switchTimeIN10ms);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendPing()
{
	TrackMessage message;
	messagePing(message);
	return sendMessage(message);
}

bool TrainBoxMaerklin::sendPing(uint32_t uid, uint16_t swVersion, uint16_t hwIdent)
{
	TrackMessage message;
	messagePing(message, uid, swVersion, hwIdent);
	return sendMessage(message);
}

// ===================================================================
// === PrintOut Functions=============================================
// ===================================================================

size_t printHex(Print &p, unsigned long hex, int digits)
{
	size_t size = 0;

	String s = String(hex, HEX);

	for (int i = s.length(); i < digits; i++)
	{
		size += p.print("0");
	}

	size += p.print(s);

	return size;
}

int parseHex(String &s, int start, int end, bool *ok)
{
	int value = 0;

	for (int i = start; i < end; i++)
	{
		char c = s.charAt(i);

		if (c >= '0' && c <= '9')
		{
			value = 16 * value + c - '0';
		}
		else if (c >= 'a' && c <= 'f')
		{
			value = 16 * value + 10 + c - 'a';
		}
		else if (c >= 'A' && c <= 'F')
		{
			value = 16 * value + 10 + c - 'A';
		}
		else
		{
			*ok = false;
			return -1;
		}
	}

	return value;
}

// ===================================================================
// === TrackMessage ==================================================
// ===================================================================

void TrackMessage::clear()
{
	command = 0;
	hash = 0;
	response = false;
	length = 0;
	for (int i = 0; i < 8; i++)
	{
		data[i] = 0;
	}
}

size_t TrackMessage::printTo(Print &p) const
{
	size_t size = 0;

	size += printHex(p, hash, 4);
	size += p.print(response ? " R " : "   ");
	size += printHex(p, command, 2);
	size += p.print(" ");
	size += printHex(p, length, 1);

	for (int i = 0; i < length; i++)
	{
		size += p.print(" ");
		size += printHex(p, data[i], 2);
	}

	return size;
}

bool TrackMessage::parseFrom(String &s)
{
	bool result = true;

	clear();

	if (s.length() < 11)
	{
		return false;
	}

	hash = parseHex(s, 0, 4, &result);
	response = s.charAt(5) != ' ';
	command = parseHex(s, 7, 9, &result);
	length = parseHex(s, 10, 11, &result);

	if (length > 8)
	{
		return false;
	}

	if (s.length() < 11 + 3 * length)
	{
		return false;
	}

	for (int i = 0; i < length; i++)
	{
		data[i] = parseHex(s, 12 + 3 * i, 12 + 3 * i + 2, &result);
	}

	return result;
}
