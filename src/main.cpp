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

void uart_putc(uint8_t d) {
	HAL_UART_Transmit(&huart1, &d, 1, 1000);
}

uint8_t uart_getc(void) {
	uint8_t d;

	(uint8_t)HAL_UART_Receive(&huart1, &d, 1, 1000);

	return d;
}

void DataExchange(volatile uint8_t *outputbuf,volatile uint8_t *inputbuf)
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
  if(inputbuf!=NULL)
  {
    HAL_GPIO_TogglePin(LED_BUILTIN_GPIO_Port, LED_BUILTIN_Pin);
    for(int i=0;i<INPUT_DATA_SIZE;i++)
    {
      inputbuf[i]=static_cast<uint8_t>(counter+i);
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

  pbInterface.init_Profibus(0x00, 0x2B, DataExchange, debugOutput);

  while (1)
  {
    __NOP();
  }
}