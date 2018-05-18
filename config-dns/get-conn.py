#!/bin/env python3

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

    json.dump(conn_list, sys.stdout)
