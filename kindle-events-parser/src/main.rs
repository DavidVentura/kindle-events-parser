use libopenlipc_sys::rLIPC;
use phf::phf_map;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;

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

fn run_and_match(filter: EventFilter, tx: Sender<EventData>) {
    // event_source: &str, event_name: &str
    let mut c = Command::new("lipc-wait-event")
        .args(vec!["-m", filter.source, filter.events.join(",").as_str()])
        .stdout(Stdio::piped())
        .spawn()
        .expect(format!("Could not start lipc-wait-event with filter {:?}", filter).as_str());

    let stdout = c.stdout.as_mut().unwrap();
    let reader = BufReader::new(stdout);
    for result in reader.lines() {
        if result.is_err() {
            println!("Error!: {:?}", result);
            continue;
        }
        let line = result.unwrap();
        println!("[{}] {}", filter.source, line);
        for k in MAP.keys().into_iter() {
            if line.starts_with(k) {
                let event = MAP.get(k).unwrap().clone();
                println!("This is a <{:?}> Event", event);
                let ed = EventData { event, data: line };
                tx.send(ed).unwrap();
                break;
            }
        }
    }
    c.wait().unwrap();
}

fn main() {
    println!("Started!");
    let (tx, rx): (Sender<EventData>, Receiver<EventData>) = mpsc::channel();

    let mut workers = Vec::new();

    let r = rLIPC::new().unwrap();

    for filter in vec![
        EventFilter {
            source: "com.lab126.powerd",
            events: vec!["goingToScreenSaver", "battLevelChanged"],
        },
        EventFilter {
            source: "com.lab126.wifid",
            events: vec!["cmConnected"],
        },
    ] {
        let _tx = tx.clone();
        workers.push(spawn(|| run_and_match(filter, _tx)));
    }

    let event_parser = spawn(move || loop {
        let rcv = rx.recv().unwrap();
        match (rcv.event, rcv.data) {
            (Event::BatteryChanged, data) => {
                // "battLevelChanged <> " where <> is u32 always -> i32 (using -1 to indicate some
                // unexpected error?)
                let battery_status = data.split(" ").nth(1).unwrap_or("-1").parse().unwrap_or(-1);
                println!("Battery at {}%", battery_status);
            }
            (Event::Connected, _) => println!("Wifi Connected"),
            (Event::ScreenOn, _) => println!("Screen on"),
            (Event::ScreenOff, _) => println!("Screen off"),
        }
    });

    workers.push(event_parser);
    for w in workers {
        w.join().expect("Job died!");
    }
}
