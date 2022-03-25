#include "Arduino.h"
#include "z21/z21InterfaceEsp32.h"

z21InterfaceEsp32::z21InterfaceEsp32(HwType hwType, uint32_t swVersion, uint16_t port, boolean debug)
:z21Interface(hwType, swVersion, debug),
port(port),
countIP(0),
IPpreviousMillis(0)
{

}


void z21InterfaceEsp32::begin()
{
  if(Udp.listen(port)) {
    Udp.onPacket([this](AsyncUDPPacket packet) {  
      uint8_t packetBuffer[packet.length() + 1];
      memcpy(packetBuffer, packet.data(), packet.length());
      packetBuffer[packet.length()] = 0;
      // Serial.print("=>");
      // Serial.print(packet.remoteIP());
      // for(int i=0;i<packet.length();i++)
      // {
      //   Serial.print(" ");
      //   Serial.print(packet.data()[i]);
      // }
      // Serial.print("\n");
      handlePacket(addIP(packet.remoteIP()), packetBuffer, packet.length());
    });
  }
}

void z21InterfaceEsp32::handlePacket(uint8_t client, uint8_t *packet, size_t packetLength)
{
  if (4 <= packetLength)
  {
    uint16_t index = 0;
    uint16_t length = 0;
    for(size_t left_size = packetLength; left_size > 3; )
    {
      length = (packet[index + 1] << 8) + packet[index];
      if(left_size < length)
      {
        break;
      }
      receive(client, &(packet[index]));  //Auswertung
      left_size -= length;
    }    
  }  
}

//--------------------------------------------------------------------------------------------
void z21InterfaceEsp32::notifyz21InterfaceEthSend(uint8_t client, uint8_t *data) 
{
 uint16_t len = data[0] + (data[1] << 8);
 if (client == 0x00) {  //Broadcast
    // Serial.print("B");
    //   for(uint16_t i=0;i<len;i++)
    //   {
    //     Serial.print(" ");
    //     Serial.print(data[i]);
    //   }
    // Serial.print("\n");
    // if(0 == countIP)
    // {
    //   Udp.broadcastTo(data, len, port);//, TCPIP_ADAPTER_IF_AP);
    // }
    
    // for (byte s = 0; s < countIP; s++) {
    //   if (mem[s].time > 0) {
    //     Udp.writeTo(data, len, mem[s].IP, port);
    //   }
    // }
    // Serial.println("B");
   Udp.broadcastTo(data, data[0], port);//, TCPIP_ADAPTER_IF_AP);

  }
  else
  {
    // Serial.print("C ");
    // Serial.print(mem[client-1].IP);
    // for(uint16_t i=0;i<len;i++)
    // {
    //   Serial.print(" ");
    //   Serial.print(data[i]);
    // }
    // Serial.print("\n");
    Udp.writeTo(data, len, mem[client-1].IP, port);
  }

}

/**********************************************************************************/
byte z21InterfaceEsp32::addIP (IPAddress ip) {
  //suche ob IP schon vorhanden?
  for (byte i = 0; i < countIP; i++) {
    if (mem[i].IP == ip) {
      mem[i].time = ActTimeIP; //setzte Zeit
      return i+1;      //Rückgabe der Speicherzelle
    }
  }
  //nicht vorhanden!
  if (countIP >= z21InterfaceclientMAX) {
    for (byte i = 0; i < countIP; i++) {
      if (mem[i].time == 0) { //Abgelaufende IP, dort eintragen!
        mem[i].IP = ip;
        mem[i].time = ActTimeIP; //setzte Zeit
        return i+1;
      }
    }
    Serial.print("EE");  //Fail
    return 0;           //Fehler, keine freien Speicherzellen!
  }
  mem[countIP].IP = ip;  //eintragen
  mem[countIP].time = ActTimeIP; //setzte Zeit
  countIP++;            //Zähler erhöhen
  return countIP;       //Rückgabe
}

void z21InterfaceEsp32::cyclic()
{
    //Nutzungszeit IP's bestimmen
  unsigned long currentMillis = millis();
  if(currentMillis - IPpreviousMillis > interval) {
    IPpreviousMillis = currentMillis;   
    for (byte i = 0; i < countIP; i++) {
        if (mem[i].time > 0) 
          mem[i].time--;    //Zeit herrunterrechnen
    }
    //notifyz21InterfacegetSystemInfo(0); //SysInfo an alle BC Clients senden!
  } 
  //notifyz21InterfacegetSystemInfo(0);
}

