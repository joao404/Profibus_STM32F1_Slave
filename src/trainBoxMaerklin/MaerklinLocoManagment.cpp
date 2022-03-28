#include "trainBoxMaerklin/MaerklinLocoManagment.h"

MaerklinLocoManagment::MaerklinLocoManagment(uint32_t uid, MaerklinCanInterface &interface,
                                             std::vector<MaerklinStationConfig> &stationList, unsigned long messageTimeout,
                                             uint8_t maxCmdRepeat, bool debug)
    : MaerklinConfigDataStream(interface, stationList),
      m_uid(uid), m_cmdTimeoutINms(messageTimeout),
      m_maxCmdRepeat(maxCmdRepeat),
      m_debug(debug)
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

void MaerklinLocoManagment::getAllLocos(std::string &locoList, std::vector<std::unique_ptr<LocoData>> &locos, void (*callback)(bool success))
{
    m_callbackFunc = callback;
    m_locos = &locos;
    m_state = LocoManagmentState::WaitingForLocoList;
    m_currentInfo.clear();
    m_currentBuffer = &locoList;
    m_currentType = DataType::Lokliste;
    startConfigDataRequest(m_currentType, &m_currentInfo, m_currentBuffer, 1);
}

bool MaerklinLocoManagment::startConfigDataRequest(DataType type, std::string *info, std::string *buffer)
{
    if (requestConfigData(type, info, buffer))
    {
        m_transmissionStarted = true;
        m_lastCmdTimeINms = millis();
        return true;
    }
    return false;
}

bool MaerklinLocoManagment::startConfigDataRequest(DataType type, std::string *info, std::string *buffer, uint8_t cmdRepeat)
{
    if (startConfigDataRequest(type, info, buffer))
    {
        m_cmdRepeat = cmdRepeat;
        return true;
    }
    return false;
}

void MaerklinLocoManagment::handleConfigDataStreamFeedback(std::string *data, uint16_t hash, bool success)
{
    if (m_transmissionStarted) // message expected
    {
        if (success)
        {
            m_transmissionStarted = false;
            // analyze buffer depending on current state
            if (nullptr != data)
            {
                if (m_debug)
                {
                    for (auto i : (*data))
                    {
                        Serial.print((char)i);
                    }
                    Serial.print("\n");
                }
            }
            switch (m_state)
            {
            case LocoManagmentState::WaitingForLocoList:
            {
                if (m_debug)
                {
                    Serial.println("Received Lokliste");
                }
                // extract all loconames and switch to lokinfo for getting data
                size_t nameStringSize = strlen(".name=");
                m_locos->clear();
                for (size_t index = 0, nameEnd = 0; (index = m_currentBuffer->find(".name=", index)) != std::string::npos; index = nameEnd)
                {
                    index += nameStringSize;
                    nameEnd = m_currentBuffer->find('\n', index);
                    m_locos->emplace_back(std::unique_ptr<LocoData>(new LocoData{m_currentBuffer->substr(index, nameEnd - index), ""}));
                    // Serial.print("Found loco:");
                    // Serial.print(m_locos->back()->name.c_str());
                    // Serial.print(" with size ");
                    // Serial.println(m_currentBuffer->substr(index, nameEnd - index).size());
                }
                m_currentLocoNum = 0;
                if (m_locos->size() > 0)
                {
                    if (m_debug)
                    {
                        Serial.print("Found ");
                        Serial.print(m_locos->size());
                        Serial.println(" locos");
                    }
                    m_state = LocoManagmentState::WaitingForLocoInfo;
                    m_currentType = DataType::Lokinfo;
                    m_currentInfo = m_locos->at(m_currentLocoNum)->name;
                    m_currentBuffer = &(m_locos->at(m_currentLocoNum)->config);
                    startConfigDataRequest(m_currentType, &m_currentInfo, m_currentBuffer, 1);
                }
                else
                {
                    Serial.println("No locos could be read");
                }
            }
            break;
            case LocoManagmentState::WaitingForLocoInfo:
            {
                m_currentLocoNum++;
                if (m_currentLocoNum < m_locos->size())
                {
                    m_state = LocoManagmentState::WaitingForLocoInfo;
                    m_currentType = DataType::Lokinfo;
                    m_currentInfo = m_locos->at(m_currentLocoNum)->name;
                    m_currentBuffer = &(m_locos->at(m_currentLocoNum)->config);
                    startConfigDataRequest(m_currentType, &m_currentInfo, m_currentBuffer, 1);
                }
            }
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
                startConfigDataRequest(m_currentType, &m_currentInfo, m_currentBuffer);
            }
            else
            {
                m_transmissionStarted = false;
                switch (m_state)
                {
                case LocoManagmentState::WaitingForLocoList:
                    // Does not seem to work
                    // Switch to old version
                    m_state = LocoManagmentState::WaitingForLocoNamen;
                    m_currentInfo = "0 2";
                    m_currentType = DataType::Loknamen;
                    startConfigDataRequest(m_currentType, &m_currentInfo, m_currentBuffer, 1);
                    break;

                case LocoManagmentState::WaitingForLocoInfo:
                {
                    Serial.print("Could not get loco:");
                    Serial.println(m_locos->at(m_currentLocoNum)->name.c_str());
                    m_currentLocoNum++;
                    if (m_currentLocoNum < m_locos->size())
                    {
                        m_state = LocoManagmentState::WaitingForLocoInfo;
                        m_currentType = DataType::Lokinfo;
                        m_currentInfo = m_locos->at(m_currentLocoNum)->name;
                        m_currentBuffer = &(m_locos->at(m_currentLocoNum)->config);
                        startConfigDataRequest(m_currentType, &m_currentInfo, m_currentBuffer, 1);
                    }
                }
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
            Serial.print((uint8_t)m_currentType);
            Serial.print(" ");
            Serial.println(m_currentInfo.c_str());
            // transmission timed out
            handleConfigDataStreamFeedback(nullptr, 0, false);
        }
    }
}
