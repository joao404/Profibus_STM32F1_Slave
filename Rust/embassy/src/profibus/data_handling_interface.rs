/*********************************************************************
 * DataHandlingInterface
 *
 * Copyright (C) 2023 Marcel Maage
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

pub trait DataHandlingInterface{ 

    fn config_error_led(&mut self) {}

    fn error_led_on(&mut self) {}

    fn error_led_off(&mut self) {}

    fn millis(&mut self) -> u32 {
        0
    }

    fn data_processing(&self, _input: &mut [u8], _output: &[u8]) {}

    fn debug_write(&mut self, _debug: &str) {}
}