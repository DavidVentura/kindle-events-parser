# mqtt-simple

The simplest possible mqtt client that can publish a message. Only supports `AtMostOnce` for `QoS`.

Translated from [micropython mqtt.simple](https://github.com/micropython/micropython-lib/blob/master/umqtt.simple/umqtt/simple.py)

Usage is

```rust
use mqtt_simple::publish_once;
let res = publish_once(
    String::from("KINDLE"), // identifier for the client
    String::from("192.168.20.125"), // target. No DNS support.
    topic,
    message,
    false, // retain
);

```
