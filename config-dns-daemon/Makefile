all:
	cargo build --release

install:
	install -m 755 target/release/config-dns-daemon /usr/local/bin/
	install -m 755 01-trigger-config-dns.sh /etc/NetworkManager/dispatcher.d/
	install -m 755 config-dns-prepare.py /usr/local/libexec/
	install -m 644 control-dns-daemon.service /etc/systemd/system/
	mkdir /var/run/config-dns-daemon

clean:
	rm /usr/local/bin/config-dns-daemon
	rm /etc/NetworkManager/dispatcher.d/01-trigger-config-dns.sh
	rm /usr/local/libexec/config-dns-prepare.py
	rm /etc/systemd/system/control-dns-daemon.service
	rmdir /var/run/config-dns-daemon
