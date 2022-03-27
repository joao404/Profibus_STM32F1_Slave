#pragma once

#include <Arduino.h>

struct MaerklinStationConfig
{
    uint16_t hash;
    uint32_t id;
    uint16_t swVersion;
    uint16_t hwType;
};