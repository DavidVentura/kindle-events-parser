use libopenlipc_sys::{rLIPC, LipcResult};
use mqtt_simple::publish_once;
use std::io::{self, Write};

#[derive(Clone, Debug)]
enum Events {
    WifiDisconnected,
    WifiConnected,
    ScreenOff,
    ScreenOn,
    BatteryChanged,
    Unknown,
}

#[derive(Debug)]
struct EventFilter<'a> {
    source: &'a str,
    events: Vec<&'a str>,
}

impl Events {
    fn from_str(s: &str) -> Events {
        match s {
            "cmConnected" => Events::WifiConnected,
            "goingToScreenSaver" => Events::ScreenOff,
            "outOfScreenSaver" => Events::ScreenOn,
            "battLevelChanged" => Events::BatteryChanged,
            "suspending" => Events::WifiDisconnected,
            "readyToSuspend" => Events::WifiDisconnected,
            _ => Events::Unknown,
        }
    }
    fn to_topic(&self) -> Option<&'static str> {
        match self {
            Events::WifiDisconnected => Some("KINDLE/CONNECTED"),
            Events::WifiConnected => Some("KINDLE/CONNECTED"),
            Events::ScreenOff => Some("KINDLE/SCREEN_STATE"),
            Events::ScreenOn => Some("KINDLE/SCREEN_STATE"),
            Events::BatteryChanged => Some("KINDLE/BATTERY_STATE"),
            Events::Unknown => None,
        }
    }
}

fn run_and_match(source: &str, in_event: &str, res: Option<LipcResult>) {
    println!("[{}] {} || {:?}", source, in_event, res);

    let ev = Events::from_str(in_event);
    let topic = ev.to_topic();

    let msg = match (ev, res) {
        (Events::BatteryChanged, Some(LipcResult::NUM(batt))) => {
            println!("Battery at {}%", batt);
            Some(batt.to_string())
        }
        (Events::WifiDisconnected, _) => {
            println!("Wifi Disconnected");
            Some(String::from("0"))
        }
        (Events::WifiConnected, _) => {
            println!("Wifi Connected");
            Some(String::from("1"))
        }
        (Events::ScreenOn, _) => {
            println!("Screen on");
            Some(String::from("1"))
        }
        (Events::ScreenOff, _) => {
            println!("Screen off");
            Some(String::from("0"))
        }
        _ => {
            println!("No idea what i got..");
            None
        }
    };

    if let Some(m) = msg {
        let topic = topic.unwrap();
        println!("Publishing {} to {}", m, topic);
        let res = publish_once(
            String::from("KINDLE"),
            String::from("192.168.20.125"),
            topic,
            m.as_str(),
            false,
        );
        match res {
            Err(e) => println!("Failed to publish! {:?}", e),
            Ok(()) => (),
        }
    }
}

fn main() {
    println!("Started!");

    let r = rLIPC::new().unwrap();

    for filter in vec![
        EventFilter {
            source: "com.lab126.powerd",
            //events: vec!["goingToScreenSaver", "battLevelChanged"],
            events: vec![],
        },
        EventFilter {
            source: "com.lab126.appmgrd",
            events: vec![],
        },
        EventFilter {
            source: "com.lab126.wifid",
            events: vec!["cmConnected"],
        },
    ] {
        if filter.events.is_empty() {
            r.subscribe(filter.source, None, |source, ev, res| {
                run_and_match(source, ev, res)
            })
            .unwrap();
        } else {
            for e in filter.events {
                r.subscribe(filter.source, Some(e), |source, ev, res| {
                    run_and_match(source, ev, res)
                })
                .unwrap();
            }
        }
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        print!(".");
        io::stdout().flush().unwrap();
    }
}
