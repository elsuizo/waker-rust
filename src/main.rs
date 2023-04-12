//----------------------------------------------------------------------------
// @date 2021-11-13
// @author Martin Noblia
// TODOs
// - [X] Periodic task blinky compile and working
// - [X] include the oled display
// - [X] do the menu with buttons
// - [X] enable UART debug
//  - [X] read the buttons
//  - [X] generate a state machine with the menu states
//----------------------------------------------------------------------------
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

mod buttons;
mod datetime;
mod io;
mod ui;

use crate::buttons::Button;
use crate::io::Logger;
use datetime::DateTime;
use heapless::String;
use panic_semihosting as _;
use rtic::app;
use stm32f1xx_hal::gpio::PinState;
use stm32f1xx_hal::{gpio, pac, prelude::*};

use core::fmt::Write;

use pac::I2C1;
use sh1106::{prelude::*, Builder};
use stm32f1xx_hal::{
    i2c::{BlockingI2c, DutyCycle, Mode},
    rtc::Rtc,
    serial::{Config, Serial},
};
use systick_monotonic::{fugit::Duration, Systick};

#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;
    //-------------------------------------------------------------------------
    //                        type alias
    //-------------------------------------------------------------------------
    type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;
    type Sda = gpio::gpiob::PB9<gpio::Alternate<gpio::OpenDrain>>;
    type Scl = gpio::gpiob::PB8<gpio::Alternate<gpio::OpenDrain>>;
    type ButtonUpPin = gpio::gpioa::PA5<gpio::Input<gpio::PullUp>>;
    type ButtonDownPin = gpio::gpioa::PA6<gpio::Input<gpio::PullUp>>;
    type ButtonEnterPin = gpio::gpioa::PA7<gpio::Input<gpio::PullUp>>;
    type OledDisplay = GraphicsMode<I2cInterface<BlockingI2c<I2C1, (Scl, Sda)>>>;

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;
    //-------------------------------------------------------------------------
    //                        resources declaration
    //-------------------------------------------------------------------------
    // Resources shared between tasks
    #[shared]
    struct Shared {
        led: Led,
    }

    #[local]
    struct Local {
        button_up: Button<ButtonUpPin>,
        button_down: Button<ButtonDownPin>,
        button_enter: Button<ButtonEnterPin>,
        rtc: Rtc,
        display: OledDisplay,
        logger: Logger,
        clock_fsm: crate::ui::ClockFSM,
    }

    //-------------------------------------------------------------------------
    //                        initialization fn
    //-------------------------------------------------------------------------
    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        //-------------------------------------------------------------------------
        //                        hardware initialization
        //-------------------------------------------------------------------------
        let rcc = cx.device.RCC.constrain();
        let mut pwr = cx.device.PWR;
        let mut flash = cx.device.FLASH.constrain();
        // let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(36.MHz())
            .pclk1(36.MHz())
            .freeze(&mut flash.acr);

        // let clocks = rcc
        //     .cfgr
        //     .use_hse(8.MHz())
        //     .sysclk(72.MHz())
        //     .pclk1(36.MHz())
        //     .freeze(&mut flash.acr);
        let mut afio = cx.device.AFIO.constrain();
        let mut backup_domain = rcc.bkp.constrain(cx.device.BKP, &mut pwr);

        let mut gpioa = cx.device.GPIOA.split();
        let mut gpiob = cx.device.GPIOB.split();
        let mut gpioc = cx.device.GPIOC.split();
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        // USART1
        let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let rx = gpiob.pb7;
        let serial = Serial::new(
            cx.device.USART1,
            (tx, rx),
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            &clocks,
        );
        let tx = serial.split().0;
        let logger = Logger::new(tx);
        // oled display pins
        let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
        let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
        let i2c = BlockingI2c::i2c1(
            cx.device.I2C1,
            (scl, sda),
            &mut afio.mapr,
            Mode::Fast {
                frequency: 100.kHz(),
                duty_cycle: DutyCycle::Ratio2to1,
            },
            clocks,
            1000,
            10,
            1000,
            1000,
        );

        //-------------------------------------------------------------------------
        //                        rtic initialization
        //-------------------------------------------------------------------------
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
        display.init().ok();
        display.flush().ok();
        let systick = cx.core.SYST;
        // let mono = Systick::new(systick, 36_000_000);
        let mono = Systick::new(systick, 8_000_000);

        let button_up_pin = gpioa.pa5.into_pull_up_input(&mut gpioa.crl);
        let button_down_pin = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        let button_enter_pin = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
        let mut rtc = Rtc::new(cx.device.RTC, &mut backup_domain);
        let today = DateTime {
            year: 2023,
            month: 4,
            day: 11,
            hour: 22,
            min: 18,
            sec: 00,
            day_of_week: datetime::DayOfWeek::Saturday,
        };
        if let Some(epoch) = today.to_epoch() {
            rtc.set_time(epoch);
        }

        rtc.listen_seconds();

        // NOTE(elsuizo:2021-11-24): here we dont need a super fast spawn(for the inititlization...)!!!
        // NOTE(elsuizo: 2023-04-11): this is one second
        react::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();

        (
            Shared { led },
            Local {
                button_up: Button::new(button_up_pin),
                button_down: Button::new(button_down_pin),
                button_enter: Button::new(button_enter_pin),
                rtc,
                display,
                logger,
                clock_fsm: crate::ui::ClockFSM::init(crate::ui::ClockState::Time),
            },
            init::Monotonics(mono),
        )
    }

    //-------------------------------------------------------------------------
    //                        tasks
    //-------------------------------------------------------------------------
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }
    // NOTE(elsuizo:2021-11-24): the maximum period of this periodic task for a responsive button
    // action is 13 ms
    // NOTE(elsuizo:2021-11-21): remember that the method set_low() needs the trait: `use embedded_hal::digital::v2::OutputPin;`
    // to be used!!!
    #[task(local = [button_up, button_down, button_enter], shared = [led])]
    fn react(cx: react::Context) {
        use crate::buttons::PinState::*;
        use crate::ui::ClockState::*;
        use crate::ui::Msg::*;

        if let PinUp = cx.local.button_up.poll() {
            dispatch_msg::spawn(Up).ok();
        }
        if let PinUp = cx.local.button_down.poll() {
            dispatch_msg::spawn(Down).ok();
        }

        if let Nothing = cx.local.button_up.poll() {
            dispatch_msg::spawn(Continue).ok();
        }

        if let Nothing = cx.local.button_down.poll() {
            dispatch_msg::spawn(Continue).ok();
        }

        // if let PinUp = cx.local.button_enter.poll() {
        //     dispatch_msg::spawn(Enter).ok();
        // }
        react::spawn_after(Duration::<u64, 1, 1000>::from_ticks(10)).unwrap();
    }

    #[task(local = [display, logger, clock_fsm, rtc], shared = [led])]
    fn dispatch_msg(cx: dispatch_msg::Context, msg: crate::ui::Msg) {
        use crate::ui::Msg::*;
        let dispatch_msg::SharedResources { mut led } = cx.shared;
        cx.local.display.clear();
        cx.local.clock_fsm.next_state(msg);
        match msg {
            Up => {
                led.lock(|l| l.toggle());
                let mut time: String<256> = String::new();
                let datetime = DateTime::new(cx.local.rtc.current_time());
                write!(&mut time, "{}", datetime).unwrap();
                cx.local.logger.log("button Up pressed!!!").ok();
                crate::ui::draw_menu(cx.local.display, cx.local.clock_fsm.state, Some(&time)).ok();
                cx.local.display.flush().unwrap();
            }
            Down => {
                led.lock(|l| l.toggle());
                cx.local.logger.log("button Down pressed!!!").ok();
                let mut time: String<256> = String::new();
                let datetime = DateTime::new(cx.local.rtc.current_time());
                write!(&mut time, "{}", datetime).unwrap();
                crate::ui::draw_menu(cx.local.display, cx.local.clock_fsm.state, Some(&time)).ok();
                cx.local.display.flush().unwrap();
            }
            Continue => {
                // led.lock(|l| l.toggle());
                let mut time: String<256> = String::new();
                let datetime = DateTime::new(cx.local.rtc.current_time());
                write!(&mut time, "{}", datetime).unwrap();
                crate::ui::draw_menu(cx.local.display, cx.local.clock_fsm.state, Some(&time)).ok();
                cx.local.display.flush().unwrap();
            }
        };
    }
}
