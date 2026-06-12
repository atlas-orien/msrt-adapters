use crate::{UdpClient, UdpClientEvent, UdpServer, UdpServerEvent};

#[test]
fn client_and_server_exchange_message() {
    let mut server = UdpServer::<4>::bind("127.0.0.1:0").unwrap();
    let server_addr = server.local_addr().unwrap();
    let mut client = UdpClient::bind("127.0.0.1:0", server_addr).unwrap();

    client.connect().unwrap();
    drive(&mut client, &mut server);

    client.send(b"hello udp").unwrap();
    let message = drive_until_server_message(&mut client, &mut server);

    assert_eq!(message, b"hello udp");
}

fn drive(client: &mut UdpClient, server: &mut UdpServer<4>) {
    for _ in 0..32 {
        let _ = client.tick().unwrap();
        let _ = server.tick().unwrap();
    }
}

fn drive_until_server_message(client: &mut UdpClient, server: &mut UdpServer<4>) -> Vec<u8> {
    for _ in 0..64 {
        let _ = client.tick().unwrap();
        match server.tick().unwrap() {
            UdpServerEvent::Message { message, .. } if message.as_bytes() != [0] => {
                return message.as_bytes().to_vec();
            }
            UdpServerEvent::Message { peer, message } => {
                server.send_to(peer, message.as_bytes()).unwrap();
            }
            UdpServerEvent::SendFailed { peer, .. } => {
                server.disconnect(peer);
            }
            UdpServerEvent::Idle => {}
        }

        match client.tick().unwrap() {
            UdpClientEvent::SendFailed(_) => client.disconnect(),
            UdpClientEvent::TransportUnavailable { .. } => client.disconnect(),
            UdpClientEvent::Message(_) | UdpClientEvent::Idle => {}
        }
    }

    panic!("message was not delivered");
}
