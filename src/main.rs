use std::io::{self, Write};

use anyhow::Context;

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
async fn main() -> anyhow::Result<()> {
    let addr = "0.0.0.0:7077";
    let sock = UdpSocket::bind(addr).await
        .with_context(|| format!("Failed to bind to address {}", addr))?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr).await
        .with_context(|| format!("Failed to connect to address {}", remote_addr))?;
    let mut buf = [0; 1028];

    let _ = sock.send(&buf[..1028]).await
        .with_context(|| format!("Failed to send the initial connection message to address {}", remote_addr))?;

    let mut file_manager = FileManager::default();

    while !file_manager.received_all_packets() {
        let len = sock.recv(&mut buf).await
            .with_context(|| format!("Failed to receive a packet from remote address {}; is the server running?", remote_addr))?;
        let packet: Packet = buf[..len].try_into()
            .with_context(|| format!("Failed to parse packet {:?}", &buf[..len]))?;
        print!(".");
        io::stdout().flush()
            .context("Failed to flush stdout")?;
        file_manager.process_packet(packet);
    }

    file_manager.write_all_files()
        .context("Failed to write files to disk")?;

    Ok(())
}
