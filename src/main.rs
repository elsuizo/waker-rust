#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;
use stm32f1xx_hal::{gpio, pac, prelude::*};
use rtic::app;

type LED = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;

// NOTE(elsuizo:2020-10-07): habia que poner el pac dentro de esto
#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        led: LED
    }

    #[init]
    fn init(mut cx: init::Context) -> init::LateResources {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc   = cx.device.RCC.constrain();
        let mut afio  = cx.device.AFIO.constrain(&mut rcc.apb2);
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        init::LateResources {
            led
        }
    }
};
