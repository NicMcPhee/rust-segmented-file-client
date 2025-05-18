use std::{
    collections::HashMap,
    ffi::OsString,
    fs::File,
    io::{self, Write},
};

use crate::packets::{Data, Header, Packet};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct PacketGroup {
    pub(crate) file_name: Option<OsString>,
    pub(crate) expected_number_of_packets: Option<usize>,
    pub(crate) packets: HashMap<u16, Vec<u8>>,
}

impl PacketGroup {
    #[must_use]
    pub fn received_all_packets(&self) -> bool {
        self.expected_number_of_packets == Some(self.packets.len())
        // self.expected_number_of_packets.map(|expected_number| {
        //     expected_number == self.packets.len()
        // }).unwrap_or(false)
    }

    pub fn process_packet(&mut self, packet: Packet) {
        match packet {
            Packet::Header(header) => self.process_header_packet(header),
            Packet::Data(data) => self.process_data_packet(data),
        }
    }

    fn process_header_packet(&mut self, header: Header) {
        self.file_name = Some(header.file_name);
    }

    fn process_data_packet(&mut self, data: Data) {
        self.packets.insert(data.packet_number, data.data);
        if data.is_last_packet {
            self.expected_number_of_packets = Some((data.packet_number as usize) + 1);
        }
    }

    /// # Panics
    ///
    /// Will panic if any of the following is true:
    ///   * The file name hasn't been set
    ///   * The expected number of packets hasn't set
    ///   * A packet number was too big to fit in a u16, i.e., the expected
    ///     number of packets was too large
    ///   * There's a missing packet in the `packets` map
    ///
    /// # Errors
    ///
    /// Will return an error if either:
    ///   * We couldn't open the file
    ///   * There was an error writing to the file
    pub fn write_file(&self) -> io::Result<()> {
        let mut file = File::create(self.file_name.as_ref().unwrap())?;
        for packet_number in 0..self.expected_number_of_packets.unwrap() {
            let packet_number: u16 =
                u16::try_from(packet_number).expect("The packet number should fit in a u16");
            let packet = self
                .packets
                .get(&packet_number)
                .expect("Didn't find an expected packet");
            file.write_all(packet)?;
        }
        Ok(())
    }
}
