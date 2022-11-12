use std::{collections::HashMap, fs::File, io::Write};

use crate::packets::{Packet, Data, Header};

use anyhow::{anyhow, Context};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct PacketGroup {
    pub(crate) file_name: Option<String>,
    pub(crate) expected_number_of_packets: Option<usize>,
    pub(crate) packets: HashMap<u16, Vec<u8>>
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
            Packet::Data(data) => self.process_data_packet(data)
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
    pub fn write_file(&self) -> anyhow::Result<()> {
        let file_name = self.file_name
            .as_ref()
            .ok_or_else(|| anyhow!("Missing file name in packet group"))?;
        let mut file = File::create(file_name)
            .with_context(|| format!("Failed to create file \"{}\"", file_name))?;
        let number_of_packets = self.expected_number_of_packets
            .ok_or_else(|| anyhow!("Didn't know the number of packets for packet group \"{}\"; was the last packet received?", file_name))?;
        assert!(number_of_packets <= u16::MAX as usize + 1, "The number of packets is 1 more than the largest u16 packet number and should be <= u16::MAX");
        // The .expect() shouldn't ever happen here because of the previous assertion.
        #[allow(clippy::expect_used)]
        let max_packet_number: u16 = u16::try_from(number_of_packets - 1).expect("The maximum packet number should fit in a u16");
        for packet_number in 0..=max_packet_number {
            let packet = self.packets
                .get(&packet_number)
                .with_context(|| format!("Didn't find an expected packet with number {} for file \"{}\"", packet_number, file_name))?;
            file.write_all(packet)
                .with_context(|| format!("Failed to write buffer to file \"{}\"", file_name))?;
        }
        Ok(())
    }
}
