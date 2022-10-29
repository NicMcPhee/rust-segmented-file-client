use std::{io, collections::HashMap};

use crate::{packets::Packet, packet_group::PacketGroup};

#[derive(Default)]
pub struct FileManager {
    // The key will be a file ID, and the value will
    // be the associated PacketGroup.
    map: HashMap<u8, PacketGroup>
}

impl FileManager {
    pub fn received_all_packets(&self) -> bool {
        // We have to check that we've seen all three files,
        // and that each file is "done", i.e., we've received all
        // of the packets for that file.
        self.map.len() == 3 
            && self.map.values().all(|pg| pg.received_all_packets())
    }

    fn packet_group_for_file_id(&mut self, file_id: u8) -> &mut PacketGroup {
        self.map.entry(file_id).or_default()
    }

    pub fn process_packet(&mut self, packet: Packet) {
        self.packet_group_for_file_id(packet.file_id())
            .process_packet(packet);
    }

    pub async fn write_all_files(&self) -> io::Result<()> {
        todo!()
    }
}
