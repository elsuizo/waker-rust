/// Button primitives and implementations
use core::convert::Infallible;
use embedded_hal::digital::v2::InputPin;

pub enum PinState {
    PinUp,
    PinDown,
    Nothing,
}

type Counter = u8;

#[derive(Copy, Clone)]
enum ButtonState {
    High(Counter),
    Low(Counter),
}

pub struct Button<P> {
    typ: P,
    state: ButtonState,
}

// TODO(elsuizo:2021-11-26): look what is the better COUNTER_THRESOLD parameter for this
impl<P: InputPin<Error = Infallible>> Button<P> {
    const COUNTER_THRESOLD: u8 = 15;

    pub fn new(typ: P) -> Self {
        Self {
            typ,
            state: ButtonState::High(0u8),
        }
    }

    /// poll the pin and generate a debounce algorithm:
    pub fn poll(&mut self) -> PinState {
        use self::ButtonState::*;
        let value = self.typ.is_high().expect("could this fail???");
        match (&mut self.state, value) {
            (High(counter), true) => *counter = 0,
            (High(counter), false) => *counter += 1,
            (Low(counter), true) => *counter += 1,
            (Low(counter), false) => *counter = 0,
        }
        match self.state {
            High(counter) if counter >= Self::COUNTER_THRESOLD => {
                self.state = Low(0);
                PinState::PinUp
            }
            Low(counter) if counter >= Self::COUNTER_THRESOLD => {
                self.state = High(0);
                PinState::PinDown
            }
            _ => PinState::Nothing,
        }
    }
}
