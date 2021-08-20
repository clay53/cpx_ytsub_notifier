use std::io::{
    stdout,
    stdin,
    Read,
    Write,
};

use serde::Deserialize;

#[derive(Deserialize)]
struct YTSubCountResponseChannelStatistics {
    #[serde(rename = "subscriberCount")]
    subscriber_count: String,
}

#[derive(Deserialize)]
struct YTSubCountResponseChannel {
    statistics: YTSubCountResponseChannelStatistics,
}

#[derive(Deserialize)]
struct YTSubCountResponse {
    items: Vec<YTSubCountResponseChannel>,
}

fn main() {
    let ports = serialport::available_ports().expect("Couldn't find any available ports.");
    for (i, port) in ports.iter().enumerate() {
        println!("{}: {}", i, port.port_name);
    }
    let mut port_choice: Result<usize, ()> = Err(());
    while port_choice.is_err() {
        print!("Enter a number between 0 and {} to choose a port: ", ports.len()-1);
        stdout().flush();
        let mut input = String::new();
        stdin().read_line(&mut input);
        let input = input.trim_end();
        match input.parse::<usize>() {
            Ok(port_num) => {
                if port_num < ports.len() {
                    port_choice = Ok(port_num);
                } else {
                    println!("Please enter a number between 0 and {}", ports.len()-1);
                }
            },
            Err(_) => println!("Please enter a number")
        }
    }
    let port_info = ports.get(port_choice.unwrap()).unwrap();
    let mut port = serialport::new(port_info.port_name.clone(), 9600).open().unwrap();

    let ytapi_key = rpassword::prompt_password_stdout("Please enter your YouTube Data API V3 Key: ").unwrap();
    let mut channel_id = String::new();
    print!("Please eneter your channel id: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut channel_id).unwrap();

    let mut subscriber_count: u32 = || -> u32 {
        loop {
            match get_subscriber_count(&channel_id, &ytapi_key) {
                Ok(subscriber_count) => return subscriber_count,
                Err(err) => println!("Error getting subscriber count: {}", err)
            }
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    }();

    println!("Starting with a subscriber count of: {}", subscriber_count);
    loop {
        match get_subscriber_count(&channel_id, &ytapi_key) {
            Ok(new_subscriber_count) => {
                if subscriber_count < new_subscriber_count {
                    subscriber_count = new_subscriber_count;
                    println!("Gained a subscriber. New count: {}", subscriber_count);
                    port.write(&[43]).unwrap();
                } else if subscriber_count > new_subscriber_count {
                    subscriber_count = new_subscriber_count;
                    println!("Lost a subscriber. New count: {}", subscriber_count);
                    port.write(&[45]).unwrap();
                }
            },
            Err(err) => println!("Error getting subscriber count: {}", err)
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

fn get_subscriber_count(channel_id: &str, ytapi_key: &str) -> Result<u32, String> {
    match reqwest::blocking::get(format!("https://youtube.googleapis.com/youtube/v3/channels?part=statistics&id={}&key={}", channel_id, ytapi_key)) {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<YTSubCountResponse>() {
                    Ok(deserialized) => {
                        let subscriber_count = deserialized.items.get(0).unwrap().statistics.subscriber_count.parse::<u32>().unwrap();
                        return Ok(subscriber_count)
                    },
                    Err(err) => return Err(err.to_string())
                }
            } else {
                return Err(format!("Server error getting subscriber count: {}", response.text().unwrap()));
            }
        },
        Err(err) => {
            return Err(format!("Failed to request subscriber count: {}", err));
        }
    }
}