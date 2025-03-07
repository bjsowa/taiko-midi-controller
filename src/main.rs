#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::time::Hertz;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

embassy_stm32::bind_interrupts!(
    struct Irqs {
        USB_LP_CAN1_RX0 => embassy_stm32::usb::InterruptHandler<embassy_stm32::peripherals::USB>;
    }
);

fn set_clocks(config: &mut embassy_stm32::Config) {
    use embassy_stm32::rcc::*;
    config.rcc.hse = Some(Hse {
        freq: Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll = Some(Pll {
        src: PllSource::HSE,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL9,
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut stm32_config = embassy_stm32::Config::default();
    set_clocks(&mut stm32_config);

    let p = embassy_stm32::init(stm32_config);

    let driver = embassy_stm32::usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    let usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);

    // config.rcc.hse = todo!();

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 7];

    // let mut state = State::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let mut midi_class = embassy_usb::class::midi::MidiClass::new(&mut builder, 0, 1, 64);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let echo_fut = async {
        loop {
            let write_res = midi_class.write_packet(&[0x90, 0x40, 0x7f]).await;
            match write_res {
                Ok(_) => info!("write_packet ok"),
                Err(err) => info!("write_packet err {}", err),
            }
            Timer::after_millis(1000).await;
        }
    };

    embassy_futures::join::join(usb_fut, echo_fut).await;
}
