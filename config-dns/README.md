# Requirements

```bash
# install knot-resolver
sudo dnf install knot resolver
# copy files form config to etc
sudo cp config/* /etc/systemd/system/
# set up interface
sudo ip address add 127.0.0.2/8 dev lo
# start dns socket
sudo systemctl start kresd.socket
# start control socket = unix domain
sudo systemctl start kresd-control@1.socket
# build config-dns binary
cargo build
# run the trigger script
sudo bash trigger.sh
# try by hand
# OR
# turn off dnssec-trigger
# set resolv conf to 127.0.0.2
# and enjoy! (no promises)
```
