# msrt-udp

Tokio UDP adapters for MSRT.

This crate wraps MSRT endpoints around `tokio::net::UdpSocket`.

- `UdpClient`: connected UDP client using `ClientEndpoint`
- `UdpServer<N>`: fixed-capacity UDP server using `ServerEndpoint<SocketAddr, N>`

The server accepts unknown peers automatically until its fixed peer table is
full.

Connected UDP sockets may surface recoverable transport feedback when a remote
peer disappears or restarts. `UdpClient::tick` reports these conditions as
`UdpClientEvent::TransportUnavailable` so clients can disconnect, back off, and
start a fresh MSRT session instead of treating the condition as fatal.

## Client

```rust
use msrt_udp::{UdpClient, UdpClientEvent};

# async fn run() -> msrt_udp::Result<()> {
let mut client = UdpClient::bind("127.0.0.1:0", "127.0.0.1:9000").await?;
client.connect()?;
client.send(b"hello")?;

loop {
    match client.tick().await? {
        UdpClientEvent::Message(message) => {
            println!("{:?}", message.as_bytes());
        }
        UdpClientEvent::SendFailed(_) => client.disconnect(),
        UdpClientEvent::TransportUnavailable { .. } => {
            client.disconnect();
            client.connect()?;
        }
        UdpClientEvent::Idle => {}
    }
}
# }
```

## Server

```rust
use msrt_udp::{UdpServer, UdpServerEvent};

# async fn run() -> msrt_udp::Result<()> {
let mut server = UdpServer::<8>::bind("127.0.0.1:9000").await?;

loop {
    match server.tick().await? {
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

## Examples

Run the server:

```sh
cargo run -p msrt-udp --example server -- 127.0.0.1:9000
```

Run the frontend in another terminal:

```sh
cargo run -p msrt-udp --example frontend -- 127.0.0.1:9000
```

To test reconnect behavior, stop the server while the frontend is running. The
frontend will report either `TransportUnavailable` from UDP socket feedback or
`SendFailed` from MSRT retry exhaustion. Start the server again and the frontend
will create a fresh MSRT session and continue sending.
