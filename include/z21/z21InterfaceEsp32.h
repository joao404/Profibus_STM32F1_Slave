#pragma once

#include <Arduino.h>
// #include <WiFiUDP.h>
#include <AsyncUDP.h>
#include "z21/z21Interface.h" 

#define ActTimeIP 60    //Aktivhaltung einer IP für (sec./2)
#define interval 2000   //interval in milliseconds for checking IP aktiv state


typedef struct		//Rückmeldung des Status der Programmierung
{
  IPAddress IP;
  byte time;  //aktive Zeit
} listofIP;


class z21InterfaceEsp32:public z21Interface
{
  public:
    z21InterfaceEsp32(HwType hwType, uint32_t swVersion, uint16_t port,  boolean debug);

    void begin();
    void cyclic();

  protected:
  void handlePacket(uint8_t client, uint8_t *packet, size_t packetLength);

	void notifyz21InterfaceEthSend(uint8_t client, uint8_t *data) override;

  private:
    const int port;
    //WiFiUDP Udp;
    AsyncUDP Udp;
    listofIP mem[z21InterfaceclientMAX];
    byte countIP;    //zähler für Eintragungen

    // will store last time of IP decount updated
    unsigned long IPpreviousMillis;   

    byte addIP (IPAddress ip);


};
