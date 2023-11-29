use std::net::*; 
use std::{str, thread};
use std::time::Duration;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use xxhash_rust::xxh32::xxh32;

const MAX_PORT: u32 = 65536;
const MIN_PORT: u32 = 49152;


pub fn bind(msg: &str) -> Result<UdpSocket> {
    let (group_ip, group_port, declare_port) = hash_msg(msg);
    let group = format!("{}:{}", group_ip, group_port).parse::<SocketAddr>()?;

    // spawn forwarding if it does not exist on this device
    if let Ok(entrance) = UdpSocket::bind(&group) {
        entrance.set_multicast_loop_v4(true)?;
        entrance.join_multicast_v4(&group_ip, &Ipv4Addr::UNSPECIFIED)?;
        let local_declare = format!("0.0.0.0:{}", declare_port);
        let declare_sock = UdpSocket::bind(local_declare).unwrap();

        thread::spawn(move || local_forwarding_service(entrance, declare_sock));
    }

    // create member and declare to forwarding service
    let member_addr = format!("0.0.0.0:0"); //, group_ip);
    let member = UdpSocket::bind(member_addr)?;
    //let member_port: u16 = member.local_addr().unwrap().port();

    let local_declare = format!("127.0.0.1:{}", declare_port);
    member.send_to(b"OLD SPORT", local_declare)?;

    // connect member to the group
    member.set_multicast_loop_v4(true)?;
    println!("member joining multicast: {}", &group_ip);
    member.join_multicast_v4(&group_ip, &Ipv4Addr::UNSPECIFIED)?;

    member.connect(group)?;

    println!(
        "member ({}) connected to: {}",
        member.local_addr().unwrap(),
        member.peer_addr().unwrap(),
    );
    Ok(member)
}

fn local_forwarding_service(entrance: UdpSocket, declare: UdpSocket) {
    let members = Arc::new(Mutex::new(HashSet::new()));
   
    let group_ip = entrance.local_addr().unwrap().ip();
    let member_writer = Arc::clone(&members);
    thread::spawn(move || {
        let mut buf = [0; 1024];
        loop {
            let (_, addr) = declare.recv_from(&mut buf).unwrap();
            let rebuilt = format!("{}:{}", group_ip, addr.port());
            //member_writer.lock().unwrap().insert(rebuilt.clone());
            member_writer.lock().unwrap().insert(addr);
            println!("ping received: {}", addr);
        }
    });
    let entrance_addr = entrance.local_addr().unwrap();
    let mut buf = [0; 1024];
    loop {
        let Ok((num_bytes, _)) = entrance.recv_from(&mut buf) else {
            // blocking receipt failed
            thread::sleep(Duration::from_millis(100));
            continue;
        };
        println!("intiate distribution");
        for member in members.lock().unwrap().iter() {
            println!("| distributing: ({}) -> ({})", entrance_addr, member);
            let _ = entrance.send_to(&buf[..num_bytes], member);
        }
    }
}

#[test]
fn working() {
    let (group, join) = dave::addr("test");

    let sock = dave::bind(group);

    dave::join(sock, join); // send declare ping

    sock.send_to(b"old sport", group);

    // but also this may not work because the socket
    // needs to declare itself and the port to do so
    // would come from the dave::addr function.
    //
    // and `bind` should spawn the forwarder, because
    // that makes logical sense. we may just want to
    // call `dave::addr` to see what the hash generates,
    // not spawn a secret background process.
}

#[test]
fn join_group() {
    let phrase = "Well Hello There, Old Sport!";

    thread::spawn(|| {
        let sock = bind(phrase).unwrap();

        for i in 0..5 {
            sock.send(b"old sport").unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let sock = bind(phrase).unwrap();

    let mut buf = [0; 1024];
    assert!(sock.recv_from(&mut buf).is_ok());

    // what if the receiver comprehends the sending address
    // not as a multicast addr but the actual sender ip:port.
    //
    // but no, the sender is literally bound to the multicast
    // and port.
    //
    // try not doing connect, then recv_from to see if
    // the sender addr is indeed the multicast addr or not.
}


fn hash_msg(msg: &str) -> (Ipv4Addr, u32, u32) {
    let hash = xxh32(msg.as_bytes(), 0);
    let addr = Ipv4Addr::new(
        224,
        ((hash >> 13) & 0xFF) as u8,
        ((hash >> 21) & 0xFF) as u8,
        ((hash >> 29) & 0xFF) as u8,
    );
    let port = hash % (MAX_PORT - MIN_PORT) + MIN_PORT;
    let declaration_port = (hash + 1) % (MAX_PORT - MIN_PORT) + MIN_PORT;
    
    (addr, port, declaration_port)
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
