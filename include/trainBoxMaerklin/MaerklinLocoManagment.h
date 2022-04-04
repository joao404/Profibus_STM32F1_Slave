#pragma once

#include <Arduino.h>
#include <string>
#include <memory>
#include "trainBoxMaerklin/MaerklinConfigDataStream.h"
#include "miniz.h"

// ToDo:
// handle different files that are received. Currently only locoInfo is used

class MaerklinLocoManagment : public MaerklinConfigDataStream
{
public:
    enum class LocoManagmentState : uint8_t
    {
        Idle,
        WaitingForLocoList,
        WaitingForLocoNamen,
        WaitingForLocoInfo
    };

public:
    // can interface is needed to request data over interface
    MaerklinLocoManagment(uint32_t uid, MaerklinCanInterface &interface, std::vector<MaerklinStationConfig> &stationList,
                          unsigned long messageTimeout, uint8_t maxCmdRepeat, bool debug = false);
    virtual ~MaerklinLocoManagment();

    void cyclic();

    uint32_t getUid();

    void getLokomotiveConfig(void (*writeFileCallback)(std::string *data), void (*resultCallback)(bool success));

protected:
    // function is called by class ConfigDataStream in case that values where successful received with or without intention
    // or a planed transmission failed

    void m_reportResultFunc(std::string *data, uint16_t hash, bool success) override
    {
        handleConfigDataStreamFeedback(data, hash, success);
    }

    bool startConfigDataRequest(DataType type, std::string *info, std::string *buffer);

    bool startConfigDataRequest(DataType type, std::string *info, std::string *buffer, uint8_t cmdRepeat);

    bool Ms2LocoToCs2Loco(std::string& locoName, std::string* ms2Data, std::string* cs2Data);

    void handleConfigDataStreamFeedback(std::string *data, uint16_t hash, bool success);

    // void newLocoList(std::string& locoList);

private:
    // ich muss den aktuellen Buffer jeweils umschalten => hierzu wird Minimum eine Statemaschine benötigt, welche kontrolliert,
    // was als nächstes gedownloaded wird
    // sobald die locolist gedownloaded ist, werden alle lokomotivnamen extahiert und gespeichert.
    // Anschließend wird jeweils mit push_back ein speicher angelegt, welcher dann request ConfigData übergeben wird

    std::vector<std::string> m_locoList;

    uint32_t m_uid{0};

    LocoManagmentState m_state{LocoManagmentState::WaitingForLocoList};

    unsigned long m_lastCmdTimeINms{0};

    unsigned long m_cmdTimeoutINms{0};

    uint8_t m_maxCmdRepeat{1};

    uint8_t m_cmdRepeat{0};

    DataType m_currentType{DataType::Lokliste};

    std::string m_currentInfo;

    std::string m_buffer;

    bool m_isZLib{false};

    uint8_t m_currentLocoNum{0};

    uint8_t m_transmissionStarted{false};

    void (*m_resultCallback)(bool success){nullptr};

    void (*m_writeFileCallback)(std::string *data){nullptr};

    bool m_debug;
};