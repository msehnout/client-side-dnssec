# Repository containing various experiments

 * Config DNS - utility that loads configuration from the running NetworkManager instance. It then configures Knot Resolver based on the connections, so that queries are sent only to appropriate networks (so your private queries does not leak to your corporate network and vice versa)
 * dns roadblock tests - attempt to implement RFC 8027 with Trust DNS API
 * NM\* - automatically generated API binding for NetworkManager. It uses GI repository.
 * scripts - so far only NM connection dump
 * dbus-connection - example of Rust code, that connects to NetworkManager over D-Bus
