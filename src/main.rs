use tokio::net::UdpSocket;
use std::io;

use rust_segmented_file_client::packets::{Packet, PacketParseError};

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:7077").await?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr).await?;
    let mut buf = [0; 1028];

    let _ = sock.send(&buf[..1028]).await?;
    loop {
        let len = sock.recv(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, remote_addr);

        let packet: Result<Packet, PacketParseError> = buf[..len].try_into();
        println!("Parsed packet is {:?}", packet);
    }
}
