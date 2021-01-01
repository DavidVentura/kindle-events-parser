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
    BatteryChanged,
}

#[derive(Debug)]
struct EventData {
    event: Event,
    data: String,
}

static MAP: phf::Map<&'static str, Event> = phf_map! {
    "cmStateChange \"CONNECTED\"" => Event::Connected,
    "goingToScreenSaver" => Event::ScreenOff,
    "battLevelChanged" => Event::BatteryChanged,
};

fn run_and_match(event_source: &str, event_name: &str, tx: Sender<EventData>) {
    let mut c = Command::new("lipc-wait-event")
        .args(vec!["-m", event_source, event_name])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = c.stdout.as_mut().unwrap();
    let reader = BufReader::new(stdout);
    for result in reader.lines() {
        if result.is_err() {
            println!("Error!: {:?}", result);
            continue;
        }
        let line = result.unwrap();
        println!("[{}] {}", event_source, line);
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

    let power_tx = tx.clone();
    let wifi_tx = tx.clone();

    let power = spawn(|| {
        run_and_match(
            "com.lab126.powerd",
            "goingToScreenSaver,battLevelChanged",
            power_tx,
        )
    });
    let wifi = spawn(|| run_and_match("com.lab126.wifid", "cmStateChange", wifi_tx));

    let event_parser = spawn(move || loop {
        let rcv = rx.recv().unwrap();
        match (rcv.event, rcv.data) {
            (Event::BatteryChanged, data) => {
                // "battLevelChanged <> " where <> is u32 always -> i32 (using -1 to indicate some
                // unexpected error?)
                let battery_status: i32 =
                    data.split(" ").nth(1).unwrap_or("-1").parse().unwrap_or(-1);
                println!("Battery at {}%", battery_status);
            }
            (Event::Connected, _) => println!("Wifi Connected"),
            (Event::ScreenOff, _) => println!("Screen off"),
        }
    });

    event_parser.join().unwrap();
    println!("Done receiving");
    wifi.join().unwrap();
    power.join().unwrap();
}
