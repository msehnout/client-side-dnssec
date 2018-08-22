getxml:
	wget https://raw.githubusercontent.com/heftig/NetworkManager/master/introspection/nm-settings.xml

gencode:
	dbus-codegen-rust < nm-settings.xml > nmdbus.rs
