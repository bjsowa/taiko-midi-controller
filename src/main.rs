#![no_main]
#![no_std]

mod midi;
mod xinput;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::time::Hertz;
use embassy_time::{Duration, Timer};
use embassy_usb::driver::EndpointIn;

use {defmt_rtt as _, panic_probe as _};

embassy_stm32::bind_interrupts!(
    struct Irqs {
        USB_LP_CAN1_RX0 => embassy_stm32::usb::InterruptHandler<embassy_stm32::peripherals::USB>;
    }
);

const USB_VID: u16 = 0x045E;
const USB_PID: u16 = 0x028E;
const USB_MANUFACTURER: &str = "bjsowa";
const USB_PRODUCT: &str = "taiko-midi-controller";
const USB_SERIAL_NUMBER: &str = "1";

const CONFIG_DESCRIPTOR_SIZE: usize = 256;
const BOS_DESCRIPTOR_SIZE: usize = 256;
const CONTROL_BUF_SIZE: usize = 256;

const MIDI_MESSAGE_CHANNEL_SIZE: usize = 100;

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

fn lcg(seed: &mut u32) -> u32 {
    const A: u32 = 1664525;
    const C: u32 = 1013904223;
    *seed = A.wrapping_mul(*seed).wrapping_add(C);
    *seed
}

fn random_note_number(seed: &mut u32) -> u8 {
    let random_value = lcg(seed);
    (random_value % 21 + 40) as u8 // Generates a number between 40 and 60
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut stm32_config = embassy_stm32::Config::default();
    set_clocks(&mut stm32_config);

    let p = embassy_stm32::init(stm32_config);

    let driver = embassy_stm32::usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    let mut usb_config = embassy_usb::Config::new(USB_VID, USB_PID);
    usb_config.manufacturer = Some(USB_MANUFACTURER);
    usb_config.product = Some(USB_PRODUCT);
    usb_config.serial_number = Some(USB_SERIAL_NUMBER);

    let mut config_descriptor = [0; CONFIG_DESCRIPTOR_SIZE];
    let mut bos_descriptor = [0; BOS_DESCRIPTOR_SIZE];
    let mut control_buf = [0; CONTROL_BUF_SIZE];

    let xinput_state = xinput::XInputState {};

    let mut builder = embassy_usb::Builder::new(
        driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let xinput_class = xinput::XInputClass::new(&mut builder, &xinput_state);

    let midi_class = embassy_usb::class::midi::MidiClass::new(&mut builder, 1, 1, 64);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let midi_message_channel = embassy_sync::channel::Channel::<
        embassy_sync::blocking_mutex::raw::NoopRawMutex,
        midi::MidiEventPacket,
        MIDI_MESSAGE_CHANNEL_SIZE,
    >::new();

    let (mut midi_sender, mut _midi_receiver) = midi_class.split();

    let write_fut = async {
        loop {
            let packet = midi_message_channel.receive().await;
            let write_res = midi_sender.write_packet(&packet.to_usb_bytes()).await;
            match write_res {
                Ok(_) => info!("write_packet ok"),
                Err(err) => info!("write_packet err {}", err),
            }
        }
    };

    let mut seed: u32 = 12345; // You can use any seed value

    let push_fut = async {
        loop {
            let note_number = random_note_number(&mut seed);
            let note_on = midi::MidiEventPacket::NoteOn(0, note_number, 127);
            let note_off = midi::MidiEventPacket::NoteOff(0, note_number, 0);

            // let ready_fut = midi_receiver.wait_connection();

            // if let core::task::Poll::Pending = embassy_futures::poll_once(ready_fut) {
            // info!("Not connected. Skipping message...");
            // Timer::after(Duration::from_millis(1000)).await;
            // continue;
            // }

            midi_message_channel.send(note_on).await;
            Timer::after(Duration::from_millis(100)).await;
            midi_message_channel.send(note_off).await;
            Timer::after(Duration::from_millis(500)).await;
        }
    };

    embassy_futures::join::join3(write_fut, push_fut, usb_fut).await;
}
