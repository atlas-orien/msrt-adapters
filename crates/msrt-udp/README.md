# msrt-udp

std UDP adapters for MSRT.

This crate wraps MSRT endpoints around `std::net::UdpSocket`.

- `UdpClient`: connected UDP client using `ClientEndpoint`
- `UdpServer<N>`: fixed-capacity UDP server using `ServerEndpoint<SocketAddr, N>`

The server accepts unknown peers automatically until its fixed peer table is
full.

## Client

```rust
use msrt_udp::{UdpClient, UdpClientEvent};

# fn run() -> msrt_udp::Result<()> {
let mut client = UdpClient::bind("127.0.0.1:0", "127.0.0.1:9000")?;
client.connect()?;
client.send(b"hello")?;

loop {
    match client.tick()? {
        UdpClientEvent::Message(message) => {
            println!("{:?}", message.as_bytes());
        }
        UdpClientEvent::SendFailed(_) => client.disconnect(),
        UdpClientEvent::Idle => {}
    }
}
# }
```

## Server

```rust
use msrt_udp::{UdpServer, UdpServerEvent};

# fn run() -> msrt_udp::Result<()> {
let mut server = UdpServer::<8>::bind("127.0.0.1:9000")?;

loop {
    match server.tick()? {
        UdpServerEvent::Message { peer, message } => {
            server.send_to(peer, message.as_bytes())?;
        }
        UdpServerEvent::SendFailed { peer, .. } => {
            server.disconnect(peer);
        }
        UdpServerEvent::Idle => {}
    }
}
# }
```
