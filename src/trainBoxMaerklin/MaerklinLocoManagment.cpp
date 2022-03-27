#include "trainBoxMaerklin/MaerklinLocoManagment.h"

MaerklinLocoManagment::MaerklinLocoManagment(uint32_t uid, MaerklinCanInterface &interface, std::vector<MaerklinStationConfig> &stationList, unsigned long messageTimeout, uint8_t maxCmdRepeat)
    : MaerklinConfigDataStream(interface, stationList), m_uid(uid), m_cmdTimeoutINms(messageTimeout), m_maxCmdRepeat(maxCmdRepeat)
{
    m_lastCmdTimeINms = millis();
}

MaerklinLocoManagment::~MaerklinLocoManagment()
{
}

uint32_t MaerklinLocoManagment::getUid()
{
    return m_uid;
}

void MaerklinLocoManagment::getAllLocos(std::vector<uint8_t> &locoList, std::vector<std::unique_ptr<LocoData>> &locos, void (*callback)(bool success))
{
    m_callbackFunc = callback;
    m_locos = &locos;
    m_state = LocoManagmentState::WaitingForLocoList;
    strncpy(m_lastInfo, "", sizeof(m_lastInfo));
    m_lastBuffer = &locoList;
    m_lastType = DataType::Lokliste;
    requestConfigData(m_lastType, m_lastInfo, m_lastBuffer);
    m_transmissionStarted = true;
    m_cmdRepeat = 1;
    m_lastCmdTimeINms = millis();
}

void MaerklinLocoManagment::handleConfigDataStreamFeedback(std::vector<uint8_t> *data, uint16_t hash, bool success)
{
    if (m_transmissionStarted) // message expected
    {
        if (success)
        {
            m_transmissionStarted = false;
            // analyze buffer depending on current state
            Serial.println("Successful request");
            if (nullptr != data)
            {
                for (auto i : (*data))
                {
                    Serial.print((char)i);
                }
                Serial.print("\n");
            }
            switch(m_state)
                {
                    case LocoManagmentState::WaitingForLocoList:
                    Serial.println("Received Lokliste");
                    // extract all loconames and switch to lokinfo for getting data
                    break;
                    default:
                    break;
                }        
                // if(nullptr != m_callbackFunc)
                // {
                //     m_callbackFunc(true);
                // }    
        }
        else
        {
            // check repeat buffer and trigger resent
            if (m_cmdRepeat < m_maxCmdRepeat)
            {
                m_cmdRepeat++;
                m_lastCmdTimeINms = millis();
                requestConfigData(m_lastType, m_lastInfo, m_lastBuffer);
            }
            else
            {
                m_transmissionStarted = false;
                switch(m_state)
                {
                    case LocoManagmentState::WaitingForLocoList:
                        // Does not seem to work
                        // Switch to old version
                        m_state = LocoManagmentState::WaitingForLocoNamen;
                        strncpy(m_lastInfo, "0 2", sizeof(m_lastInfo));
                        m_lastType = DataType::Loknamen;
                        requestConfigData(m_lastType, m_lastInfo, m_lastBuffer);
                        m_lastCmdTimeINms = millis();
                        m_transmissionStarted = true;
                        m_cmdRepeat = 1;
                    break;
                    default:
                    break;
                }
            }
        }
    }
    else
    {
        // message not expected. Only [lokliste] is handled, if transmission was successful
        if (success)
        {
            Serial.println("Unexpected data received");
            if (nullptr != data)
            {
                for (auto i : (*data))
                {
                    Serial.print((char)i);
                }
                Serial.print("\n");
            }
        }
    }
}

void MaerklinLocoManagment::cyclic()
{
    if (m_transmissionStarted)
    {
        unsigned long currentTimeINms = millis();
        if ((m_lastCmdTimeINms + m_cmdTimeoutINms) < currentTimeINms)
        {
            Serial.print("Timeout:");
            Serial.println(m_lastInfo);
            // transmission timed out
            handleConfigDataStreamFeedback(nullptr, 0, false);
        }
    }
}
