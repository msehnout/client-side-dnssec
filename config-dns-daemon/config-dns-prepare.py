#!/bin/env python3

import socket
import sys
import json
import gi
gi.require_version('NM', '1.0')
from gi.repository import NM

if __name__ == "__main__":
    # create Client object
    client = NM.Client.new(None)

    # get all connections
    connections = client.get_active_connections()
    conn_list = []

    # print the connections
    for c in connections:
        #default = "default" if c.
        id = c.get_id()
        if "virbr" in id or "docker" in id:
            continue
        cfg = c.get_ip4_config()
        addr = [(x.get_address(), x.get_prefix()) for x in cfg.get_addresses()]
        new_conn = {}
        new_conn['id'] = id
        new_conn['type'] = c.get_connection_type()
        new_conn['default'] = c.get_default()
        new_conn['addresses'] = [str(x.get_address())+'/'+str(x.get_prefix()) for x in cfg.get_addresses()]
        #new_conn['prefix'] = c.get_prefix()
        new_conn['nameservers'] = cfg.get_nameservers()
        new_conn['domains'] = list(set(cfg.get_domains()+cfg.get_searches()))
        conn_list.append(new_conn)

    # Create a UDS socket
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    # Connect the socket to the port where the server is listening
    server_address = '/var/run/config-dns-daemon/control'
    print('connecting to {}'.format(server_address))
    try:
        sock.connect(server_address)
    except socket.error as msg:
        print(msg)
        sys.exit(1)

    try:
        # Send data
        message = json.JSONEncoder().encode(conn_list).encode('utf-8')
        message += b'\n'
        print('sending {!r}'.format(message))
        sock.sendall(message)

        amount_received = 0
        amount_expected = len('Success')

        while amount_received < amount_expected:
            data = sock.recv(16)
            amount_received += len(data)
            print('received {!r}'.format(data))

    finally:
        print('closing socket')
        sock.close()
