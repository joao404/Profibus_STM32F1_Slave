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

#include "trainBoxMaerklin/MaerklinCanInterface.h"

size_t printHex(Print &p, unsigned long hex, int digits);
int parseHex(String &s, int start, int end, bool *ok);

MaerklinCanInterface::MaerklinCanInterface(word hash, bool debug)
	: m_hash(hash),
	  m_debug(debug)
{
	if (m_debug)
	{
		Serial.println(F("Creating TrainBoxMaerklin"));
	}
}

MaerklinCanInterface::~MaerklinCanInterface()
{
	if (m_debug)
	{
		Serial.println(F("Destroying TrainBoxMaerklin"));
	}
}

void MaerklinCanInterface::begin()
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

uint16_t MaerklinCanInterface::getHash()
{
	return m_hash;
}

bool MaerklinCanInterface::isDebug()
{
	return m_debug;
}

void MaerklinCanInterface::generateHash()
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

bool MaerklinCanInterface::exchangeMessage(TrackMessage &out, TrackMessage &in, word timeout)
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

void MaerklinCanInterface::handleReceivedMessage(TrackMessage &message)
{
	// Serial.print("==> ");
	// Serial.println(message);
	bool messageHandled{false};
	// check message if it is a response or not and call callbacks
	if (message.response)
	{
		switch (static_cast<MaerklinCanInterface::Cmd>(message.command))
		{
		case MaerklinCanInterface::Cmd::systemCmd:
			switch (static_cast<MaerklinCanInterface::SubCmd>(message.data[4]))
			{
			case MaerklinCanInterface::SubCmd::locoRemoveCycle:
				if (5 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onLocoRemoveCycle(id);
				}
				break;
			case MaerklinCanInterface::SubCmd::locoDataProtocol:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onLocoDataProtocol(id, static_cast<ProtocolLoco>(message.data[5]));
				}
				break;
			case MaerklinCanInterface::SubCmd::accTime:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t accTimeIN10ms = (message.data[5] << 8) + message.data[6];
					messageHandled = onAccTime(id, accTimeIN10ms);
				}
				break;
			case MaerklinCanInterface::SubCmd::fastReadMfx:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t mfxSid = (message.data[5] << 8) + message.data[6];
					messageHandled = onFastReadMfx(id, mfxSid);
				}
				break;
			case MaerklinCanInterface::SubCmd::setTrackProtocol:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					messageHandled = onTrackProtocol(id, message.data[5]);
				}
				break;
			case MaerklinCanInterface::SubCmd::setMfxCounter:
				if (6 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t counter = (message.data[5] << 8) + message.data[6];
					messageHandled = onMfxCounter(id, counter);
				}
				break;
			case MaerklinCanInterface::SubCmd::systemStatus:
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
			case MaerklinCanInterface::SubCmd::systemIdent:
				if (7 == message.length)
				{
					uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
					uint16_t feedbackId = (message.data[5] << 8) + message.data[6];
					messageHandled = onSystemIdent(id, feedbackId);
				}
				break;
			case MaerklinCanInterface::SubCmd::systemReset:
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
		case MaerklinCanInterface::Cmd::locoSpeed:
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
		case MaerklinCanInterface::Cmd::locoDir:
			if (5 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint8_t dir = message.data[4];
				messageHandled = onLocoDir(id, dir);
			}
			break;
		case MaerklinCanInterface::Cmd::locoFunc:
			if (6 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint8_t function = message.data[4];
				uint8_t value = message.data[5];
				messageHandled = onLocoFunc(id, function, value);
			}
			break;
		case MaerklinCanInterface::Cmd::readConfig:
			if (7 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint16_t cvAdr = (message.data[4] << 8) + message.data[5];
				uint8_t value = message.data[6];
				messageHandled = onReadConfig(id, cvAdr, value, true);
			}
			else if (6 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint16_t cvAdr = (message.data[4] << 8) + message.data[5];
				uint8_t value = 0;
				messageHandled = onReadConfig(id, cvAdr, value, false);
			}
			break;
		case MaerklinCanInterface::Cmd::writeConfig:
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

		case MaerklinCanInterface::Cmd::accSwitch:
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
		case MaerklinCanInterface::Cmd::ping:
			if (8 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				uint16_t swVersion = (static_cast<uint16_t>(message.data[4]) << 8) + message.data[5];
				uint16_t hwIdent = (static_cast<uint16_t>(message.data[6]) << 8) + message.data[7];
				messageHandled = onPing(message.hash, id, swVersion, hwIdent);
			}
			break;
		case MaerklinCanInterface::Cmd::statusDataConfig:
			if (8 == message.length)
			{
				std::array<uint8_t, 8> data{message.data[0], message.data[1], message.data[2], message.data[3], message.data[4], message.data[5], message.data[6], message.data[7]};
				messageHandled = onStatusDataConfig(message.hash, data);
			}
			else if (6 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				messageHandled = onStatusDataConfig(message.hash, id, message.data[4], message.data[5]);
			}
			break;
		case MaerklinCanInterface::Cmd::requestConfigData:
			if (8 == message.length)
			{
				std::array<uint8_t, 8> data{message.data[0], message.data[1], message.data[2], message.data[3], message.data[4], message.data[5], message.data[6], message.data[7]};
				messageHandled = onConfigData(message.hash, data);
			}
			break;
		default:

			break;
		}
	}
	switch (static_cast<MaerklinCanInterface::Cmd>(message.command))
	{
	case MaerklinCanInterface::Cmd::systemCmd:
		switch (static_cast<MaerklinCanInterface::SubCmd>(message.data[4]))
		{
		case MaerklinCanInterface::SubCmd::systemStop:
			if ((5 == message.length) || (8 == message.length))
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				messageHandled = onSystemStop(id);
			}
			break;
		case MaerklinCanInterface::SubCmd::systemGo:
			if (5 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				messageHandled = onSystemGo(id);
			}
			break;
		case MaerklinCanInterface::SubCmd::systemHalt:
			if (5 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				messageHandled = onSystemHalt(id);
			}
			break;
		case MaerklinCanInterface::SubCmd::locoStop:
			if (5 == message.length)
			{
				uint32_t id = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
				messageHandled = onLocoStop(id);
			}
			break;
		case MaerklinCanInterface::SubCmd::systemOverLoad:
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
	case MaerklinCanInterface::Cmd::configDataSteam:
		if (8 == message.length)
		{
			std::array<uint8_t, 8> data{message.data[0], message.data[1], message.data[2], message.data[3], message.data[4], message.data[5], message.data[6], message.data[7]};
			messageHandled = onConfigDataStream(message.hash, data);
		}
		else if (7 == message.length)
		{
			uint32_t length = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
			uint16_t crc = (message.data[4] << 8) + message.data[5];
			messageHandled = onConfigDataStream(message.hash, length, crc, message.data[6]);
		}
		else if (6 == message.length)
		{
			uint32_t length = (message.data[0] << 24) + (message.data[1] << 16) + (message.data[2] << 8) + message.data[3];
			uint16_t crc = (message.data[4] << 8) + message.data[5];
			messageHandled = onConfigDataStream(message.hash, length, crc);
		}
		else
		{
			messageHandled = onConfigDataSteamError(message.hash);
		}
		break;
	default:
		break;
	}

	if (messageHandled)
	{
	}
}

void MaerklinCanInterface::messageSystemStop(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::systemStop);
}

void MaerklinCanInterface::messageSystemGo(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::systemGo);
}

void MaerklinCanInterface::messageSystemHalt(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::systemHalt);
}

void MaerklinCanInterface::messageLocoStop(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoStop);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::locoStop);
}

void MaerklinCanInterface::messageLocoRemoveCycle(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::locoRemoveCycle);
}

void MaerklinCanInterface::messageLocoDataProtocol(TrackMessage &message, uint32_t uid, ProtocolLoco protocol)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::locoDataProtocol);
	message.data[5] = static_cast<uint8_t>(protocol);
}

void MaerklinCanInterface::messageAccTime(TrackMessage &message, uint16_t accTimeIN10ms, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::accTime);
	message.data[5] = highByte(accTimeIN10ms);
	message.data[6] = lowByte(accTimeIN10ms);
}

void MaerklinCanInterface::messageFastReadMfx(TrackMessage &message, uint16_t mfxSid, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::fastReadMfx);
	message.data[5] = highByte(mfxSid);
	message.data[6] = lowByte(mfxSid);
}

void MaerklinCanInterface::messageSetTrackProtocol(TrackMessage &message, uint8_t protocols, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::setTrackProtocol);
	message.data[5] = protocols;
}

void MaerklinCanInterface::messageSetMfxCounter(TrackMessage &message, uint16_t counter, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::setMfxCounter);
	message.data[5] = highByte(counter);
	message.data[6] = lowByte(counter);
}

void MaerklinCanInterface::messageSystemStatus(TrackMessage &message, uint8_t channelNumber, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::systemStatus);
	message.data[5] = channelNumber;
}

void MaerklinCanInterface::messageSystemStatus(TrackMessage &message, uint8_t channelNumber, uint16_t configuration, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x08;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::systemStatus);
	message.data[5] = channelNumber;
	message.data[6] = highByte(configuration);
	message.data[7] = lowByte(configuration);
}

void MaerklinCanInterface::messageSetSystemIdent(TrackMessage &message, uint16_t systemIdent, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x07;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::systemIdent);
	message.data[5] = highByte(systemIdent);
	message.data[6] = lowByte(systemIdent);
}

void MaerklinCanInterface::messageSystemReset(TrackMessage &message, uint8_t resetTarget, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::system);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::systemCmd);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = static_cast<uint8_t>(MaerklinCanInterface::SubCmd::systemReset);
	message.data[5] = resetTarget;
}

// ===================================================================
// === LocoCmd =======================================================
// ===================================================================

void MaerklinCanInterface::messageLocoSpeed(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoSpeed);
	message.length = 0x04;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
}

void MaerklinCanInterface::messageLocoSpeed(TrackMessage &message, uint32_t uid, uint16_t speed)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoSpeed);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = highByte(speed);
	message.data[5] = lowByte(speed);
}

void MaerklinCanInterface::messageLocoDir(TrackMessage &message, uint32_t uid)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoDir);
	message.length = 0x04;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
}

void MaerklinCanInterface::messageLocoDir(TrackMessage &message, uint32_t uid, uint8_t dir)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoDir);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = dir;
}

void MaerklinCanInterface::messageLocoFunc(TrackMessage &message, uint32_t uid, uint8_t function)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoFunc);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = function;
}

void MaerklinCanInterface::messageLocoFunc(TrackMessage &message, uint32_t uid, uint8_t function, uint8_t value)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoFunc);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = function;
	message.data[5] = value;
}

void MaerklinCanInterface::messageReadConfig(TrackMessage &message, uint32_t id, uint16_t cvAdr, uint8_t number)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoFunc);
	message.length = 0x07;
	message.data[0] = 0xFF & (id >> 24);
	message.data[1] = 0xFF & (id >> 16);
	message.data[2] = 0xFF & (id >> 8);
	message.data[3] = 0xFF & id;
	message.data[4] = highByte(cvAdr);
	message.data[5] = lowByte(cvAdr);
	message.data[6] = number;
}

void MaerklinCanInterface::messageWriteConfig(TrackMessage &message, uint32_t id, uint16_t cvAdr, uint8_t value, bool directProc, bool writeByte)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::locoFunc);
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

void MaerklinCanInterface::messageAccSwitch(TrackMessage &message, uint32_t uid, uint8_t position, uint8_t current)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::accSwitch);
	message.length = 0x06;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = position;
	message.data[5] = current;
}

void MaerklinCanInterface::messageAccSwitch(TrackMessage &message, uint32_t uid, uint8_t position, uint8_t current, uint16_t switchTimeIN10ms)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::locoAccCommand);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::accSwitch);
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

void MaerklinCanInterface::messagePing(TrackMessage &message)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::ping);
	message.length = 0x00;
}

void MaerklinCanInterface::messagePing(TrackMessage &message, uint32_t uid, uint16_t swVersion, uint16_t hwIdent)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::ping);
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

void MaerklinCanInterface::messageStatusDataConfig(TrackMessage &message, uint32_t uid, uint8_t index)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::noPrio);
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::statusDataConfig);
	message.length = 0x05;
	message.data[0] = 0xFF & (uid >> 24);
	message.data[1] = 0xFF & (uid >> 16);
	message.data[2] = 0xFF & (uid >> 8);
	message.data[3] = 0xFF & uid;
	message.data[4] = index;
}

void MaerklinCanInterface::messageConfigData(TrackMessage &message, std::array<uint8_t, 8> &request)
{
	message.clear();
	message.prio = static_cast<uint8_t>(MessagePrio::maxPrio); // message needs max prio because MS does recognize it otherwise
	message.command = static_cast<uint8_t>(MaerklinCanInterface::Cmd::requestConfigData);
	message.length = 0x08;
	for (size_t i = 0; i < 8; i++)
	{
		message.data[i] = request.at(i);
	}
}

// ===================================================================
// === SystemCmd =====================================================
// ===================================================================

bool MaerklinCanInterface::sendSystemStop(uint32_t uid)
{
	TrackMessage message;
	messageSystemStop(message, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSystemGo(uint32_t uid)
{
	TrackMessage message;
	messageSystemGo(message, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSystemHalt(uint32_t uid)
{
	TrackMessage message;
	messageSystemHalt(message, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendLocoStop(uint32_t uid)
{
	TrackMessage message;
	messageLocoStop(message, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendLocoRemoveCycle(uint32_t uid)
{
	TrackMessage message;
	messageLocoRemoveCycle(message, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendLocoDataProtocol(uint32_t uid, ProtocolLoco protocol)
{
	TrackMessage message;
	messageLocoDataProtocol(message, uid, protocol);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendAccTime(uint16_t accTimeIN10ms, uint32_t uid)
{
	TrackMessage message;
	messageAccTime(message, accTimeIN10ms, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendFastReadMfx(uint16_t mfxSid, uint32_t uid)
{
	TrackMessage message;
	messageFastReadMfx(message, mfxSid, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSetTrackProtocol(uint8_t protocols, uint32_t uid)
{
	TrackMessage message;
	messageSetTrackProtocol(message, protocols, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSetMfxCounter(uint16_t counter, uint32_t uid)
{
	TrackMessage message;
	messageSetMfxCounter(message, counter, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSystemStatus(uint8_t channelNumber, uint32_t uid)
{
	TrackMessage message;
	messageSystemStatus(message, channelNumber, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSystemStatus(uint8_t channelNumber, uint16_t configuration, uint32_t uid)
{
	TrackMessage message;
	messageSystemStatus(message, channelNumber, configuration, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSetSystemIdent(uint16_t systemIdent, uint32_t uid)
{
	TrackMessage message;
	messageSetSystemIdent(message, systemIdent, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendSystemReset(uint8_t resetTarget, uint32_t uid)
{
	TrackMessage message;
	messageSystemReset(message, resetTarget, uid);
	return sendMessage(message);
}

// ===================================================================
// === LocoCmd =======================================================
// ===================================================================

bool MaerklinCanInterface::requestLocoSpeed(uint32_t uid)
{
	TrackMessage message;
	messageLocoSpeed(message, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::setLocoSpeed(uint32_t uid, uint16_t speed)
{
	TrackMessage message;
	messageLocoSpeed(message, uid, speed);
	return sendMessage(message);
}

bool MaerklinCanInterface::requestLocoDir(uint32_t uid)
{
	TrackMessage message;
	messageLocoDir(message, uid);
	return sendMessage(message);
}

bool MaerklinCanInterface::setLocoDir(uint32_t uid, uint8_t dir)
{
	TrackMessage message;
	messageLocoDir(message, uid, dir);
	return sendMessage(message);
}

bool MaerklinCanInterface::requestLocoFunc(uint32_t uid, uint8_t function)
{
	TrackMessage message;
	messageLocoFunc(message, uid, function);
	return sendMessage(message);
}

bool MaerklinCanInterface::setLocoFunc(uint32_t uid, uint8_t function, uint8_t value)
{
	TrackMessage message;
	messageLocoFunc(message, uid, function, value);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendReadConfig(uint32_t id, uint16_t cvAdr, uint8_t number)
{
	TrackMessage message;
	messageReadConfig(message, id, cvAdr, number);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendWriteConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool directProc, bool writeByte)
{
	TrackMessage message;
	messageWriteConfig(message, id, cvAdr, value, directProc, writeByte);
	return sendMessage(message);
}

bool MaerklinCanInterface::setAccSwitch(uint32_t uid, uint8_t position, uint8_t current)
{
	TrackMessage message;
	messageAccSwitch(message, uid, position, current);
	return sendMessage(message);
}

bool MaerklinCanInterface::setAccSwitch(uint32_t uid, uint8_t position, uint8_t current, uint16_t switchTimeIN10ms)
{
	TrackMessage message;
	messageAccSwitch(message, uid, position, current, switchTimeIN10ms);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendPing()
{
	TrackMessage message;
	messagePing(message);
	return sendMessage(message);
}

bool MaerklinCanInterface::sendPing(uint32_t uid, uint16_t swVersion, uint16_t hwIdent)
{
	TrackMessage message;
	messagePing(message, uid, swVersion, hwIdent);
	return sendMessage(message);
}

bool MaerklinCanInterface::requestStatusDataConfig(uint32_t uid, uint8_t index)
{
	TrackMessage message;
	messageStatusDataConfig(message, uid, index);
	return sendMessage(message);
}

bool MaerklinCanInterface::requestConfigData(std::array<uint8_t, 8> &request)
{
	TrackMessage message;
	messageConfigData(message, request);
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
