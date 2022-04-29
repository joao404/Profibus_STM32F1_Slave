/*********************************************************************
 * Profibus Stm32f1 Slave
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

/* Includes ------------------------------------------------------------------*/
#include "main.h"
#include "main.c"
#include "can.h"
#include "usart.h"
#include "gpio.h"
#include "tim.h"
#include "xprintf.h"
#include "profibus/CProfibusSlaveStm32f1.h"
#include <vector>


CProfibusSlave::Config pbConfig;

void uart_putc(uint8_t d) {
	HAL_UART_Transmit(&huart1, &d, 1, 1000);
}

uint8_t uart_getc(void) {
	uint8_t d;

	(uint8_t)HAL_UART_Receive(&huart1, &d, 1, 1000);

	return d;
}

void DataExchange(std::vector<uint8_t>& outputBuf, std::vector<uint8_t>& inputBuf)
{
  /*
  if(outputbuf!=NULL)
  {
    for(int i=0;i<OUTPUT_DATA_SIZE;i++)
    {
      Serial.print(outputbuf[i]);
    }
    Serial.println();
  } 
  */

  static int counter = 0;
  if(0 != inputBuf.size())
  {
    HAL_GPIO_TogglePin(LED_BUILTIN_GPIO_Port, LED_BUILTIN_Pin);
    for(uint8_t i=0;i<inputBuf.size();i++)
    {
      inputBuf[i]=static_cast<uint8_t>(counter+i);
    }
    counter++;
  } 
}

void debugOutput(uint8_t* buffer, uint8_t len)
{
  HAL_UART_Transmit_IT(&huart1, buffer, len);
}


int main(void)
{
  xdev_in(uart_getc);
	xdev_out(uart_putc);
  HAL_Init();
  SystemClock_Config();
  MX_GPIO_Init();
  MX_USART1_UART_Init();

  xprintf("PB Slave Systemfrequenz: %d\n",HAL_RCC_GetSysClockFreq());

  pbConfig.counterFrequency = F_CPU;
  pbConfig.speed = 500000;
  pbConfig.identHigh = 0x00;
  pbConfig.identLow = 0x2B;
  pbConfig.bufSize = 45;
  pbConfig.inputDataSize = 2;
  pbConfig.outputDataSize = 5;
  pbConfig.moduleCount = 5;
  pbConfig.userParaSize = 0;
  pbConfig.externDiagParaSize = 0;
  pbConfig.vendorDataSize = 0;

  pbInterface.init_Profibus(pbConfig, DataExchange, debugOutput);

  while (1)
  {
    __NOP();
  }
}