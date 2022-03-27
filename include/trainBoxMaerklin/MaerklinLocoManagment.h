#pragma once

#include <Arduino.h>
#include "trainBoxMaerklin/MaerklinCanInterface.h"
#include "trainBoxMaerklin/MaerklinStationConfig.h"
#include <array>
#include <memory>
#include <vector>
#include "miniz.h"

// ToDo:
// handle different files that are received. Currently only locoInfo is used

class MaerklinLocoManagment
{
    public:
        struct LocoData
        {
            std::array<uint8_t, 16> name;
            std::vector<uint8_t> config;
        };

    public:
        // can interface is needed to request data over interface
        MaerklinLocoManagment(uint32_t uid, MaerklinCanInterface& interface, std::vector<MaerklinStationConfig>& stationList);
        virtual ~MaerklinLocoManagment();

        uint32_t getUid();

        void setHash(uint16_t hash){m_hash = hash;}

        void getAllLocos(std::vector<uint8_t>& locoList, std::vector<std::unique_ptr<LocoData>> &locos);

        void requestConfigData(const char* command, std::vector<uint8_t>& buffer);

        // called by MaerklinCanInterface
        //bool handleNewData(uint8_t* data);

        bool onConfigData(uint16_t hash, std::array<uint8_t, 8> data);

        bool onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc);

        bool onConfigDataStream(uint16_t hash, uint32_t streamlength, uint16_t crc, uint8_t res);

        bool onConfigDataStream(uint16_t hash, std::array<uint8_t, 8>& data);

        bool onConfigDataSteamError(uint16_t hash);

    protected:

        uint16_t updateCRC(uint16_t CRC_acc, uint8_t CRC_input);

        // ich muss den aktuellen Buffer jeweils umschalten => hierzu wird Minimum eine Statemaschine benötigt, welche kontrolliert,
        // was als nächstes gedownloaded wird
        // sobald die locolist gedownloaded ist, werden alle lokomotivnamen extahiert und gespeichert.
        // Anschließend wird jeweils mit push_back ein speicher angelegt, welcher dann request ConfigData übergeben wird
        std::vector<uint8_t>* m_buffer {nullptr};
        std::vector<std::unique_ptr<LocoData>>* m_locos {nullptr};

        std::vector<MaerklinStationConfig>& m_stationList;

        uint32_t m_uid {0};

        uint16_t m_hash {0};

        uint16_t m_hashExpected {0};

        bool m_isSenderHash {true};

        MaerklinCanInterface& m_interface;

        bool m_transmissionStarted {false};

        uint16_t m_crcExpected {0};

        uint32_t m_length {0};

        uint32_t m_lengthExpected {0};

        bool m_isZLib {false};
};