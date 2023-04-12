/// User interface primitives
use embedded_graphics::{
    image::{Image, ImageRawLE},
    mono_font::{ascii::FONT_9X15, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

// TODO(elsuizo:2021-11-28): use this constants for a better text positions
// pub const DISPLAY_WIDTH: i32 = 128;
// pub const DISPLAY_HEIGHT: i32 = DISPLAY_WIDTH / 2;
// pub const ROWS_HEIGT: i32 = DISPLAY_WIDTH / 3;
// const CHAR_HEIGHT: i32 = 14;
// const CHAR_WIDTH: i32 = 6;

/// This is the principal function that renders all the menu states
pub fn draw_menu<D>(target: &mut D, state: ClockState, msg: Option<&str>) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let logo_image = ImageRawLE::new(include_bytes!("../Images/rust.raw"), 64);
    // normal text
    let normal = MonoTextStyleBuilder::new()
        .font(&FONT_9X15)
        .text_color(BinaryColor::On)
        .build();
    // text with background
    // let background = MonoTextStyleBuilder::from(&normal)
    //     .background_color(BinaryColor::On)
    //     .text_color(BinaryColor::Off)
    //     .build();

    match (state, msg) {
        (ClockState::Time, Some(time)) => {
            Text::new(time, Point::new(0, 13), normal).draw(target)?;
        }
        (ClockState::Time, None) => {}
        (ClockState::Alarm, _) => {
            Text::new("--- Alarm ---", Point::new(0, 13), normal).draw(target)?;
        }
        (ClockState::Image, _) => {
            Image::new(&logo_image, Point::new(32, 0)).draw(target)?;
            // Text::new(message, Point::new(0, 13), normal).draw(target)?;
        } // MenuState::Clock => {}
    }
    Ok(())
}

//-------------------------------------------------------------------------
//                        finite state machine for the menu
//-------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub enum Msg {
    Up,       // Up button
    Down,     // Down button
    Continue, // Continue in the actual state
}

type BackgroundFlag = bool;

#[derive(Copy, Clone)]
pub enum ClockState {
    Time,
    Alarm,
    Image,
}

// TODO(elsuizo: 2023-04-10): what is this???
impl ClockState {
    // fn is_row(&self) -> bool {
    //     matches!(self, Self::Time | Self::Alarm)
    // }
}

#[derive(Copy, Clone)]
pub struct ClockFSM {
    pub state: ClockState,
}

impl ClockFSM {
    pub fn init(state: ClockState) -> Self {
        Self { state }
    }

    pub fn next_state(&mut self, msg: Msg) {
        use ClockState::*;
        use Msg::*;

        self.state = match (self.state, msg) {
            (Time, Up) => Alarm,
            (Time, Continue) => Time,
            (Alarm, Down) => Time,
            (Alarm, Up) => Image,
            (Alarm, Continue) => Alarm,
            (Time, Down) => Image,
            (Image, _) => Time,
            // (Image, Continue) => Image,
        }
    }
}
