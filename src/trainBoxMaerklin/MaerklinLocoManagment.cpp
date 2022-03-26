#include "trainBoxMaerklin/MaerklinLocoManagment.h"

MaerklinLocoManagment::MaerklinLocoManagment(uint32_t uid, MaerklinCanInterface &interface)
    : m_uid(uid), m_interface(interface)
{
}

MaerklinLocoManagment::~MaerklinLocoManagment()
{
}

uint32_t MaerklinLocoManagment::getUid()
{
    return m_uid;
}

// requestConfigData("lokliste", buffer);
void MaerklinLocoManagment::requestConfigData(const char *command, std::vector<uint8_t> &buffer)
{
    m_transmissionStarted = true;
    m_buffer = &buffer;
    m_buffer->clear();
    std::array<uint8_t, 8> request;
    while (*command)
    {
        for (uint8_t i = 0; i < request.size(); i++, command++)
        {
            if ('\0' != *command)
            {
                if ('~' == *command)
                {
                    request[i] = 0;
                }
                else
                {
                    request[i] = *command;
                }
            }
            else // the rest should be zero
            {
                request[i] = 0;
            }
        }
        Serial.print("Sending:");
        for (auto i : request)
        {
            Serial.print((char)i);
        }
        Serial.print("\n");
        m_interface.requestConfigData(request);
        if ('\0' == *command)
        {
            // delay is not good and should be replaced by a state machine in the near future
            delayMicroseconds(50);
        }
    }
}

// called by MaerklinCanInterface
// bool MaerklinLocoManagment::handleNewData(uint8_t* data)
// {

// }

bool MaerklinLocoManagment::onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc)
{
    Serial.println("onConfigDataStream6");
    bool result{false};
    // receiver hash is expected
    if (m_transmissionStarted)
    {
        if (hash == m_hash)
        {
            m_hashExpected = hash;
            m_lengthExpected = streamlength;
            m_length = 0;
            m_crcExpected = crc;
            result = true;
        }
    }
    return result;
}

bool MaerklinLocoManagment::onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc, uint8_t res)
{
    Serial.println("onConfigDataStream7");
    bool result{false};
    // sender hash is expected
    if (m_transmissionStarted)
    {
        m_hashExpected = hash;
        m_lengthExpected = streamlength;
        m_length = 0;
        m_crcExpected = crc;
        result = true;
    }
    return result;
}

bool MaerklinLocoManagment::onConfigDataStream(uint16_t hash, std::array<uint8_t, 8> &data)
{
    Serial.println("onConfigDataStream8");
    bool result{false};
    // hash is expected the same as in start message
    if (m_transmissionStarted)
    {
        if (hash == m_hashExpected)
        {
            /*
            for (auto i : data)
            {
                Serial.print((char)i);
            }
            Serial.print("\n");
            */
            result = true;
            if (nullptr != m_buffer)
            {
                for (auto i : data)
                {
                    m_buffer->push_back(i);
                }
            }
            m_length += 8;
            // ToDo:
            m_crc = calcCRC(0, 0);

            if (m_length == m_lengthExpected)
            {
                Serial.println("Successsfull reading");
                if (nullptr != m_buffer)
                {
                    for (auto i : (*m_buffer))
                    {
                        Serial.print((char)i);
                    }
                    Serial.print("\n");
                }
                
                if (m_crc == m_crcExpected)
                {
                    // Success
                    // copy buffer to new location and request next file if needed
                }
            }
        }
    }
    return result;
}

bool MaerklinLocoManagment::onConfigDataSteamError(uint16_t hash)
{
    Serial.println("onConfigDataSteamError");
    if (m_transmissionStarted && (hash == m_hashExpected))
    {
        m_transmissionStarted = false;
        m_lengthExpected = 0;
        m_hashExpected = 0;
        return true;
    }
    return false;
}

uint16_t MaerklinLocoManagment::calcCRC(uint16_t CRC_acc, uint8_t CRC_input)
{
    //     u16 CtDataSender::updateCRC (u16 CRC_acc, u8 CRC_input)
    // {
    // #define POLY 0x1021
    // // Create the CRC "dividend" for polynomial arithmetic (binary arithmetic with no carries)
    // CRC_acc = CRC_acc ^ (CRC_input << 8);
    // // "Divide" the poly into the dividend using CRC XOR subtraction CRC_acc holds the
    // // "remainder" of each divide. Only complete this division for 8 bits since input is 1 byte
    // for (int i = 0; i < 8; i++) {
    // // Check if the MSB is set (if MSB is 1, then the POLY can "divide" into the "dividend")
    // if ((CRC_acc & 0x8000) == 0x8000) {
    // // if so, shift the CRC value, and XOR "subtract" the poly
    // CRC_acc = CRC_acc << 1;
    // CRC_acc ^= POLY;
    // }
    // else {
    // // if not, just shift the CRC value
    // CRC_acc = CRC_acc << 1;
    // }
    // }
    // // Return the final remainder (CRC value)
    // return CRC_acc;
    // }
    return CRC_acc;
}