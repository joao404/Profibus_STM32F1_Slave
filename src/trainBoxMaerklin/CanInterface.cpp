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

#include "trainBoxMaerklin/CanInterface.h"
#include <Arduino.h>
#include <driver/can.h>
#include <driver/gpio.h>
#include <esp_system.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "freertos/semphr.h"

CanInterface::CanInterface()
{
}

CanInterface::~CanInterface()
{
}

void CanInterface::begin()
{
    Serial.println(F("Setting up CAN..."));
    /* set CAN pins and baudrate */
    can_general_config_t general_config = {
        .mode = CAN_MODE_NORMAL,
        .tx_io = (gpio_num_t)GPIO_NUM_4,//11,//5,
        .rx_io = (gpio_num_t)GPIO_NUM_5,//10,//4,
        .clkout_io = (gpio_num_t)CAN_IO_UNUSED,
        .bus_off_io = (gpio_num_t)CAN_IO_UNUSED,
        .tx_queue_len = 120,
        .rx_queue_len = 120,
        .alerts_enabled = CAN_ALERT_ABOVE_ERR_WARN | CAN_ALERT_ERR_PASS | CAN_ALERT_BUS_OFF | CAN_ALERT_BUS_RECOVERED |
                          CAN_ALERT_RX_QUEUE_FULL | CAN_ALERT_BUS_ERROR, // CAN_ALERT_NONE,
        .clkout_divider = 0};
    can_timing_config_t timing_config = CAN_TIMING_CONFIG_250KBITS();
    can_filter_config_t filter_config = CAN_FILTER_CONFIG_ACCEPT_ALL();
    esp_err_t error;

    error = can_driver_install(&general_config, &timing_config, &filter_config);
    if (error == ESP_OK)
    {
        Serial.println(F("CAN Driver installation success..."));
    }
    else
    {
        Serial.println(F("CAN Driver installation fail..."));
        return;
    }

    // start CAN driver
    error = can_start();
    if (error == ESP_OK)
    {
        Serial.println(F("CAN Driver start success..."));
    }
    else
    {
        Serial.println(F("CAN Driver start FAILED..."));
        return;
    }
}

void CanInterface::cyclic()
{
    can_message_t frame;
    while (can_receive(&frame, 0) == ESP_OK)
    {
        notify(&frame);
    }
    errorHandling();
}

bool CanInterface::transmit(can_message_t& frame, uint16_t timeoutINms)
{
    bool result {true};
    if(can_transmit(&frame, pdMS_TO_TICKS(timeoutINms)) != ESP_OK)
    {
        result = false;
        errorHandling();
    }
    return result;
}

bool CanInterface::receive(can_message_t& frame, uint16_t timeoutINms)
{
    return (can_receive(&frame, pdMS_TO_TICKS(timeoutINms)) == ESP_OK);
}

void CanInterface::errorHandling()
{
    uint32_t alerts;
    can_read_alerts(&alerts, 0);
    if (alerts & CAN_ALERT_ABOVE_ERR_WARN)
    {
        Serial.println(F("Surpassed Error Warning Limit"));
    }
    if (alerts & CAN_ALERT_ERR_PASS)
    {
        Serial.println(F("Entered Error Passive state"));
    }
    if (alerts & CAN_ALERT_BUS_OFF)
    {
        Serial.println(F("Bus Off state"));
        // Prepare to initiate bus recovery, reconfigure alerts to detect bus recovery completion
        // can_reconfigure_alerts(CAN_ALERT_BUS_RECOVERED, NULL);
        for (int i = 3; i > 0; i--)
        {
            Serial.print(F("Initiate bus recovery in"));
            Serial.println(i);
            vTaskDelay(pdMS_TO_TICKS(1000));
        }
        can_initiate_recovery(); // Needs 128 occurrences of bus free signal
        Serial.println(F("Initiate bus recovery"));
    }
    if (alerts & CAN_ALERT_BUS_RECOVERED)
    {
        // Bus recovery was successful, exit control task to uninstall driver
        Serial.println(F("Bus Recovered"));
        if (can_start() == ESP_OK)
        {
            Serial.println(F("CAN Driver start success..."));
        }
    }
    if (alerts & CAN_ALERT_RX_QUEUE_FULL)
    {
        Serial.println(F("RxFull"));
    }
    if (alerts & CAN_ALERT_BUS_ERROR)
    {
        Serial.println(F("BusError"));
    }
}