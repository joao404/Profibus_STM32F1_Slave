/*********************************************************************
 * TrainBox Maerklin Esp32
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

#include <Arduino.h>
#include <memory>
#include "trainBoxMaerklin/MaerklinCanInterface.h"
#include "trainBoxMaerklin/CanInterface.h"
#include "Helper/Observer.h"
#include <driver/can.h>

//#define CAN_DEBUG


class MaerklinCanInterfaceEsp32 : public MaerklinCanInterface, public Observer
{
    public:


	/**
	 * Creates a new TrackController with the given hash and debugging
	 * flag. A zero hash will result in a unique hash begin generated.
	 */
    MaerklinCanInterfaceEsp32(word hash, bool debug);

    /**
     * Is called when a TrackController is being destroyed. Does the
     * necessary cleanup. No need to call this manually.
     */
    virtual ~MaerklinCanInterfaceEsp32();

    // set can observer for receiving and writing messages
    bool setCanObserver(std::shared_ptr<CanInterface> canInterface);

    /**
     * Initializes the CAN hardware and starts receiving CAN
     * messages. CAN messages are put into an internal buffer of
     * limited size, so they don't get lost, but you have to take
     * care of them in time. Otherwise the buffer might overflow.
     */
    void begin() override;

    /**
     * Stops receiving messages from the CAN hardware. Clears
     * the internal buffer.
     */
    void end() override;

    /**
     * Sends a message and reports true on success. Internal method.
     * Normally you don't want to use this, but the more convenient
     * methods below instead.
     */
    bool sendMessage(TrackMessage &message) override;

    /**
     * Receives an arbitrary message, if available, and reports true
     * on success. Does not block. Internal method. Normally you
     * don't want to use this, but the more convenient methods below
     * instead.
     */
    bool receiveMessage(TrackMessage &message) override;

    virtual void notifyCanReceived(can_message_t frame){};

    void cyclic();

    void update(Observable& observable, void* data);

    void errorHandling();


    
    private:
        std::shared_ptr<CanInterface> m_canInterface;
};
