import socket
import time
import threading
import sys
import queue
import unittest
import struct
import xxhash
from typing import *
from collections import namedtuple

MAX_PORT = 65536
MIN_PORT = 49152

BUFFER_SIZE = 1024

# type Member = UdpSocket

class MemberTable:
    def __init__(self):
        self.table = {}

    def preload(self, msg):
        _, _ = self._get(msg)

    def send(self, msg, data=None):
        sock, group = self._get(msg)
        if data is None:
            data = msg
        sock.sendto(data.encode(), group.group)

    def recv(self, msg):
        sock, _ = self._get(msg)
        data, _ = sock.recvfrom(BUFFER_SIZE)
        return data.decode()

    def _get(self, msg):
        try:
            s, g = self.table[msg]
        except:
            s, g = (member(), group(msg))
            join(s, g)
            # ^allow throwing exception
            self.table[msg] = (s, g)
        finally:
            return s, g 

Group = namedtuple('Group', ['group', 'join'])
# group: SocketAddrV4
# join: SocketAddrV4

def group(msg):
    hash_32 = xxhash.xxh32(msg.encode()).intdigest()
    ip = [
        224,
        ((hash_32 >> 13) & 0xFF),# as u8,
        ((hash_32 >> 21) & 0xFF),# as u8,
        ((hash_32 >> 29) & 0xFF),# as u8,
    ]
    group_port = hash_32 % (MAX_PORT - MIN_PORT) + MIN_PORT
    join_port = (hash_32 + 1) % (MAX_PORT - MIN_PORT) + MIN_PORT

    group_ip = f"{ip[0]}.{ip[1]}.{ip[2]}.{ip[3]}"
    join_ip = f"0.0.0.0"

    return Group((group_ip, group_port), (join_ip, join_port))

def member():
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind(('0.0.0.0', 0))
    return sock

def join(member, group):
    join_multicast(member, group.group)

    try:
        f_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        f_sock.bind(group.group)

        j_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        j_sock.bind(group.join)
        join_multicast(f_sock, group.group)
        
        fs = threading.Thread(target=forwarding_service, args=(f_sock, j_sock))
        fs.daemon = True
        fs.start()
    except:
        pass
    member.sendto("KEBAB".encode(), group.join)

def forwarding_service(f_sock, j_sock):
    members = queue.Queue()

    def member_loader(ms):
        while True:
            _, addr = j_sock.recvfrom(BUFFER_SIZE)
            ms.put(addr)
    threading.Thread(target=member_loader, args=(members,)).start()
    
    exit_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    while True:
        #try:
        data, _ = f_sock.recvfrom(BUFFER_SIZE)
        for member in list(members.queue):
            exit_sock.sendto(data, member)
        #except:
        #    time.sleep(0.1)

def join_multicast(sock, addr):
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_LOOP, 1)
    ip = addr[0]
    group = socket.inet_aton(ip)
    mreq = struct.pack('4sL', group, socket.INADDR_ANY)
    # ^?? different than rust

    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)

#class DaveTest(unittest.TestCase):
#    def send_receive(self):
#        pass
#
#    def send_receive_multiple(self):
#        pass
#
#    def member_table_basic(self):
#        def mt_thread():
#            mt = MemberTable()
#
#            self.Assert(mt.recv("entity-cam09") is not None)
#
#            self.AssertEqual(
#                mt.recv("@office-speakers"),
#                "terminate",
#            )
#        threading.Thread(target=mt_thread)
#        time.sleep(0.1)
#
#        mt = MemberTable()
#        mt.send("@office-speakers", "terminate");
#        mt.send("#entity-cam09", None);

PHRASE = "Well Hello There, Old Sport!"

if __name__ == "__main__":
    pass
    #mt = MemberTable()
    #mt.preload("#entity-cam09")
    #mt.preload("@office-speakers")

    #print(mt.recv("#entity-cam09"))
    #print(mt.recv("@office-speakers"))

    # -----------------
   
    #want = Group(("224.200.82.1", 52441), ("0.0.0.0", 52442))
    #got = group(PHRASE)

    #print(want == got)
    #print(want)
    #print(got)

    
