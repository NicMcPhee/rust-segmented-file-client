use std::collections::HashMap;

use crate::packets::{Packet, Data, Header};

#[derive(Default)]
pub struct PacketGroup {
    file_name: Option<String>,
    expected_number_of_packets: Option<usize>,
    packets: HashMap<u16, Vec<u8>>
}

impl PacketGroup {
    pub fn received_all_packets(&self) -> bool {
        self.expected_number_of_packets == Some(self.packets.len())
        // self.expected_number_of_packets.map(|expected_number| {
        //     expected_number == self.packets.len()
        // }).unwrap_or(false)
    }

    pub fn process_packet(&mut self, packet: Packet) {
        match packet {
            Packet::Header(header) => self.process_header_packet(header),
            Packet::Data(data) => self.process_data_packet(data)
        }
    }

    fn process_header_packet(&mut self, header: Header) {
        self.file_name = Some(header.file_name);
    }

    fn process_data_packet(&mut self, data: Data) {
        self.packets.insert(data.packet_number, data.data);
        if data.is_last_packet {
            self.expected_number_of_packets = Some((data.packet_number+1) as usize)
        }
    }
}
