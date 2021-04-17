//-------------------------------------------------------------------------
// @file logger.rs
//
// @date 03/30/21 12:35:46
// @author Martin Noblia
// @email mnoblia@disroot.org
//
// @brief
//
// @detail
//
//  Licence:
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
//--------------------------------------------------------------------------
// esto lo saque de https://github.com/ferrous-systems/internet-of-streams/blob/master/sensor-node/src/logger.rs
// quiero ver si se puede utilizar para el stm32

extern crate embedded_hal;
extern crate nb;

use embedded_hal as hal;

use hal::serial::Write;
use nb::block;
use stm32f1xx_hal::pac::USART1;

use stm32f1xx_hal::serial::Tx;

pub struct Logger {
    tx_pin: Tx<USART1>,
}

impl Logger {
    pub fn new(tx_pin: Tx<USART1>) -> Self {
        Self { tx_pin }
    }

    pub fn log(&mut self, data: &str) -> Result<(), ()> {
        self.send("LOG: ".as_bytes())?;
        self.send(data.as_bytes())?;
        self.send("\r\n".as_bytes())
    }

    pub fn warn(&mut self, data: &str) -> Result<(), ()> {
        self.send("WRN: ".as_bytes())?;
        self.send(data.as_bytes())?;
        self.send("\r\n".as_bytes())
    }

    pub fn error(&mut self, data: &str) -> Result<(), ()> {
        self.send("ERR: ".as_bytes())?;
        self.send(data.as_bytes())?;
        self.send("\r\n".as_bytes())
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<(), ()> {
        for &byte in buf {
            if byte == 0x00 {
                continue;
            }
            block!(self.tx_pin.write(byte)).unwrap();
        }
        Ok(())
    }
}
