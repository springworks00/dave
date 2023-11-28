use std::net::*; 
use std::{str, thread};
use std::time::Duration;
use std::collections::HashSet;

use anyhow::{anyhow, Result};
use xxhash_rust::xxh32::xxh32;

const LOOPBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const MAX_PORT: u32 = 65536;
const MIN_PORT: u32 = 49152;

// do not connect() to this addr, because though you should
// send to it, you will never be receiving from it.
// (and binding to it will fail)
pub fn group(msg: &str) -> Result<(UdpSocket, SocketAddr)> {
    let (ip, port) = hash_msg(msg);
    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>()?;

    let generalized_addr = format!("0.0.0.0:{}", port); 
    if let Ok(sock) = UdpSocket::bind(&generalized_addr) {
        sock.set_multicast_loop_v4(true)?;
        sock.join_multicast_v4(&ip, &Ipv4Addr::UNSPECIFIED)?;

        thread::spawn(move || local_forwarding_service(sock));
    }
    //Err(anyhow!(""))
    let sock = UdpSocket::bind("0.0.0.0:0").unwrap(); 
    sock.set_multicast_loop_v4(true)?;
    sock.join_multicast_v4(&ip, &Ipv4Addr::UNSPECIFIED)?;
    Ok((sock, addr))
}

pub fn join(new_member_sock: &UdpSocket, group: &SocketAddr) -> Result<()> {
    // ping_sock must be different from new_member_sock bc:
    // - nms is not bound to 127.0.0.1
    // - nms cannot be bound to 127.0.0.1 if it wants to
    //   reach the global group (which it must)
    // - nms must be bound to 127.0.0.1 if it only seeks
    //   too inform the local group of its port
    // - the ping must only inform the local group to avoid
    //   multiple remote groups forwarding to the same addr
    //   (resulting in duplicate packets being received)

    let ping_sock = UdpSocket::bind("127.0.0.1:0")?;

    let local_group = format!("127.0.0.1:{}", group.port());
    let localized_member = format!("127.0.0.1:{}", new_member_sock.local_addr()?.port());

    ping_sock.send_to(localized_member.as_bytes(), local_group)?;

    Ok(())
}

#[test]
fn join_group() {
    let phrase = "Well Hello There, Old Sport!";

    thread::spawn(|| {
      let (sock, g) = group(phrase).unwrap();
      join(&sock, &g).unwrap();
    
      for i in 0..5 {
          println!("sending: {}", i);
          sock.send_to(b"old sport", g).unwrap();
          thread::sleep(Duration::from_millis(1000));
      }
    });

    let (sock, g) = group(phrase).unwrap();
    //let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
    join(&sock, &g).unwrap();

    let mut buf = [0; 1024];
    assert!(sock.recv_from(&mut buf).is_ok());

    // XXX
    // conclusion: forwarding service is botched
    // possibilities:
    // - submittance not working on join().
    //   neither sock is even being received on the entrance,
    //   possibly because the entrance is not listening on
    //   loopback.
    // - no tmp solution. must rework the registration
    //   process, knowing that the entrance socket will not
    //   receive from 127.0.0.1, ONLY the multicast addr
}

fn local_forwarding_service(entrance: UdpSocket) {
    let mut members: HashSet<SocketAddr> = HashSet::new();
    let exit = UdpSocket::bind("0.0.0.0:0").unwrap();

    let mut buf = [0; 1024];
    loop {
        let Ok((num_bytes, addr)) = entrance.recv_from(&mut buf) else {
            // blocking receipt failed
            thread::sleep(Duration::from_millis(100));
            continue;
        };
        if addr.ip() != LOOPBACK {
            println!("got non-loopback");
            // addr is not a new member, so forward the data
            for member in members.iter() {
                let _ = exit.send_to(&buf[..num_bytes], member);
            }
            continue;
        }
        println!("got loopback");
        let Ok(addr) = str::from_utf8(&buf[..num_bytes]) else {
            // new member addr is not utf8 data
            continue;
        };
        let Ok(addr) = addr.parse::<SocketAddr>() else {
            // new member addr is not a valid SocketAddr
            continue;
        };
        println!("got valid loopback");
        members.insert(addr);
    }
}

fn hash_msg(msg: &str) -> (Ipv4Addr, u32) {
    let hash = xxh32(msg.as_bytes(), 0);
    let addr = Ipv4Addr::new(
        224,
        ((hash >> 13) & 0xFF) as u8,
        ((hash >> 21) & 0xFF) as u8,
        ((hash >> 29) & 0xFF) as u8,
    );
    let port = hash % (MAX_PORT - MIN_PORT) + MIN_PORT;
    
    (addr, port)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    //#[test]
    //fn hash_msg_basic() {
    //    let phrase = "Well Hello There, Old Sport!";
    //    assert_eq!(
    //        hash_msg(phrase),
    //        (Ipv4Addr::new(224, 200, 82, 1), 52441),
    //    );
    //}
}
