use std::net::*; 
use std::{str, thread};
use std::time::Duration;
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::cell::UnsafeCell;

use anyhow::Result;
use xxhash_rust::xxh32::xxh32;

const MAX_PORT: u32 = 65536;
const MIN_PORT: u32 = 49152;

pub const BUFFER_SIZE: usize = 1024;

pub type Member = UdpSocket;

pub struct MemberTable<'a> {
    buf: UnsafeCell<[u8; BUFFER_SIZE]>,
    table: UnsafeCell<HashMap<&'a str, (Member, Group)>>,
    nonblocking: bool,
}

impl<'a> MemberTable<'a> {
    pub fn nonblocking() -> Self {
        Self {
            buf: UnsafeCell::new([0; BUFFER_SIZE]),
            table: UnsafeCell::new(HashMap::new()),
            nonblocking: true,
        }
    }
    pub fn preload(&self, msgs: &[&'a str]) {
        msgs.iter().for_each(|msg| {
            let _ = self.get(msg);
        });
    }
    pub fn send(&self, msg: &'a str, data: Option<&str>) {
        let (sock, group) = self.get(msg);
        
        let data = data.unwrap_or(msg);
        sock.send_to(data.as_bytes(), group).unwrap();
        println!("sending to: {:?}", group);
    }
    pub fn recv(&self, msg: &'a str) -> Option<String> {
        let buf = unsafe { &mut *self.buf.get() };
        let (sock, _) = self.get(msg);

        let (num_bytes, _) = sock.recv_from(buf).ok()?;
        let data = str::from_utf8(&buf[..num_bytes]).unwrap();
        Some(data.to_string())
    }
    fn get(&self, msg: &'a str) -> (&UdpSocket, &Group) {
        let table = unsafe { &mut *self.table.get() };
        let (sock, group) = table.entry(msg).or_insert_with(|| {
            let (m, g) = (member(), group(msg));
            if self.nonblocking {
                m.set_nonblocking(true).unwrap();
            }
            join(&m, &g).unwrap();
            (m, g)
        });
        (sock, group)
    }
}

impl<'a> Default for MemberTable<'a> {
    fn default() -> Self {
        Self {
            buf: UnsafeCell::new([0; BUFFER_SIZE]),
            table: UnsafeCell::new(HashMap::new()),
            nonblocking: false,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Group {
    group: SocketAddrV4,
    join: SocketAddrV4,
}

impl ToSocketAddrs for Group {
    type Iter = std::option::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        self.group.to_socket_addrs()
    }
}

fn group(msg: &str) -> Group {
    let hash = xxh32(msg.as_bytes(), 0);
    //let ip = Ipv4Addr::new(
    //    224,
    //    ((hash >> 13) & 0xFF) as u8,
    //    ((hash >> 21) & 0xFF) as u8,
    //    ((hash >> 29) & 0xFF) as u8,
    //);
    let port = hash % (MAX_PORT - MIN_PORT) + MIN_PORT;
    let join_port = (hash + 1) % (MAX_PORT - MIN_PORT) + MIN_PORT;
    
    //let addr = format!("{}:{}", ip, port);
    let addr = format!("255.255.255.255:{}", port);
    let join = format!("0.0.0.0:{}", join_port);

    Group {
        group: addr.parse().unwrap(),
        join: join.parse().unwrap(),
    }
}

fn member() -> Member {
    let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
    sock.set_broadcast(true).unwrap();
    sock
}

fn join(member: &Member, group: &Group) -> Result<()> {
    // add the member to the multicast ip of the group
    // join_multicast(member, group.group)?;

    // create forwarding service if non-existant
    if let Ok(f_sock) = UdpSocket::bind(group.group) {
        println!("forwarding service listening on {:?}", group.group);
        let j_sock = UdpSocket::bind(group.join)?;
        // join_multicast(&f_sock, group.group)?; 

        thread::spawn(move || forwarding_service(f_sock, j_sock));
    }

    // submit the member on the group.join address
    member.send_to(b"DAVE", group.join)?; 

    Ok(())
}

fn forwarding_service(f_sock: UdpSocket, j_sock: UdpSocket) -> ! {
    let members = Arc::new(Mutex::new(HashSet::new()));

    // add new members as they join
    let member_writer = Arc::clone(&members);
    thread::spawn(move || {
        let mut buf = [0; BUFFER_SIZE];
        loop {
            let (_, addr) = j_sock.recv_from(&mut buf).unwrap();
            member_writer.lock().unwrap().insert(addr);
        }
    });

    // forward network data to local members
    let exit = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut buf = [0; BUFFER_SIZE];
    loop {
        let Ok((num_bytes, _)) = f_sock.recv_from(&mut buf) else {
            thread::sleep(Duration::from_millis(100));
            continue;
        };
        for member in members.lock().unwrap().iter() {
            let _ = exit.send_to(&buf[..num_bytes], member);
        }
    }
}

// fn join_multicast(sock: &UdpSocket, addr: SocketAddrV4) -> Result<()> {
//     sock.set_multicast_loop_v4(true)?;
//     sock.join_multicast_v4(addr.ip(), &Ipv4Addr::UNSPECIFIED)?;
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    const PHRASE: &'static str = "Well Hello There, Old Sport!";
    
    #[test]
    fn group_hash_validation() {
        let want = Group {
            group: "224.200.82.1:52441".parse().unwrap(),
            join: "0.0.0.0:52442".parse().unwrap(),
        };
        let got = group(PHRASE);

        assert_eq!(want, got);
    }
    #[test]
    fn send_receive() {
        let group = group(PHRASE);

        let (sender, receiver) = (member(), member());
        join(&sender, &group).unwrap();
        join(&receiver, &group).unwrap();

        thread::spawn(move || {
            sender.send_to(b"OLD SPORT", &group).unwrap();
        });

        let mut buf = [0; BUFFER_SIZE];
        receiver.recv_from(&mut buf).unwrap();
    }
    #[test]
    fn send_receive_multiple() {
        let group = group(PHRASE);
        let mut threads = vec![];

        let (s1, r1, r2) = (member(), member(), member());
        join(&s1, &group).unwrap();
        join(&r1, &group).unwrap();
        join(&r2, &group).unwrap();
        
        threads.push(thread::spawn(move || {
            s1.send_to(b"OLD SPORT", &group).unwrap();
        }));

        threads.push(thread::spawn(move || {
            let mut buf = [0; BUFFER_SIZE];
            r1.recv_from(&mut buf).unwrap();
        }));

        threads.push(thread::spawn(move || {
            let mut buf = [0; BUFFER_SIZE];
            r2.recv_from(&mut buf).unwrap();
        }));

        threads.into_iter().for_each(|t| t.join().unwrap());
    }
    #[test]
    fn member_table_basic() {
        thread::spawn(|| {
            let mt = MemberTable::default();

            assert!(mt.recv("#entity-cam09").is_some());
            assert_eq!(
                mt.recv("@office-speakers"), 
                Some("terminate".to_string()),
            );
        });
        thread::sleep(Duration::from_millis(100));

        let mt = MemberTable::default();
        mt.send("@office-speakers", Some("terminate"));
        mt.send("#entity-cam09", None);
    }
    #[test]
    fn with_python() {
        let mt = MemberTable::default();
        mt.send("#entity-cam09", None);
        mt.send("@office-speakers", Some("terminate"));
    }
}
