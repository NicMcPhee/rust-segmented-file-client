use std::io::{self, Write};

use tokio::net::UdpSocket;

use anyhow::{Result, Context};

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
async fn main() -> Result<()> {
    let sock 
        = UdpSocket::bind("0.0.0.0:7077")
            .await
            .context(format!("Failed to bind to 0.0.0.0:7077"))?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr)
        .await
        .context(format!("Failed to connect to remove address {remote_addr:?}"))?;

    let mut buf = [0; 1028];
    let _ 
        = sock.send(&buf[..1028])
            .await
            .context(format!("Failed to send initial connect message"))?;

    let mut file_manager = FileManager::default();

    while !file_manager.received_all_packets() {
        let len 
            = sock.recv(&mut buf)
                .await
                .context(format!("Error receiving UDP packet"))?;
        let packet: Packet
            = buf[..len]
                .try_into()
                // TODO: Remove this context since it's not helpful?
                .context(format!("Failed to convert data to Packet"))?;
        print!(".");
        io::stdout().flush()?;
        file_manager.process_packet(packet);
    }

    file_manager.write_all_files()?;

    Ok(())
}
