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
#include "Can2Udp.h"

#define defaultPassword "12345678" // Default Z60 network password

WebServer webServer;
AutoConnect autoConnect(webServer);
AutoConnectConfig configAutoConnect;

AutoConnectAux auxZ60Config("/z60config", "Z60 Config");
ACCheckbox(progActive, "progActive", "Trackprogramming activ",false);
ACSubmit(submit, "Save", "/z60config_save");
ACText(readingLocoHeader, "Press this button to trigger reading of current loco list from Mobile Station with lowest uid (Master)");
ACSubmit(readingLoco, "Reading Loco", "/readingLocoList");


CanInterface canInterface;

z60 centralStation(canInterface, z21Interface::HwType::Z21_XL, 0xFFFFFFF0, 0x0140, 21105, 0, true);

MaerklinLocoManagment locoManagment(0x0, centralStation);

Can2Udp can2Udp(canInterface, false);

/**********************************************************************************/
void setup()
{
  Serial.begin(230000); // UDP to Serial Kommunikation

  autoConnect.onNotFound([]()
                       {
    String message = "File Not Found\n";
    message += "URI: ";
    message += webServer.uri();
    message += "\nMethod: ";
    message += (webServer.method() == HTTP_GET) ? "GET" : "POST";
    message += "\nArguments: ";
    message += webServer.args();
    message += "\n";
    Serial.print(message);
    for (uint8_t i = 0; i < webServer.args(); i++) {
     message += " " + webServer.argName(i) + ": " + webServer.arg(i) + "\n";
    }
    webServer.send(404, "text/plain", message); });

    webServer.on("/can",[]()
    {
      Serial.println("Can requested");
    webServer.send(200, "text/plain", "Can is here"); });

    webServer.on("/config/prefs.cs2",[]()
    {
      Serial.println("prefs requested");
    webServer.send(200, "text/plain", "[Preferences]\nversion\n .minor=2\npage\n .entry\n ..key=Vergabe\n ..value=auto (DHCP)\n"
    " .entry\n ..key=Mac-Adresse\n ..value=00:80:82:8A:99:46\n .entry ..key=IpAdresse ..value=192.168.4.1\n .entry\n"
    " ..key=NetzMaske\n ..value=255.255.255.0\n .entry\n ..key=IpGateway\n ..value=...\n .entry\n ..key=IpDNS\n"
    " ..value=192.168.4.1\npage\n .entry\n ..key=CANGateway\n ..value=auto\n ..id=1\n .entry\n ..key=GwZielAddr\n"
    " ..value=192.168.4.255\npage\n .entry\n ..key=Version"); });
    webServer.on("/config/lokomotive.cs2",[]()
    {
      Serial.println("lokomotive requested");
    webServer.send(200, "text/plain", "[lokomotive]\n version\n .major=0\n .minor=1\n session\n .id=1\n"
    " lokomotive\n .uid=0x48\n .name=DHG300\n .adresse=0x48\n .typ=mm2_prg\n .sid=0x1\n .mfxuid=0x0\n"
    " .icon=DHG300\n .symbol=7\n .av=6\n .bv=3\n .volume=25\n .velocity=0\n .richtung=0\n .tachomax=320\n"
    " .vmax=60\n .vmin=3\n .xprotokoll=0\n .mfxtyp=0\n .stand=0x0\n .fahrt=0x0\n .funktionen\n ..nr=0\n"
    " ..typ=1\n ..dauer=0\n ..wert=0\n ..vorwaerts=0x0\n ..rueckwaerts=0x0\n .funktionen\n ..nr=1\n"
    " ..typ=51\n ..dauer=0\n ..wert=0\n ..vorwaerts=0x0\n ..rueckwaerts=0x0\n .inTraktion=0xffffffff\n"); });

  configAutoConnect.ota = AC_OTA_BUILTIN;
  configAutoConnect.apid = "z60AP";
  configAutoConnect.psk = defaultPassword;
  configAutoConnect.apip = IPAddress(192, 168, 4, 1); // Sets SoftAP IP address
  configAutoConnect.netmask = IPAddress(255, 255, 255, 0);
  configAutoConnect.title = "z60";
  configAutoConnect.beginTimeout = 15000;
  configAutoConnect.autoReset = false;

  configAutoConnect.homeUri = "/z60config";

  // reconnect with last ssid in handleClient
  configAutoConnect.autoReconnect = true;
  configAutoConnect.reconnectInterval = 15;

  configAutoConnect.portalTimeout = 1;

  configAutoConnect.immediateStart = true;
  configAutoConnect.autoRise = true;
  configAutoConnect.retainPortal = true;

  autoConnect.config(configAutoConnect);

  auxZ60Config.add({progActive, submit, readingLocoHeader, readingLoco});

  autoConnect.join(auxZ60Config);

  autoConnect.begin();

  // WiFi.softAP("z60AP", defaultPassword);
  // IPAddress myIP = WiFi.softAPIP();
  // Serial.print("AP IP address: ");
  // Serial.println(myIP);

  canInterface.begin();

  centralStation.setLocoManagment(&locoManagment);

  centralStation.begin();

  can2Udp.begin();

  Serial.println("OK"); // start - reset serial receive Buffer
}

/**********************************************************************************/
void loop()
{
  autoConnect.handleClient();
  canInterface.cyclic();
  centralStation.cyclic();
  delayMicroseconds(1);
}
