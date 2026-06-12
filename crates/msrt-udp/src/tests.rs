use crate::{UdpClient, UdpClientEvent, UdpServer, UdpServerEvent};
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn client_and_server_exchange_message() {
    let mut server = UdpServer::<4>::bind("127.0.0.1:0").await.unwrap();
    let server_addr = server.local_addr().unwrap();
    let mut client = UdpClient::bind("127.0.0.1:0", server_addr).await.unwrap();

    client.connect().unwrap();
    drive(&mut client, &mut server).await;

    client.send(b"hello udp").unwrap();
    let message = drive_until_server_message(&mut client, &mut server).await;

    assert_eq!(message, b"hello udp");
}

async fn drive(client: &mut UdpClient, server: &mut UdpServer<4>) {
    for _ in 0..32 {
        let _ = client.tick().await.unwrap();
        let _ = server.tick().await.unwrap();
        sleep(Duration::from_millis(1)).await;
    }
}

async fn drive_until_server_message(client: &mut UdpClient, server: &mut UdpServer<4>) -> Vec<u8> {
    for _ in 0..64 {
        let _ = client.tick().await.unwrap();
        match server.tick().await.unwrap() {
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

        match client.tick().await.unwrap() {
            UdpClientEvent::SendFailed(_) => client.disconnect(),
            UdpClientEvent::TransportUnavailable { .. } => client.disconnect(),
            UdpClientEvent::Message(_) | UdpClientEvent::Idle => {}
        }

        sleep(Duration::from_millis(1)).await;
    }

    panic!("message was not delivered");
}
