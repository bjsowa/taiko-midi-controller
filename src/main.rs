// #![no_std]
// #![no_main]

// extern crate panic_semihosting;

// use cortex_m_rt::{entry, exception};

// #[entry]
// fn main() -> ! {
//     // println!("Hello, world!");
//     loop {}
// }

#![no_main]
#![no_std]

// set the panic handler
extern crate panic_semihosting;

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use stm32f1xx_hal::{prelude::*, time::Hertz};

static TOGGLE_LED: AtomicBool = AtomicBool::new(false);

#[entry]
fn main() -> ! {
    let mut core = cortex_m::Peripherals::take().unwrap();
    let device = stm32f1xx_hal::stm32::Peripherals::take().unwrap();
    let rcc = device.RCC.constrain();
    let mut flash = device.FLASH.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(Hertz::MHz(8))
        .sysclk(Hertz::MHz(72))
        .freeze(&mut flash.acr);

    // configure the user led
    let mut gpiob = device.GPIOB.split();
    let mut led = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);

    // configure SysTick to generate an exception every second
    core.SYST.set_clock_source(SystClkSource::Core);
    core.SYST.set_reload(16_000_000); // 1/4 second
    core.SYST.enable_counter();
    core.SYST.enable_interrupt();

    loop {
        // sleep
        cortex_m::asm::wfi();
        if TOGGLE_LED.swap(false, Ordering::AcqRel) {
            led.toggle();
        }
    }
}

#[exception]
fn SysTick() {
    TOGGLE_LED.store(true, Ordering::Release);
}
