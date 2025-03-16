use embassy_usb::driver::Driver;

const XINPUT_IFACE_CLASS_VENDOR: u8 = 0xFF; // Vendor specific
const XINPUT_IFACE_SUBCLASS: u8 = 0x5D; // XInput
const XINPUT_IFACE_PROTOCOL_CONTROL: u8 = 0x01; // Protocol for control interface

const XINPUT_DESC_DESCTYPE: u8 = 0x21; // a common descriptor type for all xinput interfaces

const XINPUT_DESC_CONTROL: &[u8] = &[
    // for control interface
    0x00, 0x01, 0x01, 0x25, // ???
    0x81, // bEndpointAddress (IN, 1)
    0x14, // bMaxDataSize
    0x00, 0x00, 0x00, 0x00, 0x13, // ???
    0x01, // bEndpointAddress (OUT, 1)
    0x08, // bMaxDataSize
    0x00, 0x00, // ???
];

const XINPUT_EP_MAX_PACKET_SIZE: u16 = 0x20;
const XINPUT_RW_BUFFER_SIZE: usize = XINPUT_EP_MAX_PACKET_SIZE as usize;

#[derive(Debug)]
pub struct XInputState {}

#[derive(Debug)]
pub struct XInputClass<'d, D: Driver<'d>> {
    ep_in: D::EndpointIn,
    ep_out: D::EndpointOut,
    state: &'d XInputState,
}

impl<'d, D: Driver<'d>> XInputClass<'d, D> {
    pub fn new(builder: &mut embassy_usb::Builder<'d, D>, state: &'d XInputState) -> Self {
        let mut xinput_function = builder.function(
            XINPUT_IFACE_CLASS_VENDOR,
            XINPUT_IFACE_SUBCLASS,
            XINPUT_IFACE_PROTOCOL_CONTROL,
        );
        let mut control_interface = xinput_function.interface();
        let mut alt_control = control_interface.alt_setting(
            XINPUT_IFACE_CLASS_VENDOR,
            XINPUT_IFACE_SUBCLASS,
            XINPUT_IFACE_PROTOCOL_CONTROL,
            None,
        );

        alt_control.descriptor(XINPUT_DESC_DESCTYPE, XINPUT_DESC_CONTROL);

        let ep_in = alt_control.endpoint_interrupt_in(XINPUT_EP_MAX_PACKET_SIZE, 4);
        let ep_out = alt_control.endpoint_interrupt_out(XINPUT_EP_MAX_PACKET_SIZE, 8);

        XInputClass {
            ep_in,
            ep_out,
            state: state,
        }
    }

    // pub async run(mut self) -> ! {
    // loop {
    //     let mut buf = [0; 32];
    //     let len = self.ep_out.read(&mut buf).await;
    //     if len > 0 {
    //         info!("Received {} bytes", len);
    //         info!("Data: {:?}", &buf[..len]);
    //     }
    // }
    // }
}
