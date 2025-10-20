import socket
import json
import mmap
# import struct
from array import array


def request_dataset(name):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect("/tmp/dataset_manager.sock")
    s.send(name.encode())

    # Receive metadata and file descriptor
    fds = array("i")
    msg, ancdata, flags, addr = s.recvmsg(4096, socket.CMSG_LEN(4))
    for cmsg_level, cmsg_type, cmsg_data in ancdata:
        if cmsg_level == socket.SOL_SOCKET and cmsg_type == socket.SCM_RIGHTS:
            fds.frombytes(cmsg_data[:4])
    fd = fds[0]
    meta = json.loads(msg.decode())

    # Map the file
    mm = mmap.mmap(fd, 0, access=mmap.ACCESS_READ)
    return meta, mm


meta, mm = request_dataset("ETHUSDT-1h")
print(meta)
