use anyhow::Result;
use std::net::*; 
use std::thread;
use std::collections::HashSet;
use xxhash_rust::xxh32::xxh32;


// do not connect() to this addr, because though you should
// send to it, you will never be receiving from it.
pub fn addr(msg: &str) -> Result<SocketAddr> {
    let (ip, port) = hash_msg(msg);
    let addr = format!("{}:{}", ip, port).parse()?;

    if let Ok(sock) = UdpSocket::bind(&addr) {
        sock.set_multicast_loop_v4(true)?;
        sock.join_multicast_v4(&ip, &Ipv4Addr::UNSPECIFIED)?;

        thread::spawn(move || forwarding_service(sock));
    }
    Ok(addr)
}

// somehow add yourself to the local member set on the
// local message forwarding service
pub fn join(member_sock: &UdpSocket, forwarding_entrance: SocketAddr) -> Result<()> {
    // ...
    todo!()
}

fn forwarding_service(entrance: UdpSocket) {
    let local_members: HashSet<SocketAddr> = HashSet::new();
    let exit = UdpSocket::bind("0.0.0.0:0").unwrap();

    let mut buf = [0; 1024];
    loop {
        let (num_bytes, _) = entrance.recv_from(&mut buf).unwrap();
        for member in local_members.iter() {
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
