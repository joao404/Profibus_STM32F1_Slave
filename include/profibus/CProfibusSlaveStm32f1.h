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

#pragma once
#include "profibus/CProfibusSlave.h"
#include "usart.h"

class CProfibusSlaveStm32f1: public CProfibusSlave{
    protected:

    void configTimer(void) override {MX_TIM2_Init();HAL_TIM_Base_Start_IT(&htim2);}

    void runTimer(void) override {__HAL_TIM_ENABLE(&htim2);}

    void stopTimer(void) override {__HAL_TIM_DISABLE(&htim2);}

    void setTimerCounter(uint32_t value) override {__HAL_TIM_SET_COUNTER(&htim2, value);}

    //void setTimerMax(uint16_t value) override {htim2.Instance->CCR1=value;}
    void setTimerMax(uint32_t value) override {__HAL_TIM_SET_AUTORELOAD(&htim2, value);}

    //void clearOverflowFlag(void) override {__HAL_TIM_CLEAR_FLAG(&htim2, TIM_FLAG_CC1);}
    void clearOverflowFlag(void) override {__HAL_TIM_CLEAR_FLAG(&htim2, TIM_FLAG_UPDATE);}

    void configUart(void) override {  MX_USART3_UART_Init(); HAL_NVIC_SetPriority(USART3_IRQn, 0, 0); HAL_NVIC_EnableIRQ(USART3_IRQn);}

    void waitForActivTransmission(void) override {while (__HAL_UART_GET_FLAG(&huart3, UART_FLAG_TXE) != SET);}

    void activateTxInterrupt(void) override {clearTxFlag();__HAL_UART_ENABLE_IT(&huart3, UART_IT_TC);}

    void deactivateTxInterrupt(void) override {__HAL_UART_DISABLE_IT(&huart3, UART_IT_TC);}

    void activateRxInterrupt(void) override {__HAL_UART_ENABLE_IT(&huart3, UART_IT_RXNE);}

    void deactivateRxInterrupt(void) override {__HAL_UART_DISABLE_IT(&huart3, UART_IT_RXNE);}

    void setTxFlag(void) override {huart3.Instance->SR |= UART_FLAG_TC;}

    void clearTxFlag(void) override {huart3.Instance->SR &= ~(UART_FLAG_TC);}

    void clearRxFlag(void) override {huart3.Instance->SR &= ~UART_FLAG_RXNE;}

    void configRs485Pin(void) override {HAL_GPIO_WritePin(GPIOB, PB_TX_EN_Pin , GPIO_PIN_RESET);HAL_GPIO_WritePin(GPIOB, PB_RX_EN_Pin , GPIO_PIN_SET);}//nothing to do

    void TxRs485Enable(void) override {HAL_GPIO_WritePin(GPIOB, PB_RX_EN_Pin , GPIO_PIN_SET);HAL_GPIO_WritePin(GPIOB, PB_TX_EN_Pin , GPIO_PIN_SET);}

    void TxRs485Disable(void) override {HAL_GPIO_WritePin(GPIOB, PB_TX_EN_Pin , GPIO_PIN_RESET);HAL_GPIO_WritePin(GPIOB, PB_RX_EN_Pin , GPIO_PIN_RESET);}

    void RxRs485Enable(void) override {HAL_GPIO_WritePin(GPIOB, PB_TX_EN_Pin , GPIO_PIN_RESET);HAL_GPIO_WritePin(GPIOB, PB_RX_EN_Pin , GPIO_PIN_RESET);}

    uint8_t getUartValue(void) override {return huart3.Instance->DR;}

    void setUartValue(uint8_t value) override {huart3.Instance->DR = value;}

    void configErrorLed(void) override {}//nothing to do

    void errorLedOn(void) override {HAL_GPIO_WritePin(LED_BUILTIN_GPIO_Port, LED_BUILTIN_Pin , GPIO_PIN_SET);}

    void errorLedOff(void) override {HAL_GPIO_WritePin(LED_BUILTIN_GPIO_Port, LED_BUILTIN_Pin , GPIO_PIN_RESET);}

    uint32_t millis(void) override {return HAL_GetTick();}

};

static CProfibusSlaveStm32f1 pbInterface{};

extern "C" void USART3_IRQHandler(void)
{
  uint32_t isrflags   = READ_REG(huart3.Instance->SR);
  uint32_t cr1its     = READ_REG(huart3.Instance->CR1);

  /* If no error occurs */
  uint32_t errorflags = (isrflags & (uint32_t)(USART_SR_PE | USART_SR_FE | USART_SR_ORE | USART_SR_NE));
  if (errorflags == RESET)
  {
    if (((isrflags & USART_SR_RXNE) != RESET) && ((cr1its & USART_CR1_RXNEIE) != RESET))
    {
      pbInterface.interruptPbRx();
    }
  }
/*
  if (((isrflags & USART_SR_TXE) != RESET) && ((cr1its & USART_CR1_TXEIE) != RESET))
  {
    //pbInterface.interruptPbTx();
    huart3.Instance->SR &= ~UART_FLAG_TXE;
  }
*/

  if (((isrflags & USART_SR_TC) != RESET) && ((cr1its & USART_CR1_TCIE) != RESET))
  {
    pbInterface.interruptPbTx();
    huart3.Instance->SR &= ~UART_FLAG_TC;
  }
}

void HAL_TIM_PeriodElapsedCallback(TIM_HandleTypeDef* htim)
{
  pbInterface.interruptTimer();
}

