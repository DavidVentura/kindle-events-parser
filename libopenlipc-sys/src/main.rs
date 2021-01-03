use libopenlipc_sys::rLIPC;
use std::io::{self, Write};

fn main() {
    let r = rLIPC::new().unwrap();
    let batt = r.get_int_prop("com.lab126.powerd", "battLevel").unwrap();
    println!("Battery: {}", batt);
    let batt_t = r.get_int_prop("com.lab126.powerd", "battTemperature");
    match batt_t {
        Ok(temp) => println!("BatteryTemp: {}", temp),
        Err(err) => println!("Failed to get temp: {}", err),
    }

    println!("Subscribing to ALL power events");
    let res = r.subscribe("com.lab126.powerd", None, |a, b| {
        println!("[{}] {}", a, b);

        let reader_status = r
            .get_str_prop("com.lab126.acxreaderplugin", "allReaderData")
            .unwrap();
        println!("allreader data: {}", reader_status);
    });
    if res.is_err() {
        println!("Failed to subscribe!! {:?}", res);
        return;
    }

    println!("Subscribed, ctrl-c to stop");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        print!(".");
        io::stdout().flush().unwrap();
    }
}
