use tokio::net::UdpSocket;

use rust_segmented_file_client::{
    packets::{Packet, PacketParseError}, 
    file_manager::FileManager
};

// TODO: Maybe use this as a chance to explore an alternative
//   error handling system like `anyhow`.
#[derive(Debug)]
enum ClientError {
    IoError(std::io::Error),
    PacketParseError(PacketParseError),
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::IoError(e)
    }
}

impl From<PacketParseError> for ClientError {
    fn from(e: PacketParseError) -> Self {
        Self::PacketParseError(e)
    }
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let sock = UdpSocket::bind("0.0.0.0:7077").await?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr).await?;
    let mut buf = [0; 1028];

    let _ = sock.send(&buf[..1028]).await?;

    let mut file_manager = FileManager::default();

    while !file_manager.received_all_packets() {
        let len = sock.recv(&mut buf).await?;
        let packet: Packet = buf[..len].try_into()?;
        println!("Got packet of len {len} for file {}.", packet.file_id());
        file_manager.process_packet(packet);
    }

    println!("Done with the loop!");

    file_manager.write_all_files().await?;

    Ok(())
}
