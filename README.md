# z21MaerklinCan

This project is based on the work of:
- Philipp Gathow (https://pgahtow.de/w/Z21_mobile)
- Joerg Pleumann (https://github.com/MBuratto/railuino)
- Gerhard Bertelsmann (https://github.com/GBert/railroad/blob/master/can2udp/src/can2lan.c)
- Hardi-St(https://github.com/Hardi-St/MirZ21)

Provides z21 and RemoteCS2/3 app(can2lan) server for Maerklin Trainbox 60113. Needs an Esp32 and a Tja1050.

Programming of locos through z21 app is deactivated by default and can be activated in webserver(will automatic be deactivated by next reboot).

Known problems:
- z21 loco programming is not tested
- Reading of configuration from additional MS2 with software version higher then 3.0 reads only locos
- RemoteCS2/3 App executes emergency stop in case of loco change when MS is connected.

ToDo:
- Configuration Writing for z21
- Testing of programming
- Reading of configuration from additional MS2 with software version less than 3.0
- add pictures for functions
