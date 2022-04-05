/*********************************************************************
 * Z21 ESP
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

#include <Arduino.h>
// For Debugging active flag and read out via serial:
#define DEBUG

#include <EEPROM.h>

#include "WebService.h"
#include "trainBoxMaerklin/CanInterface.h"
#include "trainBoxMaerklin/MaerklinLocoManagment.h"
#include "z60.h"
#include "Can2Lan.h"

#include <SPIFFS.h>

#define defaultPassword "12345678" // Default Z60 network password

std::shared_ptr<CanInterface> canInterface = std::make_shared<CanInterface>();

const uint16_t hash{0};
const uint32_t serialNumber{0xFFFFFFF0};
const uint16_t swVersion{0x0140};
const int16_t z21Port{21105};
z60 centralStation(hash, serialNumber, z21Interface::HwType::Z21_XL, swVersion, z21Port, true);

Can2Lan *can2Lan;

MaerklinLocoManagment locoManagment(0x0, centralStation, centralStation.getStationList(), 15000, 3);

File lokomotiveCs2;

/**********************************************************************************/
void setup()
{
  Serial.begin(230000);

  // Start the filesystem
  SPIFFS.begin(false);

  AutoConnectConfig configAutoConnect;

  configAutoConnect.ota = AC_OTA_BUILTIN;
  configAutoConnect.apid = "z60AP-" + String((uint32_t)(ESP.getEfuseMac() >> 32), HEX);
  configAutoConnect.psk = defaultPassword;
  configAutoConnect.apip = IPAddress(192, 168, 4, 1); // Sets SoftAP IP address
  configAutoConnect.netmask = IPAddress(255, 255, 255, 0);
  configAutoConnect.title = "z60";
  configAutoConnect.beginTimeout = 15000;
  configAutoConnect.autoReset = false;

  configAutoConnect.homeUri = "/";

  // reconnect with last ssid in handleClient
  configAutoConnect.autoReconnect = true;
  configAutoConnect.reconnectInterval = 15;

  configAutoConnect.portalTimeout = 1;

  configAutoConnect.immediateStart = true;
  configAutoConnect.autoRise = true;
  configAutoConnect.retainPortal = true;

  WebService *webService = WebService::getInstance();

  if (nullptr != webService)
  {
    auto programmingFkt = [](bool result)
    { centralStation.setProgramming(result); };

    auto readingFkt = []()
    {
      auto lambdaWriteFile = [](std::string *data)
      {
        if (nullptr != data)
        { /*Serial.println(data->c_str());*/
          lokomotiveCs2.print(data->c_str());
        }
      };

      auto lambdaWriteFileResult = [](bool success)
      {
        Serial.println(success ? "Getting locos success" : "Getting locos failed");
        lokomotiveCs2.close();
        WebService::getInstance()->setLokomotiveAvailable(success);
      };

      WebService::getInstance()->setLokomotiveAvailable(false);
      lokomotiveCs2 = SPIFFS.open("/config/lokomotive.cs2", FILE_WRITE);
      if (!lokomotiveCs2)
      {
        Serial.println("ERROR failed to open lokomotive.cs2 for writing");
      }
      else
      {
        locoManagment.getLokomotiveConfig(lambdaWriteFile, lambdaWriteFileResult);
      }
    };

    webService->begin(configAutoConnect, programmingFkt, readingFkt);
  }

  if (nullptr != canInterface.get())
  {
    canInterface->begin();
  }

  if (!centralStation.setCanObserver(canInterface))
  {
    Serial.println("ERROR: No can interface defined");
  }

  centralStation.setLocoManagment(&locoManagment);

  centralStation.begin();

  can2Lan = Can2Lan::getCan2Lan();
  if (nullptr != can2Lan)
  {
    can2Lan->begin(canInterface, true, false);
  }

  Serial.println("OK"); // start - reset serial receive Buffer
}

/**********************************************************************************/
void loop()
{
  WebService *webService = WebService::getInstance();
  if (nullptr != webService)
  {
    webService->cyclic();
  }
  canInterface->cyclic();
  centralStation.cyclic();
  locoManagment.cyclic();
  delayMicroseconds(1);
}