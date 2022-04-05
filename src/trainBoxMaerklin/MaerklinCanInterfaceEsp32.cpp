/*********************************************************************
 * TrainBox Maerklin Esp32
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

#include "trainBoxMaerklin/MaerklinCanInterfaceEsp32.h"

MaerklinCanInterfaceEsp32::MaerklinCanInterfaceEsp32(word hash, bool debug)
    : MaerklinCanInterface(hash, debug)
{
}

MaerklinCanInterfaceEsp32::~MaerklinCanInterfaceEsp32()
{
  end();
}

bool MaerklinCanInterfaceEsp32::setCanObserver(std::shared_ptr<CanInterface> canInterface)
{
  m_canInterface = canInterface;
  return nullptr != m_canInterface;
}

void MaerklinCanInterfaceEsp32::begin()
{
  m_canInterface->attach(*this);

  MaerklinCanInterface::begin();
}

void MaerklinCanInterfaceEsp32::end()
{
}

void MaerklinCanInterfaceEsp32::update(Observable &observable, void *data)
{
  if (&observable == m_canInterface.get())
  {
    if (nullptr != data)
    {
      can_message_t *frame = static_cast<can_message_t *>(data);

      TrackMessage message;
      message.clear();
      message.command = (frame->identifier >> 17) & 0xff;
      message.hash = frame->identifier & 0xffff;
      message.response = bitRead(frame->identifier, 16);
      message.length = frame->data_length_code;

      for (int i = 0; i < frame->data_length_code; i++)
      {
        message.data[i] = frame->data[i];
      }

#ifdef CAN_DEBUG
      if (m_debug)
      {
        Serial.print("==> ");
        Serial.println(message);
      }
#endif
      handleReceivedMessage(message);
    }
  }
}

bool MaerklinCanInterfaceEsp32::sendMessage(TrackMessage &message)
{
  can_message_t tx_frame;

  message.hash = m_hash;

  tx_frame.identifier = (static_cast<uint32_t>(message.prio) << 25) | (static_cast<uint32_t>(message.command) << 17) | (uint32_t)message.hash;
  tx_frame.flags = CAN_MSG_FLAG_EXTD | CAN_MSG_FLAG_SS;
  tx_frame.data_length_code = message.length;

  for (int i = 0; i < message.length; i++)
  {
    tx_frame.data[i] = message.data[i];
  }

#ifdef CAN_DEBUG
  if (m_debug)
  {
    Serial.print("<== ");
    Serial.print(tx_frame.identifier, HEX);
    Serial.print(" ");
    Serial.println(message);
  }
#endif

  bool result{false};
  if (nullptr != m_canInterface.get())
  {
    result = m_canInterface->transmit(tx_frame, 100u);
  }
  return result;
}

bool MaerklinCanInterfaceEsp32::receiveMessage(TrackMessage &message)
{
  can_message_t rx_frame;

  bool result{false};

  if (nullptr != m_canInterface.get())
  {
    (m_canInterface->receive(rx_frame, 200u) == ESP_OK);
  }

  if (result)
  {
    message.clear();
    message.prio = (rx_frame.identifier >> 25) & 0x0f;
    message.command = (rx_frame.identifier >> 17) & 0xff;
    message.hash = rx_frame.identifier & 0xffff;
    message.response = bitRead(rx_frame.identifier, 16);
    message.length = rx_frame.data_length_code;

    for (int i = 0; i < rx_frame.data_length_code; i++)
    {
      message.data[i] = rx_frame.data[i];
    }

#ifdef CAN_DEBUG
    if (m_debug)
    {
      Serial.print("==> ");
      Serial.println(message);
    }
#endif
  }

  return result;
}