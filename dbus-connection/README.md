# Examples of DBus connections

All of them are written in Rust using the dbus-rs crate, which is used e.g. by Stratis storage project.

In order to find out what signals are emitted by NetworkManager, you can use `dbus-monitor` utility:
```
$ dbus-monitor --system "type='signal',sender='org.freedesktop.NetworkManager',interface='org.freedesktop.NetworkManager'"
```
When the monitor is running, try to change the system connections somehow, e.g. disconnect and reconnect your Wi-Fi. You should see output like this one:
```
signal time=1535016703.597745 sender=org.freedesktop.DBus -> destination=:1.1358 serial=2 path=/org/freedesktop/DBus; interface=org.freedesktop.DBus; member=NameAcquired
   string ":1.1358"
signal time=1535016708.474548 sender=:1.13 -> destination=(null destination) serial=2979 path=/org/freedesktop/NetworkManager; interface=org.freedesktop.NetworkManager; member=PropertiesChanged
   array [
      dict entry(
         string "ActiveConnections"
         variant             array [
               object path "/org/freedesktop/NetworkManager/ActiveConnection/4"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/3"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/1"
            ]
      )
      dict entry(
         string "WirelessEnabled"
         variant             boolean false
      )
   ]
signal time=1535016715.542467 sender=:1.13 -> destination=(null destination) serial=3066 path=/org/freedesktop/NetworkManager; interface=org.freedesktop.NetworkManager; member=PropertiesChanged
   array [
      dict entry(
         string "WirelessEnabled"
         variant             boolean true
      )
   ]
signal time=1535016715.593116 sender=:1.13 -> destination=(null destination) serial=3081 path=/org/freedesktop/NetworkManager; interface=org.freedesktop.NetworkManager; member=PropertiesChanged
   array [
      dict entry(
         string "ActiveConnections"
         variant             array [
               object path "/org/freedesktop/NetworkManager/ActiveConnection/4"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/3"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/1"
            ]
      )
   ]
signal time=1535016719.009538 sender=:1.13 -> destination=(null destination) serial=3173 path=/org/freedesktop/NetworkManager; interface=org.freedesktop.NetworkManager; member=PropertiesChanged
   array [
      dict entry(
         string "ActiveConnections"
         variant             array [
               object path "/org/freedesktop/NetworkManager/ActiveConnection/7"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/4"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/3"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/1"
            ]
      )
   ]
signal time=1535016719.012313 sender=:1.13 -> destination=(null destination) serial=3180 path=/org/freedesktop/NetworkManager; interface=org.freedesktop.NetworkManager; member=PropertiesChanged
   array [
      dict entry(
         string "ActiveConnections"
         variant             array [
               object path "/org/freedesktop/NetworkManager/ActiveConnection/7"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/4"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/3"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/1"
            ]
      )
   ]
signal time=1535016719.017040 sender=:1.13 -> destination=(null destination) serial=3189 path=/org/freedesktop/NetworkManager; interface=org.freedesktop.NetworkManager; member=PropertiesChanged
   array [
      dict entry(
         string "ActiveConnections"
         variant             array [
               object path "/org/freedesktop/NetworkManager/ActiveConnection/7"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/4"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/3"
               object path "/org/freedesktop/NetworkManager/ActiveConnection/1"
            ]
      )
   ]
```
as you can see there is multiple signals emitted for one reconnection, which is unfortunate and I'm not sure what is the cause. Analogy from electrical engineering would be switch bouncing:
https://en.wikipedia.org/wiki/Switch#Contact\_bounce
so I would recommend similar solution :) 
