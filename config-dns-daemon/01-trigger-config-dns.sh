#!/bin/sh

if [ -x /usr/libexec/dnssec-trigger-script ]; then
    exec /usr/libexec/config-dns-prepare-connections.py
fi

