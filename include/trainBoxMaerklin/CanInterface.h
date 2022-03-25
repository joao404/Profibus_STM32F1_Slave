/*********************************************************************
 * CanInterface
 *
 * Copyright (C) 2022 Marcel Maage
 * 
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * LICENSE file for more details.
 */

#pragma once

#include "Helper/Observer.h"
#include <driver/can.h>


class CanInterface : public Observable
{
public:
    CanInterface();
    virtual ~CanInterface();

    void begin();

    void cyclic();

    bool transmit(can_message_t& frame, uint16_t timeoutINms);

    bool receive(can_message_t& frame, uint16_t timeoutINms);

private:
    void errorHandling();
};