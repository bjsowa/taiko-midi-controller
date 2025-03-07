#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

embassy_stm32::bind_interrupts!(
    struct Irqs {
        USB_LP_CAN1_RX0 => embassy_stm32::usb::InterruptHandler<embassy_stm32::peripherals::USB>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let driver = embassy_stm32::usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    let config = embassy_usb::Config::new(0xc0de, 0xcafe);

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 7];

    // let mut state = State::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
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

    // loop {
    // midi_class.write_packet(&[0x90, 0x40, 0x7f]).await.unwrap();
    // }

    // info!("Hello World!");

    // let mut led = Output::new(p.PB12, Level::High, Speed::Low);

    // loop {
    //     info!("high");
    //     led.set_high();
    //     Timer::after_millis(300).await;

    //     info!("low");
    //     led.set_low();
    //     Timer::after_millis(300).await;
    // }
}
