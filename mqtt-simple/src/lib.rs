#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bigendian_conversion() {
        let expected = vec![0, 11];
        assert_eq!(Protocol::to_big_endian(11), expected);
    }
    #[test]
    fn test_connect_payload() {
        let expected = vec![
            16, 23, 0, 4, 77, 81, 84, 84, 4, 2, 0, 5, 0, 11, 99, 108, 105, 101, 110, 116, 95, 110,
            97, 109, 101,
        ];
        assert_eq!(Protocol::connect_payload("client_name", 5), expected);
    }

    #[test]
    fn test_publish_payload() {
        let expected = vec![
            48, 22, 0, 10, 115, 111, 109, 101, 95, 116, 111, 112, 105, 99, 109, 121, 32, 109, 101,
            115, 115, 97, 103, 101,
        ];
        assert_eq!(
            Protocol::publish_payload("some_topic", "my message", false, QoS::AtMostOnce, 0),
            expected
        );
    }
}

use std::io::prelude::*;
use std::io::{self, Read};
use std::net::TcpStream;

pub struct Client {
    name: String,
    server: std::net::SocketAddr,
}
pub struct ConnectedClient {
    socket: TcpStream,
    pid: u16,
    keepalive: u8,
}
struct Protocol {}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl Client {
    pub fn new(name: String, server: String) -> Result<Client, std::net::AddrParseError> {
        Ok(Client {
            name,
            server: format!("{}:1883", server).parse()?,
        })
    }

    pub fn connect(
        &mut self,
        keepalive: u8,
    ) -> Result<ConnectedClient, Box<dyn std::error::Error>> {
        let payload = Protocol::connect_payload(self.name.as_ref(), keepalive);
        let mut stream =
            TcpStream::connect_timeout(&self.server, std::time::Duration::from_secs(3))?;
        stream.write(payload.as_ref())?;

        let mut buf = vec![0 as u8; 4];
        stream.read_exact(&mut buf)?;
        assert!(buf[0] == 0x20);
        assert!(buf[1] == 0x02);
        assert!(buf[3] == 0x00);

        Ok(ConnectedClient {
            socket: stream,
            keepalive,
            pid: 0,
        })
    }
}
impl ConnectedClient {
    pub fn publish(
        &mut self,
        topic: &str,
        msg: &str,
        retain: bool,
        qos: QoS,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = Protocol::publish_payload(topic, msg, retain, qos, self.pid);
        self.socket.write(payload.as_ref())?;
        self.pid += 1;

        self.drain_ping();
        Ok(())
    }

    fn drain_ping(&mut self) {
        let mut buf: Vec<u8> = Vec::new();
        self.socket.set_nonblocking(true).unwrap();
        match self.socket.read_to_end(&mut buf) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    panic!(e)
                }
            }
        };
        self.socket.set_nonblocking(false).unwrap();
        if buf.len() > 0 {
            println!("Stuff on the queue!! {:?}", buf);
        }
    }
}

impl Drop for ConnectedClient {
    fn drop(&mut self) {
        let res = self.socket.write(vec![0xe0, 0x0].as_ref());
        if res.is_err() {
            println!("Error disconnecting! {:?}", res);
        }
    }
}
impl Protocol {
    fn connect_payload(client_id: &str, keepalive: u8) -> Vec<u8> {
        let mut premsg: Vec<u8> = Vec::new();
        premsg.push(0x10);

        let mut msg = vec![0x0, 0x4, b'M', b'Q', b'T', b'T', 4, 2, 0, 0];

        let mut size: u8 = 10 + 2 + client_id.len() as u8;
        let clean_session = 1;
        msg[7] = clean_session << 1;

        // keepalive is u8 so keepalive >> 8 is always 0
        // and keepalive & 0xFF is always keepalive
        msg[8] |= 0;
        msg[9] |= keepalive;

        while size > 0x7f {
            println!("Size stuff {}", size);
            premsg.push((size & 0x7f) | 0x0);
            size >>= 7;
        }
        premsg.push(size);

        let mut payload: Vec<u8> = vec![];
        payload.extend(premsg);
        payload.extend(msg);
        payload.extend(Protocol::to_big_endian(client_id.len() as u16));
        payload.extend(client_id.as_bytes());
        payload
    }

    fn publish_payload(topic: &str, msg: &str, retain: bool, qos: QoS, pid: u16) -> Vec<u8> {
        let mut pkt: Vec<u8> = Vec::new();
        pkt.push(0x30);

        let mut size: u8 = 2 + topic.len() as u8 + msg.len() as u8;
        if qos > QoS::AtMostOnce {
            size += 2
        }
        pkt[0] |= ((qos as u8) << 1) | (retain as u8);

        while size > 0x7f {
            pkt.push((size & 0x7f) | 0x80);
            size >>= 7;
        }
        pkt.push(size);
        pkt.extend(Protocol::to_big_endian(topic.len() as u16));
        pkt.extend(topic.as_bytes());

        if qos > QoS::AtMostOnce {
            pkt.extend(Protocol::to_big_endian(pid));
        }
        pkt.extend(msg.as_bytes());
        pkt
    }

    fn to_big_endian(n: u16) -> Vec<u8> {
        vec![(n >> 8) as u8, (n & 0xFF) as u8]
    }
}
