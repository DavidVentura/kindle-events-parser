use libopenlipc_sys::rLIPC;
use phf::phf_map;
use std::io::{self, Write};

#[derive(Clone, Debug)]
enum Event {
    Connected,
    ScreenOff,
    ScreenOn,
    BatteryChanged,
}

#[derive(Debug)]
struct EventData {
    event: Event,
    data: String,
}

#[derive(Debug)]
struct EventFilter<'a> {
    source: &'a str,
    events: Vec<&'a str>,
}

static MAP: phf::Map<&'static str, Event> = phf_map! {
    "cmConnected" => Event::Connected,
    "goingToScreenSaver" => Event::ScreenOff,
    "outOfScreenSaver" => Event::ScreenOn,
    "battLevelChanged" => Event::BatteryChanged,
};

/*
static EVENT_TO_TOPIC: phf::Map<Event, &'static str> = phf_map! {
    Event::Connected => "KINDLE/CONNECTED",
    Event::ScreenOn => "KINDLE/SCREEN_STATE",
    Event::ScreenOff => "KINDLE/SCREEN_STATE",
 => "KINDLE/CONNECTED",
};

*/
#[allow(dead_code)]
#[allow(unused_variables)]
fn publish(topic: &str, value: &str) {}

fn run_and_match(source: &str, in_event: &str, intarg: Option<i32>, strarg: Option<String>) {
    println!("[{}] {} || {:?} || {:?}", source, in_event, intarg, strarg);
    for k in MAP.keys().into_iter() {
        if in_event.to_string() == k.to_string() {
            let event = MAP.get(k).unwrap().clone();
            match (event, intarg, strarg) {
                (Event::BatteryChanged, Some(batt), _) => {
                    println!("Battery at {}%", batt);
                }
                (Event::Connected, _, _) => println!("Wifi Connected"),
                (Event::ScreenOn, _, _) => println!("Screen on"),
                (Event::ScreenOff, _, _) => println!("Screen off"),
                _ => println!("No idea what i got.."),
            }
            break;
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
            source: "com.lab126.wifid",
            events: vec!["cmConnected"],
        },
    ] {
        if filter.events.is_empty() {
            r.subscribe(filter.source, None, run_and_match).unwrap();
        } else {
            for e in filter.events {
                r.subscribe(filter.source, Some(e), run_and_match).unwrap();
            }
        }
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        print!(".");
        io::stdout().flush().unwrap();
    }
}
