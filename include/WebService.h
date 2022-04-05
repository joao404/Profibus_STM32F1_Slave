#pragma once

#include <WebServer.h>
#include <AutoConnect.h>
#include <SPIFFS.h>

// class is designed as a singelton
class WebService
{
public:
    static WebService* getInstance();
    virtual ~WebService();

    void cyclic();

    void begin(AutoConnectConfig& autoConnectConfig, void (*programmingFkt)(bool), void (*readingFkt)(void));

    void setLokomotiveAvailable(bool isAvailable);

private:
    static WebService* m_instance;
    WebService();
    static void handleNotFound(void);
    String getContentType(const String &filename);

    void (*m_programmingFkt)(bool);
    void (*m_readingFkt)(void);

    bool m_lokomotiveAvailable{true};

    WebServer m_WebServer;
    AutoConnect m_AutoConnect;
    AutoConnectAux m_auxZ60Config;
    AutoConnectCheckbox m_progActive;
    AutoConnectCheckbox m_readingLoco;
    AutoConnectSubmit m_saveButton;
};