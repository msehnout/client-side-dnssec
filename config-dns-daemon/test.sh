#!/bin/bash

set -e
make all
sudo systemctl stop config-dns-daemon
sudo make clean
sudo make install
sudo systemctl start config-dns-daemon
sudo systemctl status config-dns-daemon
