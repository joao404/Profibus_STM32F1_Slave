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

#include "z21.h"

z21::z21(CanInterface &canInterface, HwType hwType, uint16_t swVersion, int16_t port, uint16_t hash, bool debug)
    : TrainBoxMaerklinEsp32(canInterface, hash, debug),
      z21InterfaceEsp32(hwType, swVersion, port, debug),
      m_trainBoxUid(0),
      m_hwIdent(0),
      directProgramming(false)
{
}

z21::~z21()
{
}

void z21::begin()
{

  if (!m_preferences.begin(m_namespaceZ21, true))
  {
    Serial.println(F("Access preferences failed"));
  }
  else
  {
    ConfigLoco buffer[256];
    size_t sizeLocoMode = m_preferences.getBytesLength(m_keyLocoMode);
    size_t readSize = 0;
    Serial.print(F("sizeLocoMode"));
    Serial.println(sizeLocoMode);
    if (0 != sizeLocoMode)
    {
      readSize = m_preferences.getBytes(m_keyLocoMode, buffer, sizeLocoMode);
      if (readSize != sizeLocoMode)
      {
        Serial.println(F(" Failed to read locoMode"));
      }
    }
    Serial.print(F("readSize"));
    Serial.println(readSize);
    readSize /= sizeof(ConfigLoco);
    for (size_t i = 0; i < readSize; i++)
    {
      m_locos.push_back(DataLoco{buffer[i].adr, buffer[i].mode, false, 0, {buffer[i].steps, 0, 0, 0, 0, 0}});
    }

    // if (preferences.getBytes(keyTurnOutMode, turnOutMode, sizeof(turnOutMode)) != sizeof(turnOutMode))
    // {
    //   Serial.println(F(" Failed to read turnOutMode"));
    // }
    m_preferences.end();
  }

  TrainBoxMaerklinEsp32::begin();
  z21InterfaceEsp32::begin();

  delay(1000);
  // check for train box
  TrackMessage ping, pong;
  messagePing(ping); //, z21Uid, word(z21InterfaceFWVersionMSB, z21InterfaceFWVersionLSB), 0xFFFF); // CS 2 with SW 1.4
  for (uint8_t i = 0; i < 5; i++)
  {
    if (exchangeMessage(ping, pong, 1000))
    {
      if (8 == pong.length)
      {
        uint16_t hwIdent = (pong.data[6] << 8) + pong.data[7];
        if ((0x0010 == hwIdent) || (0x0011 == hwIdent)) // 60112 and 60113
        {
          m_trainBoxUid = (pong.data[0] << 24) + (pong.data[1] << 16) + (pong.data[2] << 8) + pong.data[3];
          m_hwIdent = hwIdent;
          uint16_t swVersion = (pong.data[4] << 8) + pong.data[5];
          Serial.print("Uid:");
          Serial.print(m_trainBoxUid, HEX);
          Serial.print(" HW:");
          Serial.print(hwIdent, HEX);
          Serial.print(" SW:");
          Serial.println(swVersion, HEX);
          break;
        }
      }
    }
  }

  z21InterfaceEsp32::setPower(EnergyState::csTrackVoltageOff);
  TrainBoxMaerklinEsp32::sendSystemStop();

  sendSystemStatus(static_cast<uint8_t>(valueChannel::current), m_trainBoxUid); // current
  // sendSystemStatus(static_cast<uint8_t>(valueChannel::voltage), m_trainBoxUid); // voltage
  // sendSystemStatus(static_cast<uint8_t>(valueChannel::temp), m_trainBoxUid);    // temp
}

void z21::cyclic()
{
  z21InterfaceEsp32::cyclic();
}

void z21::saveLocoConfig()
{
  ConfigLoco buffer[256];

  size_t index = 0;
  for (DataLoco n : m_locos)
  {
    buffer[index].adr = n.adr;
    buffer[index].mode = n.mode;
    buffer[index].steps = n.data[0] & 0x07;
    index++;
  }

  if (!m_preferences.begin(m_namespaceZ21, false))
  {
    Serial.println(F("Access preferences failed"));
  }
  else
  {
    if (m_preferences.putBytes(m_keyLocoMode, buffer, m_locos.size() * sizeof(ConfigLoco)) != m_locos.size() * sizeof(ConfigLoco))
    {
      Serial.println(F(" Failed to write locoMode"));
    }
    m_preferences.end();
  }
}

bool z21::calcSpeedZ21toTrainbox(uint8_t data, uint8_t speedConfig, uint8_t &speed)
{
  bool emergencyStop = false;
  if (0 == data)
  {
    speed = 0;
  }
  else if (1 == data)
  {
    emergencyStop = true;
  }
  else
  {
    if (static_cast<uint8_t>(StepConfig::Step28) == speedConfig)
    {
      speed = (((data & 0x0F) << 1) | ((data & 0x10) >> 4)) - 3;
    }
    else
    {
      speed = data - 1;
    }
  }
  return emergencyStop;
}

void z21::calcSpeedTrainboxToZ21(uint8_t speed, uint8_t speedConfig, uint8_t &data)
{
  if (0 == speed)
  {
    data = 0;
  }
  else
  {
    if (static_cast<uint8_t>(StepConfig::Step28) == speedConfig)
    {
      uint8_t buf = speed + 3;
      data = ((buf & 0x1E) >> 1) | ((buf & 0x01) << 4);
    }
    else
    {
      data = speed + 1;
    }
  }
}

bool z21::getConfig1(std::array<uint8_t, 10> &config)
{
  bool success{false};
  memset(&config[0], 0, config.size());
  success = true;
  // if (!m_preferences.begin(m_namespaceZ21, false))
  // {
  //   Serial.println(F("Access preferences failed"));
  // }
  // else
  // {
  //   if (m_preferences.getBytes(m_keyConfig2, &config[0], config.size()) != config.size())
  //   {
  //     Serial.println(F(" Failed to read config1"));
  //   }
  //   else
  //   {
  //     success = true;
  //   }
  //   m_preferences.end();
  // }
  return success;
}

void z21::setConfig1(std::array<uint8_t, 10> &config)
{
  // if (!m_preferences.begin(m_namespaceZ21, false))
  // {
  //   Serial.println(F("Access preferences failed"));
  // }
  // else
  // {
  //   if (m_preferences.putBytes(m_keyConfig1, &config[0], config.size()) != config.size())
  //   {
  //     Serial.println(F(" Failed to write config1"));
  //   }
  //   m_preferences.end();
  // }
}

bool z21::getConfig2(std::array<uint8_t, 16> &config)
{
  bool success{false};
  memset(&config[0], 0, config.size());
  success = true;
  // if (!m_preferences.begin(m_namespaceZ21, false))
  // {
  //   Serial.println(F("Access preferences failed"));
  // }
  // else
  // {
  //   if (m_preferences.getBytes(m_keyConfig2, &config[0], config.size()) != config.size())
  //   {
  //     Serial.println(F(" Failed to read config2"));
  //   }
  //   else
  //   {
  //     success = true;
  //   }
  //   m_preferences.end();
  // }
  return success;
}

void z21::setConfig2(std::array<uint8_t, 16> &config)
{
  // if (!m_preferences.begin(m_namespaceZ21, false))
  // {
  //   Serial.println(F("Access preferences failed"));
  // }
  // else
  // {
  //   if (m_preferences.putBytes(m_keyConfig2, &config[0], config.size()) != config.size())
  //   {
  //     Serial.println(F(" Failed to write config2"));
  //   }
  //   m_preferences.end();
  // }
}

uint16_t z21::getSerialNumber()
{
  return m_hwIdent;
}

// onCallback
bool z21::onSystemStop(uint32_t id)
{
  Serial.println("onSystemStop");
  uint8_t data[16];
  data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_TRACK_POWER);
  data[1] = 0x00; // Power OFF
  EthSend(0, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, (static_cast<uint16_t>(BcFlagShort::Z21bcAll) | static_cast<uint16_t>(BcFlagShort::Z21bcNetAll)));
  return true;
}

bool z21::onSystemGo(uint32_t id)
{
  Serial.println("onSystemGo");
  uint8_t data[16]; // z21Interface send storage
  data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_TRACK_POWER);
  data[1] = 0x01;
  EthSend(0x00, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, (static_cast<uint16_t>(BcFlagShort::Z21bcAll) | static_cast<uint16_t>(BcFlagShort::Z21bcNetAll)));
  return true;
}

bool z21::onSystemHalt(uint32_t id)
{
  Serial.println("onSystemHalt");
  uint8_t data[16]; // z21Interface send storage
  data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_STOPPED);
  data[1] = 0x00;
  EthSend(0x00, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, (static_cast<uint16_t>(BcFlagShort::Z21bcAll) | static_cast<uint16_t>(BcFlagShort::Z21bcNetAll)));
  return true;
}

bool z21::onLocoStop(uint32_t id)
{
  Serial.print("onLocoStop:");
  Serial.println(id, HEX);
  // uint8_t data[16]; // z21Interface send storage
  //  data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_TRACK_POWER);
  //  data[1] = 0x00;
  //  EthSend(0x00, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, 0);
  uint16_t adr = 0;
  if (id >= static_cast<uint32_t>(AddrOffset::DCC))
  {
    adr = static_cast<uint16_t>(id - static_cast<uint32_t>(AddrOffset::DCC));
  }
  else
  {
    adr = static_cast<uint16_t>(id);
  }

  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  // for (dataLoco finding : locos)
  {
    if (finding->adr == adr)
    {
      uint8_t emergencyStop = 0x01;
      finding->data[1] = emergencyStop + (finding->data[1] & 0x80);
      notifyLocoState(0, adr, finding->data);
      break;
    }
  }
  return true;
}

bool z21::onLocoRemoveCycle(uint32_t id)
{
  return false;
}

bool z21::onLocoDataProtocol(uint32_t id, ProtocolLoco protocol)
{
  return false;
}

bool z21::onAccTime(uint32_t id, uint16_t accTimeIN10ms)
{
  return false;
}

bool z21::onFastReadMfx(uint32_t id, uint16_t mfxSid)
{
  return false;
}

bool z21::onTrackProtocol(uint32_t id, uint8_t param)
{
  return true;
}

bool z21::onMfxCounter(uint32_t id, uint16_t counter)
{
  return false;
}

bool z21::onSystemOverLoad(uint32_t id, uint8_t channel)
{
  Serial.print("onSystemOverLoad:");
  Serial.println(channel);
  uint8_t data[16];
  data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_BC_TRACK_POWER);
  data[1] = 0x00; // Power OFF
  EthSend(0, 0x07, z21Interface::Header::LAN_X_HEADER, data, true, 0);
  return true;
}

bool z21::onSystemStatus(uint32_t id, uint8_t channel, bool valid)
{
  return false;
}

bool z21::onSystemStatus(uint32_t id, uint8_t channel, uint16_t value)
{
  Serial.print("Channel ");
  Serial.print(channel);
  Serial.print(" send ");
  Serial.println(value);

  if (static_cast<uint8_t>(valueChannel::current) == channel)
  {
    m_currentINmA = (value - 0x0F) * 10;
    sendSystemStatus(static_cast<uint8_t>(valueChannel::voltage), m_trainBoxUid); // voltage
  }
  else if (static_cast<uint8_t>(valueChannel::voltage) == channel)
  {
    m_voltageINmV = value * 10;
    sendSystemStatus(static_cast<uint8_t>(valueChannel::temp), m_trainBoxUid);    // temp
  }
  else if (static_cast<uint8_t>(valueChannel::temp) == channel)
  {
    m_tempIN10_2deg = value;
    sendSystemInfo(0, m_currentINmA, m_voltageINmV, m_tempIN10_2deg); // report System State to z21Interface clients
  }  
  return true;
}

bool z21::onSystemIdent(uint32_t id, uint16_t feedbackId)
{
  return false;
}

bool z21::onSystemReset(uint32_t id, uint8_t target)
{
  Serial.println("onSystemReset");
  return true;
}
///////////////////////////
bool z21::onLocoSpeed(uint32_t id)
{
  // reading was not possible
  Serial.println(F("onLocoSpeed"));
  return true;
}

bool z21::onLocoSpeed(uint32_t id, uint16_t speed)
{
  uint16_t adr = 0;
  if (id >= static_cast<uint32_t>(AddrOffset::DCC))
  {
    adr = static_cast<uint16_t>(id - static_cast<uint32_t>(AddrOffset::DCC));
  }
  else
  {
    adr = static_cast<uint16_t>(id);
  }

  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  // for (dataLoco finding : locos)
  {
    if (finding->adr == adr)
    {
      uint8_t divider = 71; // 14 steps
      uint8_t stepConfig = (finding->data[0] & 0x03);
      if (stepConfig == static_cast<uint8_t>(StepConfig::Step128))
      {
        divider = 8;
      }
      else if (stepConfig == static_cast<uint8_t>(StepConfig::Step28))
      {
        divider = 35;
      }

      Serial.print("Id:");
      Serial.print(adr);
      Serial.print(" onLocoSpeed:");
      Serial.println(speed);

      uint8_t locoSpeed = static_cast<uint8_t>(speed / divider);
      uint8_t locoSpeedDcc = 0;
      calcSpeedTrainboxToZ21(locoSpeed, stepConfig, locoSpeedDcc);
      finding->data[1] = locoSpeedDcc | (finding->data[1] & 0x80);
      notifyLocoState(0, adr, finding->data);
      break;
    }
  }
  return true;
}

// 0 = Fahrtrichtung bleibt
// 1 = Fahrtrichtung vorwärts
// 2 = Fahrtrichtung rückwärts
// 3 = Fahrtrichtung umschalten
bool z21::onLocoDir(uint32_t id, uint8_t dir)
{

  // Serial.print("Id:");
  // Serial.print(id);
  // Serial.print(" onLocoDir:");
  // Serial.println(dir);

  uint16_t adr = 0;
  if (id >= static_cast<uint16_t>(AddrOffset::DCC))
  {
    adr = static_cast<uint16_t>(id - static_cast<uint32_t>(AddrOffset::DCC));
  }
  else
  {
    adr = static_cast<uint16_t>(id);
  }

  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  {
    if (finding->adr == adr)
    {
      finding->data[1] = (finding->data[1] & 0x7F) + (2 == dir ? 0x00 : 0x80);
      // notifyLocoState(0, static_cast<uint16_t>(id), finding->second);
      break;
    }
  }
  return true;
}

bool z21::onLocoFunc(uint32_t id, uint8_t function, uint8_t value)
{
  Serial.print("Id:");
  Serial.print(id);
  Serial.print(" onLocoFunc:");
  Serial.print(function);
  Serial.print(" value:");
  Serial.println(value);

  uint16_t adr = 0;
  if (id >= static_cast<uint16_t>(AddrOffset::DCC))
  {
    adr = static_cast<uint16_t>(id - static_cast<uint32_t>(AddrOffset::DCC));
  }
  else
  {
    adr = static_cast<uint16_t>(id);
  }

  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  {
    if (finding->adr == adr)
    {
      if (0 == function)
      {
        bitWrite(finding->data[2], 4, 0 == value ? 0 : 1);
      }
      else if (function < 5)
      {
        bitWrite(finding->data[2], function - 1, 0 == value ? 0 : 1);
      }
      else if (function < 13)
      {
        bitWrite(finding->data[3], function - 5, 0 == value ? 0 : 1);
      }
      else if (function < 21)
      {
        bitWrite(finding->data[4], function - 13, 0 == value ? 0 : 1);
      }
      else if (function < 29)
      {
        bitWrite(finding->data[5], function - 21, 0 == value ? 0 : 1);
      }
      else
      {
        Serial.println("### ERROR: Function number to big");
      }
      notifyLocoState(0, adr, finding->data);
      break;
    }
  }
  return true;
}

bool z21::onWriteConfig(uint32_t id, uint16_t cvAdr, uint8_t value, bool writeSuccessful, bool verified)
{
  Serial.print("Id:");
  Serial.print(id);
  Serial.print(" cvAdr:");
  Serial.print(cvAdr);
  Serial.print(" value:");
  Serial.print(value);
  Serial.print(" :");
  Serial.print(writeSuccessful);
  Serial.println(verified);
  if (directProgramming)
  {
    // sending report in case that direct programming
  }
  return true;
}

///////////////////////////
bool z21::onAccSwitch(uint32_t id, uint8_t position, uint8_t current)
{
  Serial.print("onAccSwitch:");
  Serial.print(id);
  Serial.print(" position:");
  Serial.print(position);
  Serial.print(" current:");
  Serial.println(current);

  uint8_t data[16];
  if (static_cast<uint32_t>(AddrOffset::DCCAcc) <= id)
  {
    id -= static_cast<uint32_t>(AddrOffset::DCCAcc);
    id += m_startAdressAccDCC;
  }
  else if (static_cast<uint32_t>(AddrOffset::MM2Acc) <= id)
  {
    id -= static_cast<uint32_t>(AddrOffset::MM2Acc);
  }

  auto finding = m_turnouts.find(id);
  if (finding == m_turnouts.end())
  {
    // delete all if buffer is to big
    if (m_maxNumberOfTurnout < m_turnouts.size())
    {
      Serial.println(F("clear turnoutlist"));
      m_turnouts.clear();
    }
    m_turnouts.insert({id, position ? 0x02 : 0x01});
    Serial.print(F("Turnout not found:"));
    Serial.println(id);
  }
  else
  {
    finding->second = position ? 0x02 : 0x01;
  }
  Serial.println(id);
  data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_GET_TURNOUT_INFO);
  data[1] = highByte(static_cast<uint16_t>(id));
  data[2] = lowByte(static_cast<uint16_t>(id));
  data[3] = position ? 0x02 : 0x01;
  EthSend(0x00, 0x09, z21Interface::Header::LAN_X_HEADER, data, true, static_cast<uint16_t>(BcFlagShort::Z21bcAll) | static_cast<uint16_t>(BcFlagShort::Z21bcNetAll)); // BC new 23.04. !!!(old = 0)
  return true;
}

bool z21::onPing(uint32_t id, uint16_t swVersion, uint16_t hwIdent)
{
  Serial.print("Ping received: Uid:");
  Serial.print(id);
  Serial.print(" swVersion:");
  Serial.print(swVersion);
  Serial.print(" hwIdent:");
  Serial.println(hwIdent);
  return true;
}

void z21::notifyLocoState(uint8_t client, uint16_t Adr, std::array<uint8_t, 6> &locoData)
{

  uint8_t data[9];
  data[0] = static_cast<uint8_t>(z21Interface::XHeader::LAN_X_LOCO_INFO); // 0xEF X-HEADER
  data[1] = (Adr >> 8) & 0x3F;
  data[2] = Adr & 0xFF;
  // Fahrstufeninformation: 0=14, 2=28, 4=128
  if ((locoData[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step14))
    data[3] = 1; // 14 steps
  if ((locoData[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step28))
    data[3] = 2; // 28 steps
  if ((locoData[0] & 0x03) == static_cast<uint8_t>(StepConfig::Step128))
    data[3] = 4; // 128 steps
  // data[3] = data[3] | 0x08; //BUSY!

  data[4] = (char)locoData[1]; // DSSS SSSS
  data[5] = (char)locoData[2]; // F0, F4, F3, F2, F1
  data[6] = (char)locoData[3]; // F5 - F12; Funktion F5 ist bit0 (LSB)
  data[7] = (char)locoData[4]; // F13-F20
  data[8] = (char)locoData[5]; // F21-F28

  EthSend(0, 14, z21Interface::Header::LAN_X_HEADER, data, true, (static_cast<uint16_t>(BcFlagShort::Z21bcAll) | static_cast<uint16_t>(BcFlagShort::Z21bcNetAll)));

  // //Info to all:
  // for (uint8_t i = 0; i < z21InterfaceclientMAX; i++) {
  // 	if (ActIP[i].client != client) {
  // 		if ((ActIP[i].BCFlag & (0 | Z21bcNetAll_s)) > 0) {
  // 			EthSend (ActIP[i].client, 14, z21Interface::Header::LAN_X_HEADER, data, true, Z21bcNone);  //Send Loco status und Funktions to BC Apps
  // 		}
  // 	}
  // 	else { //Info to client that ask:
  // 		if (ActIP[i].adr == Adr) {
  // 			data[3] = data[3] & B111;	//clear busy flag!
  // 		}
  // 		EthSend (client, 14, z21Interface::Header::LAN_X_HEADER, data, true, Z21bcNone);  //Send Loco status und Funktions to request App
  // 		data[3] = data[3] | 0x08; //BUSY!
  // 	}
  // }
}

// Z21

void z21::handleGetLocoMode(uint16_t adr, uint8_t &mode)
{
  mode = 0;
  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  {
    if (finding->adr == adr)
    {
      mode = finding->mode;
      return;
    }
  }
  Serial.print("handleGetLocoMode");
  Serial.println(adr);
}

void z21::handleSetLocoMode(uint16_t adr, uint8_t mode)
{
  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  {
    if (finding->adr == adr)
    {
      if (finding->mode != mode)
      {
        finding->mode = mode;
        // Write to flash
        saveLocoConfig();
        // speed is send again in next cycle
        finding->isActive = false;
      }
      return;
    }
  }
  Serial.print("handleSetLocoMode");
  Serial.println(adr);
  if (m_locos.size() >= m_maxNumberOfLoco)
  {
    m_locos.pop_back();
  }
  m_locos.push_front(DataLoco{adr, mode, false, 0, {static_cast<uint8_t>(StepConfig::Step128), 0, 0, 0, 0, 0}});
  saveLocoConfig();
}

void z21::handleGetTurnOutMode(uint16_t adr, uint8_t &mode)
{
  if (adr >= m_startAdressAccDCC)
  {
    mode = 0;
  }
  else
  {
    mode = 1;
  }
}

void z21::handleSetTurnOutMode(uint16_t adr, uint8_t mode)
{
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceRailPower(EnergyState State)
{
  Serial.print("Power: ");
  Serial.println(static_cast<uint8_t>(State), HEX);

  if (EnergyState::csNormal == State)
  {
    TrainBoxMaerklinEsp32::sendSystemGo(0); // trainBoxUid);
    // TrackMessage out, in;
    // messageSystemGo(out);
    // exchangeMessage(out, in, 1000)
  }
  else if (EnergyState::csEmergencyStop == State)
  {
    TrainBoxMaerklinEsp32::sendSystemHalt(0); // trainBoxUid);
    // TrainBoxMaerklinEsp32::sendSystemStop();
  }
  else if (EnergyState::csTrackVoltageOff == State)
  {
    TrainBoxMaerklinEsp32::sendSystemStop(0); // trainBoxUid);
  }
  z21InterfaceEsp32::setPower(State);
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceS88Data(uint8_t gIndex)
{
  // z21Interface.setS88Data (datasend);  //Send back state of S88 Feedback
  Serial.println("S88Data");
}

void z21::notifyz21InterfaceLocoState(uint16_t Adr, uint8_t data[])
{
  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  {
    if (finding->adr == Adr)
    {
      uint8_t index = 0;
      for (auto i : finding->data)
      {
        data[index++] = i;
      }
      if (finding->data[0] == static_cast<uint8_t>(StepConfig::Step128))
      {
        data[0] = 4;
      }
      return;
    }
  }
  Serial.print("notifyz21InterfaceLocoState:");
  Serial.println(Adr);
  if (m_locos.size() >= m_maxNumberOfLoco)
  {
    m_locos.pop_back();
  }
  m_locos.push_front(DataLoco{Adr, 0, false, 0, {static_cast<uint8_t>(StepConfig::Step128), 0, 0, 0, 0, 0}});
  saveLocoConfig();
  data[0] = static_cast<uint8_t>(StepConfig::Step128);
  memset(&data[1], 0, 5);
  // before state was requested her, which is not possible if mode of loco is not known
}

void z21::notifyz21InterfaceLocoFkt(uint16_t Adr, uint8_t type, uint8_t fkt)
{
  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  {
    if (finding->adr == Adr)
    {
      uint32_t id = static_cast<uint32_t>(Adr) + (finding->mode ? 0 : static_cast<uint32_t>(AddrOffset::DCC));
      setLocoFunc(id, fkt, type);
      return;
    }
  }
  if (m_locos.size() >= m_maxNumberOfLoco)
  {
    m_locos.pop_back();
  }
  Serial.print("Loco not found:");
  Serial.println(Adr);
  m_locos.push_front(DataLoco{Adr, 0, false, 0, {static_cast<uint8_t>(StepConfig::Step128), 0, 0, 0, 0, 0}});
  saveLocoConfig();
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceLocoSpeed(uint16_t Adr, uint8_t speed, uint8_t stepConfig)
{
  for (auto finding = m_locos.begin(); finding != m_locos.end(); ++finding)
  {
    if (finding->adr == Adr)
    {
      // adapt adress for trainbox
      uint32_t id = static_cast<uint32_t>(Adr) + (finding->mode ? 0 : static_cast<uint32_t>(AddrOffset::DCC));

      if (finding->data[0] != stepConfig)
      {
        finding->data[0] = stepConfig;
        // safe config to flash
        saveLocoConfig();
        finding->isActive = true;
      }
      if (!finding->isActive)
      {
        // set data protocol
        if (0 == finding->mode) // DCC
        {
          switch (stepConfig)
          {
          case static_cast<uint8_t>(StepConfig::Step14):
            sendLocoDataProtocol(id, ProtocolLoco::DCC_SHORT_14);
            break;
          case static_cast<uint8_t>(StepConfig::Step28):
            if (m_longDccAddressStart > Adr)
            {
              sendLocoDataProtocol(id, ProtocolLoco::DCC_SHORT_28);
            }
            else
            {
              sendLocoDataProtocol(id, ProtocolLoco::DCC_LONG_28);
            }
            break;
          case static_cast<uint8_t>(StepConfig::Step128):
            if (m_longDccAddressStart > Adr)
            {
              sendLocoDataProtocol(id, ProtocolLoco::DCC_SHORT_128);
            }
            else
            {
              sendLocoDataProtocol(id, ProtocolLoco::DCC_LONG_128);
            }
            break;
          default:
            sendLocoDataProtocol(id, ProtocolLoco::DCC_SHORT_28);
            break;
          }
        }
        finding->isActive = true;
      }

      uint8_t locoSpeedAdapted = 0;
      if (calcSpeedZ21toTrainbox(speed & 0x7F, stepConfig, locoSpeedAdapted))
      {
        // emergency break
        Serial.print("Emergency Break:");
        Serial.println(id);
        sendLocoStop(id);
      }
      else
      {
        unsigned long currentTimeINms = millis();
        if ((finding->lastSpeedCmdTimeINms + minimumCmdIntervalINms) < currentTimeINms)
        {
          finding->lastSpeedCmdTimeINms = currentTimeINms;
          uint8_t steps = 1;
          if (static_cast<uint8_t>(StepConfig::Step14) == stepConfig)
          {
            steps = 14;
          }
          else if (static_cast<uint8_t>(StepConfig::Step28) == stepConfig)
          {
            steps = 28;
          }
          else
          {
            steps = 128;
          }
          uint16_t locoSpeedTrainBox = static_cast<uint16_t>(static_cast<uint32_t>(locoSpeedAdapted) * 1000 / static_cast<uint32_t>(steps));
          Serial.print("SetSpeed:");
          Serial.print(locoSpeedTrainBox);
          Serial.print(" Dir:");
          Serial.println(speed & 0x80 ? 1 : 2);

          setLocoDir(id, speed & 0x80 ? 1 : 2);
          setLocoSpeed(id, locoSpeedTrainBox);
        }
      }
      return;
    }
  }
  Serial.print("Loco not found:");
  Serial.println(Adr);
  if (m_locos.size() >= m_maxNumberOfLoco)
  {
    m_locos.pop_back();
  }
  m_locos.push_front(DataLoco{Adr, 0, false, 0, {stepConfig, 0, 0, 0, 0, 0}});
  saveLocoConfig();
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceAccessory(uint16_t Adr, bool state, bool active)
{
  Serial.print("setAccSwitch:");
  Serial.print(Adr);
  Serial.print(" state:");
  Serial.print(state);
  Serial.print(" active:");
  Serial.println(active);

  auto finding = m_turnouts.find(Adr);
  if (finding == m_turnouts.end())
  {
    // delete all if buffer is to big
    if (m_maxNumberOfTurnout < m_turnouts.size())
    {
      Serial.println(F("clear turnoutlist"));
      m_turnouts.clear();
    }
    m_turnouts.insert({Adr, state ? 0x02 : 0x01});
    Serial.print(F("Turnout not found:"));
    Serial.println(Adr);
  }
  else
  {
    finding->second = state ? 0x02 : 0x01;
  }

  uint32_t adrTurnOut = static_cast<uint32_t>(Adr);

  // if (adrTurnOut < 256)
  // {
  //   adrTurnOut += bitRead(turnOutMode[(Adr + 1) / 8], (Adr + 1) % 8) ? static_cast<uint32_t>(AddrOffset::MM2Acc) : static_cast<uint32_t>(AddrOffset::DCCAcc);
  // }
  // else
  // {
  //   adrTurnOut += static_cast<uint32_t>(AddrOffset::DCCAcc);
  // }
  if (adrTurnOut >= m_startAdressAccDCC)
  {
    adrTurnOut -= m_startAdressAccDCC;
    adrTurnOut += static_cast<uint32_t>(AddrOffset::DCCAcc);
  }
  else
  {
    adrTurnOut += static_cast<uint32_t>(AddrOffset::MM2Acc);
  }
  Serial.println(adrTurnOut);
  setAccSwitch(adrTurnOut, state ? 0x01 : 0x00, active ? 0x00 : 0x01, 0); // only execute command, no deactivation
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceAccessoryInfo(uint16_t Adr, uint8_t &position)
{
  auto finding = m_turnouts.find(Adr);
  position = finding != m_turnouts.end() ? finding->second : 0;
}

//--------------------------------------------------------------------------------------------
uint8_t z21::notifyz21InterfaceLNdispatch(uint16_t Adr)
// return the Slot that was dispatched, 0xFF at error!
{
  Serial.println("LNdispatch");
  return 0xFF;
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceLNSendPacket(uint8_t *data, uint8_t length)
{
  Serial.println("LNSendPacket");
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceCVREAD(uint8_t cvAdrMSB, uint8_t cvAdrLSB)
{
  Serial.println("CVREAD");
  directProgramming = true;
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceCVWRITE(uint8_t cvAdrMSB, uint8_t cvAdrLSB, uint8_t value)
{
  Serial.println("CVWRITE");
  directProgramming = true;
  uint16_t cvAdr = (cvAdrMSB << 8) + cvAdrLSB;
  sendWriteConfig(80, cvAdr, value, true, false);
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceMMWRITE(uint8_t regAdr, uint8_t value)
{
  Serial.println("MMWRITE");
  directProgramming = true;
  sendWriteConfig(80, static_cast<uint8_t>(regAdr), value, true, false);
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceDCCWRITE(uint8_t regAdr, uint8_t value)
{
  Serial.println("MMWRITE");
  directProgramming = true;
  sendWriteConfig(0xC001, static_cast<uint8_t>(regAdr), value, true, false);
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceDCCREAD(uint8_t regAdr)
{
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceCVPOMWRITEBYTE(uint16_t Adr, uint16_t cvAdr, uint8_t value)
{
  Serial.println("CVPOMWRITEBYTE");
  directProgramming = false;
  sendWriteConfig(static_cast<uint16_t>(Adr), cvAdr, value, false, true);
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceCVPOMWRITEBIT(uint16_t Adr, uint16_t cvAdr, uint8_t value)
{
  Serial.println("CVPOMWRITEBIT");
  // directProgramming = false;
  // sendWriteConfig(static_cast<uint16_t>(Adr), cvAdr, value, false, false);
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfaceCVPOMREADBYTE(uint16_t Adr, uint16_t cvAdr)
{
  Serial.println("CVPOMREADBYTE");
  directProgramming = false;
}

//--------------------------------------------------------------------------------------------
void z21::notifyz21InterfacegetSystemInfo(uint8_t client)
{
  // uint16_t inAm = 0;
  // uint16_t temp = 1600;
  // uint16_t volt = 0x4650; // 18V

  sendSystemStatus(static_cast<uint8_t>(valueChannel::current), m_trainBoxUid); // current
  // sendSystemStatus(static_cast<uint8_t>(valueChannel::voltage), m_trainBoxUid); // voltage
  // sendSystemStatus(static_cast<uint8_t>(valueChannel::temp), m_trainBoxUid);    // temp
  // sendSystemInfo(client, 1000, 18000, 1600); // report System State to z21Interface clients
  //(12-22V): 20V=0x4e20, 21V=0x5208, 22V=0x55F0
}