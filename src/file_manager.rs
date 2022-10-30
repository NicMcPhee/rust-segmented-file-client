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

#[cfg(test)]
mod process_packet_tests {
    use crate::{packets::{Header, Packet, Data}, file_manager::FileManager};

    #[test]
    fn just_header_packet() {
        let test_file_name = "test_file.txt".to_string();
        let header = Header {
            file_id: 37,
            file_name: test_file_name.clone(),
        };

        let mut file_manager = FileManager::default();
        file_manager.process_packet(Packet::Header(header));

        let map = file_manager.map;
        assert_eq!(map.len(), 1);
        let file_number = map.keys().nth(0).unwrap();
        let packet_group = map.get(file_number).unwrap();
        assert_eq!(test_file_name, packet_group.file_name.clone().unwrap());
        assert_eq!(None, packet_group.expected_number_of_packets);
        assert_eq!(0, packet_group.packets.len());
    }

    #[test]
    fn just_data_packet_not_last() {
        let file_id = 37;
        let packet_number = 82;
        let is_last_packet = false;
        let bytes: Vec<u8> = vec![5, 8, 9];
        let data_packet = Data {
            file_id,
            packet_number,
            is_last_packet,
            data: bytes.clone(),
        };

        let mut file_manager = FileManager::default();
        file_manager.process_packet(Packet::Data(data_packet));

        let map = file_manager.map;
        assert_eq!(map.len(), 1);
        let file_number = map.keys().nth(0).unwrap();
        let packet_group = map.get(file_number).unwrap();
        assert_eq!(None, packet_group.file_name);
        assert_eq!(None, packet_group.expected_number_of_packets);
        assert_eq!(1, packet_group.packets.len());
        assert_eq!(bytes, packet_group.packets[&packet_number]);
    }

    #[test]
    fn just_data_packet_is_last() {
        let file_id = 37;
        let packet_number = 82;
        let is_last_packet = true;
        let bytes: Vec<u8> = vec![5, 8, 9];
        let data_packet = Data {
            file_id,
            packet_number,
            is_last_packet,
            data: bytes.clone(),
        };

        let mut file_manager = FileManager::default();
        file_manager.process_packet(Packet::Data(data_packet));

        let map = file_manager.map;
        assert_eq!(map.len(), 1);
        let file_number = map.keys().nth(0).unwrap();
        let packet_group = map.get(file_number).unwrap();
        assert_eq!(None, packet_group.file_name);
        assert_eq!(Some(1 + packet_number as usize), packet_group.expected_number_of_packets);
        assert_eq!(1, packet_group.packets.len());
        assert_eq!(bytes, packet_group.packets[&packet_number]);
    }
}
