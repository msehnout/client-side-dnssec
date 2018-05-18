#!/bin/bash

CONNECTIONS=$(python3 get-conn.py)
echo "Run configuration with these connections: ${CONNECTIONS}"

systemctl restart kresd@1.service
target/debug/config-dns "${CONNECTIONS}"

dnssec-trigger-control hotspot_signon
echo "nameserver 127.0.0.2" > /etc/resolv.conf
