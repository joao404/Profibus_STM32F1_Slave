#pragma once

#include <Arduino.h>
#include <trainBoxMaerklin/MaerklinCanInterface.h>
#include <array>

// ToDo:
// handle different files that are received. Currently only locoInfo is used

class MaerklinLocoManagment
{
    public:
        // can interface is needed to request data over interface
        MaerklinLocoManagment(uint32_t uid, MaerklinCanInterface& interface);
        virtual ~MaerklinLocoManagment();

        uint32_t getUid();

        void setHash(uint16_t hash){m_hash = hash;}

        void requestLocoInfo();

        // called by MaerklinCanInterface
        //bool handleNewData(uint8_t* data);

        bool onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc);

        bool onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc, uint8_t res);

        bool onConfigDataStream(uint16_t hash, std::array<uint8_t, 8>& data);

        bool onConfigDataSteamError(uint16_t hash);

    protected:

        uint16_t calcCRC(uint16_t CRC_acc, uint8_t CRC_input);

        std::vector<uint8_t> locoinfo;

        uint32_t m_uid {0};

        uint16_t m_hash {0};

        uint16_t m_hashExpected {0};

        bool m_isSenderHash {true};

        MaerklinCanInterface& m_interface;

        bool m_transmissionStarted {false};

        uint16_t m_crc {0};

        uint16_t m_crcExpected {0};

        uint32_t m_length {0};

        uint32_t m_lengthExpected {0};
};