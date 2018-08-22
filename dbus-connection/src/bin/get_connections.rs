extern crate dbus;

use dbus::{BusType, Connection, Path};
use dbus::stdintf::org_freedesktop_dbus::Properties;

fn main() {
    let c = Connection::get_private(BusType::System).unwrap();
    for i in vec![1, 3, 4, 12] {
        let p = c.with_path("org.freedesktop.NetworkManager", format!("/org/freedesktop/NetworkManager/ActiveConnection/{}", i), 5000);
        let ipconfig_path: Path = p.get("org.freedesktop.NetworkManager.Connection.Active", "Ip4Config").unwrap();
        println!("{:?}", ipconfig_path);
        let p = c.with_path("org.freedesktop.NetworkManager", ipconfig_path, 5000);
        let domains: Vec<String> = p.get("org.freedesktop.NetworkManager.IP4Config", "Domains").unwrap();
        println!("{:?}", domains);
    }
}
