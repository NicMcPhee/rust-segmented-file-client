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
        for packet_group in self.map.values() {
            packet_group.write_file()?;
        }
        Ok(())
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

#[cfg(test)]
mod quickcheck_tests {
    use std::ops::Deref;

    use crate::packets::{Header, Data};

    use super::*;

    #[quickcheck_macros::quickcheck]
    fn header_doesnt_crash(packet: Header) -> bool {
        let mut file_manager = FileManager::default();
        file_manager.process_packet(Packet::Header(packet));
        true
    }

    #[quickcheck_macros::quickcheck]
    fn data_doesnt_crash(packet: Data) -> bool {
        let mut file_manager = FileManager::default();
        file_manager.process_packet(Packet::Data(packet));
        true
    }

    #[quickcheck_macros::quickcheck]
    fn header_sets_name(packet: Header) -> bool {
        let mut file_manager = FileManager::default();
        assert_eq!(None, file_manager.map.get(&packet.file_id));
        file_manager.process_packet(Packet::Header(packet.clone()));
        assert_eq!(0, file_manager.map.get(&packet.file_id).unwrap().packets.len());
        file_manager.map.get(&packet.file_id).unwrap().file_name == Some(packet.file_name.clone())
    }

    #[quickcheck_macros::quickcheck]
    fn data_add_vec(packet: Data) -> bool {
        let mut file_manager = FileManager::default();
        assert_eq!(None, file_manager.map.get(&packet.file_id));
        file_manager.process_packet(Packet::Data(packet.clone()));

        let group = file_manager.map.get(&packet.file_id).unwrap();
        assert_eq!(None, group.file_name);
        if packet.is_last_packet {
            assert_eq!(Some((packet.packet_number as usize)+1), group.expected_number_of_packets);
        } else {
            assert_eq!(None, group.expected_number_of_packets);
        }
        assert_eq!(1, group.packets.len());
        group.packets.get(&packet.packet_number).unwrap().deref() == packet.data
    }
}

#[cfg(test)]
mod all_packets_tests {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    use crate::packets::{Header, Data, Packet};

    use super::FileManager;

    #[test]
    fn processes_full_packet_set() {
        let mut packets = Vec::new();
        let file_name = "test_file_name".to_string();
        let file_id = 42;
        let num_packets: u16 = 3;
        packets.push(Packet::Header(Header { file_name: file_name.clone(), file_id }));
        for packet_number in 0..num_packets {
            let val: u8 = (packet_number % 100).try_into().unwrap();
            packets.push(Packet::Data(Data { file_id, packet_number, is_last_packet: packet_number == 2, 
                data: vec![val, val+1]}));
        }
        let mut rng = thread_rng();

        let mut file_manager = FileManager::default();
        packets.shuffle(&mut rng);
        for p in packets {
            file_manager.process_packet(p);
        }

        assert_eq!(1, file_manager.map.len());
        let group = file_manager.map.get(&file_id).unwrap();
        assert_eq!(Some(file_name), group.file_name);
        assert_eq!(Some(3), group.expected_number_of_packets);
        for packet_number in 0..num_packets {
            let data = group.packets.get(&packet_number).unwrap();
            assert_eq!(2, data.len());
            let val: u8 = (packet_number % 100).try_into().unwrap();
            assert_eq!(val, data[0]);
            assert_eq!(val+1, data[1]);
        }
    }
}