//----------------------------------------------------------------------------
// @file menu.rs
//
// @date 2021-08-28
// @author Martin Noblia
// @email mnoblia@disroot.org
//
// @brief
//
// @detail
//
// Licence MIT:
// Copyright <2021> <Martin Noblia>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.  THE SOFTWARE IS PROVIDED
// "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
// LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
// ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//----------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub enum DisplayState {
    Row1,
    Row2,
    Row3,
}

pub enum Message {
    Up,
    Down,
    Nop,
}

#[derive(Copy, Clone)]
pub struct DisplayStateMachine {
    current_state: DisplayState,
}

impl DisplayStateMachine {
    pub fn init(initial_state: DisplayState) -> Self {
        Self {
            current_state: initial_state,
        }
    }

    // NOTE(elsuizo:2021-09-01): this is a minimal state machine for the display behaviour
    pub fn dispatch(&mut self, msg: Message) {
        match (self.current_state, msg) {
            (DisplayState::Row1, Message::Up) => self.current_state = DisplayState::Row3,
            (DisplayState::Row1, Message::Down) => self.current_state = DisplayState::Row2,
            (DisplayState::Row2, Message::Up) => self.current_state = DisplayState::Row1,
            (DisplayState::Row2, Message::Down) => self.current_state = DisplayState::Row3,
            (DisplayState::Row3, Message::Up) => self.current_state = DisplayState::Row2,
            (DisplayState::Row3, Message::Down) => self.current_state = DisplayState::Row1,
            (_, Message::Nop) => {}
        }
    }
}
