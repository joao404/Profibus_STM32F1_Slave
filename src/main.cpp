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

#include <WebServer.h>
#include <AutoConnect.h>

#include "trainBoxMaerklin/CanInterface.h"
#include "trainBoxMaerklin/MaerklinLocoManagment.h"
#include "z60.h"
#include "Can2Lan.h"

#include <SPIFFS.h>

#define defaultPassword "12345678" // Default Z60 network password

void handleNotFound(void);
String getContentType(const String &filename);

WebServer webServer;
AutoConnect autoConnect(webServer);
AutoConnectConfig configAutoConnect;

AutoConnectAux auxZ60Config("/", "Z60 Config");
ACCheckbox(progActive, "progActive", "Trackprogramming activ", false);
ACCheckbox(readingLoco, "readingLoco", "Read locos from Mobile Station", false);
ACSubmit(saveButton, "Save", "/z60configured");

CanInterface canInterface;

const uint16_t hash{0};
const uint32_t serialNumber{0xFFFFFFF0};
const uint16_t swVersion{0x0140};
const int16_t z21Port{21105};
z60 centralStation(canInterface, hash, serialNumber, z21Interface::HwType::Z21_XL, swVersion, z21Port, true);

Can2Lan *can2Lan;

MaerklinLocoManagment locoManagment(0x0, centralStation, centralStation.getStationList(), 15000, 3);

File lokomotiveCs2;

bool lokomotiveCs2IsValid{true};

/**********************************************************************************/
void setup()
{
  Serial.begin(230000);

  webServer.on("/can", []()
               {
      Serial.println("Can requested");
    webServer.send(200, "", ""); });

  webServer.on("/config/prefs.cs2", []()
               {
      Serial.println("prefs requested");
    webServer.send(200, "text/plain", 
    F(
      "[Preferences]\nversion\n .major=0\n .minor=1\npage\n .entry\n ..key=Version\n ..value=\n"
      "page\n .entry\n ..key=SerNum\n ..value=84\n .entry\n ..key=GfpUid\n ..value=1129525928\n .entry\n ..key=GuiUid\n"
      " ..value=1129525929\n .entry\n ..key=HardVers\n ..value=3.1\n"
    )); });

  // webServer.on("/config/lokomotive.cs2", []()
  //              {
  //     Serial.println("lokomotive requested");
  //   webServer.send(200, "text/plain",
  //   F(
  // "[lokomotive]\n version\n .major=0\n .minor=1\n session\n .id=1\n"
  // " lokomotive\n .uid=0x48\n .name=DHG300\n .adresse=0x48\n .typ=mm2_prg\n .sid=0x1\n .mfxuid=0x0\n"
  // " .icon=DHG300\n .symbol=7\n .av=6\n .bv=3\n .volume=25\n .velocity=0\n .richtung=0\n .tachomax=320\n"
  // " .vmax=60\n .vmin=3\n .xprotokoll=0\n .mfxtyp=0\n .stand=0x0\n .fahrt=0x0\n .funktionen\n ..nr=0\n"
  // " ..typ=1\n ..dauer=0\n ..wert=0\n ..vorwaerts=0x0\n ..rueckwaerts=0x0\n .funktionen\n ..nr=1\n"
  // " ..typ=51\n ..dauer=0\n ..wert=0\n ..vorwaerts=0x0\n ..rueckwaerts=0x0\n .inTraktion=0xffffffff\n"
  // )); });

  webServer.on("/config/magnetartikel.cs2", []()
               {
      Serial.println("magnetartikel requested");
    webServer.send(200, "text/plain",
    F(
      "[magnetartikel]\n"
      "version\n"
      " .minor=1\n"
    )); });

  webServer.on("/config/gleisbild.cs2", []()
               {
      Serial.println("gleisbild requested");
    webServer.send(200, "text/plain",
    F(
      "[gleisbild]\n"
      "version\n"
      " .major=1\n"
      "groesse\n"
      "zuletztBenutzt\n"
      " .name=gleisbildDummy\n"
      "seite\n"
      " .name=gleisbildDummy\n"
    )); });

  webServer.on("/config/fahrstrassen.cs2", []()
               {
      Serial.println("fahrstrassen requested");
    webServer.send(200, "text/plain",
    F(
      "[fahrstrassen]\n"
      "version\n"
      " .minor=4\n"
    )); });

  webServer.on("/config/gleisbilder/gleisbildDummy.cs2", []()
               {
      Serial.println("gleisbildDummy requested");
    webServer.send(200, "text/plain",
    F(
      "[gleisbildseite]\n"
      "version\n"
      " .major=1\n"
    )); });

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

  autoConnect.config(configAutoConnect);

  autoConnect.onNotFound(handleNotFound);

  auxZ60Config.add({progActive, readingLoco, saveButton});

  autoConnect.join(auxZ60Config);

  webServer.on("/z60configured", []()
               {
    if (webServer.hasArg("progActive"))
    {
      Serial.println("setProgramming(true)");
      centralStation.setProgramming(true);
    }
    else
    {
      Serial.println("setProgramming(false)");
      centralStation.setProgramming(false);
    }
    if (webServer.hasArg("readingLoco"))
    {
      Serial.println("trigger loco reading");
      lokomotiveCs2IsValid = false;
      lokomotiveCs2 = SPIFFS.open("/config/lokomotive.cs2", FILE_WRITE);
      if(!lokomotiveCs2)
      {
        Serial.println("ERROR failed to open lokomotive.cs2 for writing");
      }
      else
      {
        locoManagment.getLokomotiveConfig([](std::string* data){if(nullptr != data){/*Serial.println(data->c_str());*/lokomotiveCs2.print(data->c_str());}},
      [](bool success){Serial.println(success?"Getting locos success":"Getting locos failed");lokomotiveCs2.close();lokomotiveCs2IsValid = success;});
      }
    }
    webServer.send(200, "text/plain", lokomotiveCs2IsValid?"Success":"Ongoing"); });

  // Start the filesystem
  SPIFFS.begin(false);

  autoConnect.begin();

  // WiFi.softAP("z60AP", defaultPassword);
  // IPAddress myIP = WiFi.softAPIP();
  // Serial.print("AP IP address: ");
  // Serial.println(myIP);

  canInterface.begin();

  centralStation.setLocoManagment(&locoManagment);

  centralStation.begin();

  can2Lan = Can2Lan::getCan2Lan();
  if (nullptr != can2Lan)
  {
    can2Lan->begin(&canInterface, true, false);
  }

  Serial.println("OK"); // start - reset serial receive Buffer
}

/**********************************************************************************/
void loop()
{
  autoConnect.handleClient();
  canInterface.cyclic();
  centralStation.cyclic();
  locoManagment.cyclic();
  delayMicroseconds(1);
}

void createLokomotiveCs2(void)
{
  const String &filePath = webServer.uri();
  if (SPIFFS.exists(filePath.c_str()))
  {
    File uploadedFile = SPIFFS.open(filePath, "r");
    String mime = getContentType(filePath);
    webServer.streamFile(uploadedFile, mime);
    uploadedFile.close();
  }
}

void handleNotFound(void)
{
  const String filePath = webServer.uri();
  Serial.print(filePath);
  Serial.println(" requested");
  if (SPIFFS.exists(filePath.c_str()))
  {
    if (strcmp("/config/lokomotive.cs2", filePath.c_str()) == 0)
    {
      if (!lokomotiveCs2IsValid)
      {
        webServer.send(404, "text/plain", "lokomotive.cs2 under construction");
        return;
      }
    }
    File uploadedFile = SPIFFS.open(filePath.c_str(), "r");
    String mime = getContentType(filePath);
    webServer.streamFile(uploadedFile, mime);
    uploadedFile.close();
  }
  else if (getContentType(filePath) == "image/png")
  {
    Serial.print(webServer.uri());
    Serial.println(" requested");
    if (SPIFFS.exists("/github.png"))
    {
      File uploadedFile = SPIFFS.open("/github.png", "r");
      webServer.streamFile(uploadedFile, "image/png");
      uploadedFile.close();
    }
    else
    {
      webServer.send(404, "text/plain", "png not available");
    }
  }
  else
  {
    String message = "File Not Found\n";
    message += "URI: ";
    message += webServer.uri();
    message += "\nMethod: ";
    message += (webServer.method() == HTTP_GET) ? "GET" : "POST";
    message += "\nArguments: ";
    message += webServer.args();
    message += "\n";
    for (uint8_t i = 0; i < webServer.args(); i++)
    {
      message += " " + webServer.argName(i) + ": " + webServer.arg(i) + "\n";
    }
    Serial.print(message);
    webServer.send(404, "text/plain", message);
  }
}

String getContentType(const String &filename)
{
  if (filename.endsWith(".txt"))
  {
    return "text/plain";
  }
  else if (filename.endsWith(".htm"))
  {
    return "text/html";
  }
  else if (filename.endsWith(".html"))
  {
    return "text/html";
  }
  else if (filename.endsWith(".css"))
  {
    return "text/css";
  }
  else if (filename.endsWith(".js"))
  {
    return "application/javascript";
  }
  else if (filename.endsWith(".json"))
  {
    return "application/json";
  }
  else if (filename.endsWith(".png"))
  {
    return "image/png";
  }
  else if (filename.endsWith(".gif"))
  {
    return "image/gif";
  }
  else if (filename.endsWith(".jpg"))
  {
    return "image/jpeg";
  }
  else if (filename.endsWith(".jpeg"))
  {
    return "image/jpeg";
  }
  else if (filename.endsWith(".ico"))
  {
    return "image/x-icon";
  }
  else if (filename.endsWith(".svg"))
  {
    return "image/svg+xml";
  }
  else if (filename.endsWith(".xml"))
  {
    return "text/xml";
  }
  else if (filename.endsWith(".pdf"))
  {
    return "application/x-pdf";
  }
  else if (filename.endsWith(".zip"))
  {
    return "application/x-zip";
  }
  else if (filename.endsWith(".gz"))
  {
    return "application/x-gzip";
  }
  return "text/plain";
}