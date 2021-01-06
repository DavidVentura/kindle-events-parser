use mqtt_simple::{Client, QoS};
use std::thread::sleep;
fn main() {
    let mut c = Client::new(String::from("my name"), String::from("192.168.20.125")).unwrap();
    let mut c = c.connect(5).unwrap();

    loop {
        c.publish("some_topic", "AAABBBCCCABABABABABA", false, QoS::AtMostOnce)
            .unwrap();
        println!("hi");
        sleep(std::time::Duration::from_secs(1));
    }
}
