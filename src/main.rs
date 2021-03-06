//----------------------------------------------------------------------------
// @date 2021-07-08
// @author Martin Noblia
// TODOs
// - [X] verificar que ande la hora bien
// - [X] verificar que el oled funciona bien y mostrar la hora
// - [ ] hacer el menu con el display
//  - [ ] que cambie la hora con los botones
//----------------------------------------------------------------------------
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

mod datetime;
mod logger;
use datetime::DateTime;
extern crate panic_semihosting;
// use core::fmt::Write;
use cortex_m::peripheral::DWT;
// TODO(elsuizo:2021-03-25): this is a mess
use stm32f1xx_hal::{
    gpio,
    i2c::{BlockingI2c, DutyCycle, I2c, Mode},
    pac,
    prelude::*,
    rtc::Rtc,
    serial::{self, Config, Serial},
    stm32,
    time::Hertz,
};

use stm32f1xx_hal::gpio::{Output, State};

use core::fmt::Write;
use embedded_graphics::{
    fonts::{Font12x16, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyle,
};
use embedded_hal::digital::v2::OutputPin;
use heapless::{consts::*, String};
use pac::I2C1;
use rtic::app;
use rtic::cyccnt::{Instant, U32Ext};
use sh1106::interface::{DisplayInterface, I2cInterface};
use sh1106::mode::displaymode::DisplayModeTrait;
use sh1106::{prelude::*, Builder};

type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;
type Sda = gpio::gpiob::PB9<gpio::Alternate<gpio::OpenDrain>>;
type Scl = gpio::gpiob::PB8<gpio::Alternate<gpio::OpenDrain>>;

type OledDisplay = GraphicsMode<I2cInterface<BlockingI2c<I2C1, (Scl, Sda)>>>;

// type OledDisplay = I2cInterface<BlockingI2c<I2C1, (PB8<Alternate<OpenDrain>>, PB9<Alternate<OpenDrain>>)>>;

const PERIOD: u32 = 8_000_000; // period of periodic task execution

#[app(device = stm32f1xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        led: Led,
        display: OledDisplay,
        rtc: Rtc,
        logger: logger::Logger,
    }

    #[init(schedule=[rtc_test])]
    fn init(mut cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();
        // cx.core.DCB.enable_trace();
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);
        let mut pwr = cx.device.PWR;
        let mut backup_domain = rcc.bkp.constrain(cx.device.BKP, &mut rcc.apb1, &mut pwr);
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        // oled pins
        let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
        let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
        // USART1
        let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let rx = gpiob.pb7;
        let mut out: String<U256> = String::new();
        // Set up the usart device. Taks ownership over the USART register and tx/rx pins. The rest of
        // the registers are used to enable and configure the device.
        let mut serial = Serial::usart1(
            cx.device.USART1,
            (tx, rx),
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            clocks,
            &mut rcc.apb2,
        );
        let mut led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::Low);
        led.set_high().unwrap();

        let i2c = BlockingI2c::i2c1(
            cx.device.I2C1,
            (scl, sda),
            &mut afio.mapr,
            Mode::Fast {
                frequency: 400_000.hz(),
                duty_cycle: DutyCycle::Ratio2to1,
            },
            clocks,
            &mut rcc.apb1,
            1000,
            10,
            1000,
            1000,
        );

        // real time clock initialization
        let mut rtc = Rtc::rtc(cx.device.RTC, &mut backup_domain);
        let today = DateTime {
            year: 2021,
            month: 4,
            day: 17,
            hour: 17,
            min: 24,
            sec: 00,
            day_of_week: datetime::DayOfWeek::Saturday,
        };
        if let Some(epoch) = today.to_epoch() {
            rtc.set_time(epoch);
        }

        rtc.listen_seconds();
        //
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
        display.init().unwrap();
        display.flush().unwrap();

        let tx = serial.split().0;
        let mut logger = logger::Logger::new(tx);
        cx.schedule.rtc_test(cx.start + PERIOD.cycles()).unwrap();

        // resources
        init::LateResources {
            led,
            display,
            rtc,
            logger,
        }
    }

    // #[task(resources=[display])]
    // fn show_text(c: show_text::Context) {
    //     Text::new("Hello world!", Point::zero())
    //         .into_styled(TextStyle::new(Font6x8, BinaryColor::On))
    //         .draw(&mut c.resources.display)
    //         .unwrap();
    // }

    #[task(resources=[led, logger, rtc, display], schedule=[rtc_test])]
    fn rtc_test(c: rtc_test::Context) {
        static mut LED_STATE: bool = false;
        // c.resources.rtc.clear_second_flag();
        // c.resources.rtc.listen_seconds();

        // if *LED_STATE {
        //     c.resources.led.set_high().unwrap();
        //     *LED_STATE = false;
        // } else {
        //     c.resources.led.set_low().unwrap();
        //     *LED_STATE = true;
        // }
        let mut out: String<U256> = String::new();
        let datetime = DateTime::new(c.resources.rtc.current_time());
        write!(&mut out, "{}", datetime).unwrap();
        Text::new(&out, Point::new(0, 4))
            .into_styled(TextStyle::new(Font12x16, BinaryColor::On))
            .draw(c.resources.display)
            .unwrap();
        c.resources.display.flush().unwrap();
        c.resources.display.clear();
        c.resources.logger.log(&out);

        c.schedule.rtc_test(c.scheduled + PERIOD.cycles()).unwrap();
    }
    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn EXTI2();
    }
};
