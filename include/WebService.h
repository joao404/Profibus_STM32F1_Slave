#pragma once

#include <WebServer.h>
#include <AutoConnect.h>
#include <SPIFFS.h>
#include <string>
#include <memory>

// class is designed as a singelton
class WebService
{
public:
    static WebService* getInstance();
    virtual ~WebService();

    void cyclic();

    void begin(AutoConnectConfig& autoConnectConfig, void (*m_deleteLocoConfigFkt)(void), void (*programmingFkt)(bool), void (*readingFkt)(void));

    void setLokomotiveAvailable(bool isAvailable);
    void setTransmissionFinished(bool hasFinished);

    void setLocoList(std::vector<std::string>* locoList){m_locoList = locoList;};

private:
    static WebService* m_instance;
    WebService();
    static void handleNotFound(void);
    String getContentType(const String &filename);

    void (*m_deleteLocoConfigFkt)(void);
    void (*m_programmingFkt)(bool);
    void (*m_readingFkt)(void);

    bool m_lokomotiveAvailable{true};
    bool m_transmissionFinished{true};

    std::vector<std::string>* m_locoList{nullptr};

    WebServer m_WebServer;
    AutoConnect m_AutoConnect;
    AutoConnectAux m_auxZ60Config;
    AutoConnectCheckbox m_deleteLocoConfig;
    AutoConnectCheckbox m_progActive;
    AutoConnectCheckbox m_readingLoco;
    AutoConnectSubmit m_saveButton;  
    AutoConnectAux m_auxZ60ConfigStatus;
    AutoConnectText m_readingStatus;
    AutoConnectText m_locoNames;
    AutoConnectSubmit m_reloadButton;     
};