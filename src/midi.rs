#[derive(Debug)]
pub enum MidiEventPacket {
    NoteOn(MidiChannel, NoteNumber, NoteVelocity),
    NoteOff(MidiChannel, NoteNumber, NoteVelocity),
}

pub type MidiChannel = u8;
pub type NoteVelocity = u8;
pub type NoteNumber = u8;

impl MidiEventPacket {
    pub fn _from_midi_bytes(bytes: [u8; 3]) -> Option<Self> {
        let status = bytes[0];
        let channel = status & 0x0F;
        let note = bytes[1];
        let velocity = bytes[2];
        match status & 0xF0 {
            0x90 => Some(MidiEventPacket::NoteOn(channel, note, velocity)),
            0x80 => Some(MidiEventPacket::NoteOff(channel, note, velocity)),
            _ => None,
        }
    }

    pub fn to_midi_bytes(&self) -> [u8; 3] {
        match self {
            MidiEventPacket::NoteOn(channel, note, velocity) => [0x90 | channel, *note, *velocity],
            MidiEventPacket::NoteOff(channel, note, velocity) => [0x80 | channel, *note, *velocity],
        }
    }

    pub fn to_usb_bytes(&self) -> [u8; 4] {
        let mut result = [0; 4];
        let midi_bytes = self.to_midi_bytes();

        match self {
            MidiEventPacket::NoteOn(_, _, _) => {
                result[0] = 0x09;
            }
            MidiEventPacket::NoteOff(_, _, _) => {
                result[0] = 0x08;
            }
        }
        result[1..4].copy_from_slice(&midi_bytes);
        result
    }
}
