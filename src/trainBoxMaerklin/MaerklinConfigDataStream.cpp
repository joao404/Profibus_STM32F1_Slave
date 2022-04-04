#include "trainBoxMaerklin/MaerklinConfigDataStream.h"

MaerklinConfigDataStream::MaerklinConfigDataStream(MaerklinCanInterface &interface, std::vector<MaerklinStationConfig> &stationList)
    : m_interface(interface), m_stationList(stationList)
{
}

MaerklinConfigDataStream::~MaerklinConfigDataStream()
{
}

bool MaerklinConfigDataStream::requestConfigData(DataType type, std::string *info, std::string *buffer)
{
    if (nullptr == buffer)
    {
        return false;
    }
    m_buffer = buffer;
    m_buffer->clear();
    m_lengthExpected = 0xFFFFFFFFUL;
    m_length = 0;

    switch (type)
    {
    case DataType::Lokliste:
    {
        std::array<uint8_t, 8> request = {'l', 'o', 'k', 'l', 'i', 's', 't', 'e'};
        m_interface.requestConfigData(request);
    }
    break;
    case DataType::Lokinfo:
    {
        std::array<uint8_t, 8> request = {'l', 'o', 'k', 'i', 'n', 'f', 'o', 0};
        m_interface.requestConfigData(request);
        Serial.println(info->c_str());
        uint8_t numberOfRequests{0};
        size_t infoIndex{0};
        do
        {
            delayMicroseconds(50);
            for (uint8_t i = 0; i < request.size(); i++)
            {
                if (info->size() > infoIndex)
                {
                    request[i] = (*info)[infoIndex];
                    infoIndex++;
                }
                else // the rest should be zero
                {
                    request[i] = 0;
                }
            }
            m_interface.requestConfigData(request);
            numberOfRequests++;
        } while (numberOfRequests < 2);
    }
    break;
    case DataType::Loknamen:
    {
        std::array<uint8_t, 8> request = {'l', 'o', 'k', 'n', 'a', 'm', 'e', 'n'};
        m_interface.requestConfigData(request);
        delayMicroseconds(50);
        size_t infoIndex{0};
        for (uint8_t i = 0; i < request.size(); i++)
        {
            if (info->size() > infoIndex)
            {
                request[i] = (*info)[infoIndex];
                infoIndex++;
            }
            else // the rest should be zero
            {
                request[i] = 0;
            }
        }
        m_interface.requestConfigData(request);
    }
    break;
    case DataType::MagInfo:
    {
        std::array<uint8_t, 8> request = {'m', 'a', 'g', 'i', 'n', 'f', 'o', 0};
        m_interface.requestConfigData(request);
        delayMicroseconds(50);
        size_t infoIndex{0};
        for (size_t i = 0; i < request.size(); i++)
        {
            if (info->size() > infoIndex)
            {
                request[i] = (*info)[infoIndex];
                infoIndex++;
            }
            else // the rest should be zero
            {
                request[i] = 0;
            }
        }
        m_interface.requestConfigData(request);
    }
    break;
    case DataType::Lokdb:
    {
        std::array<uint8_t, 8> request = {'l', 'o', 'k', 'd', 'b', 0, 0, 0};
        m_interface.requestConfigData(request);
    }
    break;
    default:
        Serial.println("DataType not supported");
        return false;
    }
    return true;
}

bool MaerklinConfigDataStream::onConfigData(uint16_t hash, std::array<uint8_t, 8> data)
{
    Serial.println("onConfigData");
    // expected hash can be set here in case of ConfigDataStream7
    m_hashExpected = hash;
    return true;
}

bool MaerklinConfigDataStream::onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc)
{
    // Serial.println("onConfigDataStream6");
    // receiver hash is expected
    m_hashExpected = hash;
    m_lengthExpected = streamlength;
    m_length = 0;
    m_crcExpected = crc;
    m_buffer->clear();
    m_configDataStreamStartReceived = true;
    return true;
}

bool MaerklinConfigDataStream::onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc, uint8_t res)
{
    // Serial.println("onConfigDataStream7");
    // sender hash is expected
    m_hashExpected = hash; // is already set in onConfigData
    m_lengthExpected = streamlength;
    // Serial.print("m_lengthExpected:");
    // Serial.println(m_lengthExpected);
    m_length = 0;
    m_crcExpected = crc;
    m_buffer->clear();
    m_configDataStreamStartReceived = true;
    return true;
}

bool MaerklinConfigDataStream::onConfigDataStream(uint16_t hash, std::array<uint8_t, 8> &data)
{
    // Serial.println("onConfigDataStream8");
    bool result{false};
    // hash is expected the same as in start message
    if (m_configDataStreamStartReceived)
    {
        if (hash == m_hashExpected)
        {
            // for (auto i : data)
            // {
            //     Serial.print((char)i);
            // }
            // Serial.print("\n");

            result = true;
            if (nullptr != m_buffer)
            {
                for (auto i : data)
                {
                    m_buffer->push_back(i);
                }
            }
            m_length += 8;

            // Serial.print("m_length");
            // Serial.println(m_length);
            if (m_length >= m_lengthExpected)
            {
                m_configDataStreamStartReceived = false;
                // Serial.println("Successsfull reading");
                // if (nullptr != m_buffer)
                // {
                //     for (auto i : (*m_buffer))
                //     {
                //         Serial.print((char)i);
                //     }
                //     Serial.print("\n");
                // }

                uint16_t CRC_acc = 0xFFFF;
                for (uint8_t i : (*m_buffer))
                {
                    CRC_acc = updateCRC(CRC_acc, i);
                }
                // Serial.print("CRC:");
                // Serial.print(m_crcExpected);
                // Serial.print("::");
                // Serial.println(CRC_acc);
                if (m_crcExpected == CRC_acc)
                {
                    // if (nullptr != m_reportResultFunc)
                    // {
                    //     m_reportResultFunc(m_buffer, hash, true);
                    // }
                    Serial.println("CRC success");
                    m_length = 0;
                    m_reportResultFunc(m_buffer, hash, true);
                    m_buffer->clear();
                    // Success
                    // copy buffer to new location and request next file if needed
                }
                else
                {
                    // if (nullptr != m_reportResultFunc)
                    // {
                    //     m_reportResultFunc(m_buffer, hash, false);
                    // }
                    Serial.printf("CRC failed %d : %d", m_crcExpected, CRC_acc);
                    m_length = 0;
                    m_reportResultFunc(nullptr, hash, false);
                    m_buffer->clear();                    
                }
            }
        }
    }
    else
    {
        // we received data before trigger message. ignore this data
    }
    return result;
}

bool MaerklinConfigDataStream::onConfigDataSteamError(uint16_t hash)
{
    Serial.println("onConfigDataSteamError");
    if (m_configDataStreamStartReceived && (hash == m_hashExpected))
    {
        m_configDataStreamStartReceived = false;
        m_lengthExpected = 0;
        m_hashExpected = 0;
        m_buffer = &m_backupBuffer;
        m_reportResultFunc(nullptr, hash, false);
        return true;
    }
    return false;
}

uint16_t MaerklinConfigDataStream::updateCRC(uint16_t CRC_acc, uint8_t CRC_input)
{
#define POLY 0x1021
    // Create the CRC "dividend" for polynomial arithmetic (binary arithmetic with no carries)
    CRC_acc = CRC_acc ^ (CRC_input << 8);
    // "Divide" the poly into the dividend using CRC XOR subtraction CRC_acc holds the
    // "remainder" of each divide. Only complete this division for 8 bits since input is 1 byte
    for (int i = 0; i < 8; i++)
    {
        // Check if the MSB is set (if MSB is 1, then the POLY can "divide" into the "dividend")
        if ((CRC_acc & 0x8000) == 0x8000)
        {
            // if so, shift the CRC value, and XOR "subtract" the poly
            CRC_acc = CRC_acc << 1;
            CRC_acc ^= POLY;
        }
        else
        {
            // if not, just shift the CRC value
            CRC_acc = CRC_acc << 1;
        }
    }
    // Return the final remainder (CRC value)
    return CRC_acc;
}