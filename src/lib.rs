use anyhow::Result;
use std::net::*; 
use std::thread;
use std::collections::HashSet;
use xxhash_rust::xxh32::xxh32;


// do not connect() to this addr, because though you should
// send to it, you will never be receiving from it.
// (and binding to it will fail)
pub fn group(msg: &str) -> Result<SocketAddr> {
    let (ip, port) = hash_msg(msg);
    let addr = format!("{}:{}", ip, port).parse()?;

    if let Ok(sock) = UdpSocket::bind(&addr) {
        sock.set_multicast_loop_v4(true)?;
        sock.join_multicast_v4(&ip, &Ipv4Addr::UNSPECIFIED)?;

        thread::spawn(move || local_forwarding_service(sock));
    }
    Ok(addr)
}

// somehow add yourself to the member set on the
// local message forwarding service.
pub fn join(new_member_sock: &UdpSocket, group: SocketAddr) -> Result<()> {
    // ping_sock must be different from new_member_sock bc:
    // - nms is not bound to 127.0.0.1
    // - nms cannot be bound to 127.0.0.1 if it wants to
    //   reach the global group (which it must)
    // - nms must be bound to 127.0.0.1 if it only seeks
    //   too inform the local group of its port
    // - the ping must only inform the local group to avoid
    //   multiple remote groups forwarding to the same addr
    //   (resulting in duplicate packets)

    let ping_sock = UdpSocket::bind("127.0.0.1:0")?;

    let local_group = format!("127.0.0.1:{}", group.port());
    let local_member = format!("127.0.0.1:{}", new_member_sock.local_addr()?.port());
    ping_sock.send_to(local_member.as_bytes(), local_group).unwrap();

    // ...
    todo!();
    // ping the universal multicast ip + the port of the
    // global forwarding address.
    // ip: 225.<WellHelloThereOldSport>
    // port: group.ip
    //
    // this will 
}

const LOOPBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

fn local_forwarding_service(entrance: UdpSocket) {
    let members: HashSet<SocketAddr> = HashSet::new();
    let exit = UdpSocket::bind("0.0.0.0:0").unwrap();

    let mut buf = [0; 1024];
    loop {
        let (num_bytes, addr) = entrance.recv_from(&mut buf).unwrap();
        if addr.ip() == LOOPBACK {
            members.insert(&buf[..num_bytes]);
            continue;
        }
        for member in members.iter() {
            exit.send_to(&buf[..num_bytes], member).unwrap();
        }
    }
}

fn hash_msg(msg: &str) -> (Ipv4Addr, u16) {
    let hash = xxh32(msg.as_bytes(), 0);

    let addr = Ipv4Addr::new(
        224,
        ((hash >> 13) & 0xFF) as u8,
        ((hash >> 21) & 0xFF) as u8,
        ((hash >> 29) & 0xFF) as u8,
    );
    let port = 9999;
    // XXX generate port from hash 
    
    (addr, port)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_msg() {
        let phrase = "Well Hello There, Old Sport!";
        assert_eq!(
            hash_msg(phrase),
            (Ipv4Addr::new(0, 0, 0, 0), 0),
        );
    }
}
