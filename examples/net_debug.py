#!/usr/bin/python3
import socket
import sys
import json

ip = "127.0.0.1"
port = 42069

# Create socket for server
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
print("Do Ctrl+c to exit the program !!")

s.bind(("0.0.0.0", 3198))

# Let's send data through UDP protocol
while True:
    data, addr = s.recvfrom(8192)
    payload = json.loads(data.decode('utf-8'))
    print("Received:", payload)
    response = json.dumps({"action": {"type": "Custom"}})
    s.sendto(response.encode('utf-8'), (ip, port))
    print("Responded")
    #data, address = s.recvfrom(4096)
    #print("\n\n 2. Client received : ", data.decode('utf-8'), "\n\n")
# close the socket
s.close()
