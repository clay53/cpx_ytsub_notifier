use std::{
    collections::{
        HashMap,
    },
    fs,
    io::{
        self,
        stdout,
        stdin,
        Write,
    }
};

use serde::{Serialize, Deserialize};

type Presets = HashMap<String, Preset>;

#[derive(Deserialize, Serialize, Clone)]
struct Preset {
    pub ytapi_key: String,
    pub channel_id: String,
}

#[derive(Deserialize)]
struct YTSubCountResponseChannelStatistics {
    #[serde(rename = "subscriberCount")]
    pub subscriber_count: String,
}

#[derive(Deserialize)]
struct YTSubCountResponseChannel {
    pub statistics: YTSubCountResponseChannelStatistics,
}

#[derive(Deserialize)]
struct YTSubCountResponse {
    pub items: Vec<YTSubCountResponseChannel>,
}

fn main() {
    let ports = serialport::available_ports().expect("Couldn't find any available ports.");
    for (i, port) in ports.iter().enumerate() {
        println!("{}: {}", i, port.port_name);
    }
    let mut port_choice: Option<usize> = None;
    let prompt = format!("Enter a number between 0 and {} to choose a port: ", ports.len()-1);
    while port_choice.is_none() {
        let input = prompt_stdout(&prompt);
        match input.parse::<usize>() {
            Ok(port_num) => {
                if port_num < ports.len() {
                    port_choice = Some(port_num);
                } else {
                    println!("Please enter a number between 0 and {}.", ports.len()-1);
                }
            },
            Err(_) => println!("Please enter a number")
        }
    }
    let port_info = ports.get(port_choice.unwrap()).unwrap();
    let mut port = serialport::new(port_info.port_name.clone(), 9600).open().unwrap();

    let ytapi_key: String;
    let channel_id: String;

    match fs::read("./presets.json") {
        Ok(bytes) => {
            match serde_json::from_slice::<Presets>(&bytes[..]) {
                Ok(mut presets) => {
                    enum SelectState {
                        TryingLoad,
                        TryingUpdate,
                        TryingNew,
                        TryingRemove,
                        Skipping,
                    }
                    
                    let mut select_state = SelectState::TryingLoad;
                    let print_presets = |presets: &Presets| {println!("\nPresets:");
                    for (i, key) in presets.keys().enumerate() {
                        println!("{}: {}", i, key);
                    }};
                    print_presets(&presets);
                    println!("Enter 'l' to switch to loading a preset, enter 'u' to switch to updating a preset, enter 'n' to switch to creating a new preset, enter 'r' to switch to removing a preset, or enter 's' to continue without touching presets");
                    loop {
                        match select_state {
                            SelectState::TryingLoad => {
                                println!("Selecting a preset to load. Enter the index of the preset to select or switch to a different mode.");
                                let input = prompt_stdout("Enter your choice: ");
                                match input.as_str() {
                                    "l" => {},
                                    "u" => select_state = SelectState::TryingUpdate,
                                    "n" => select_state = SelectState::TryingNew,
                                    "r" => select_state = SelectState::TryingRemove,
                                    "s" => select_state = SelectState::Skipping,
                                    _ => match input.parse::<usize>() {
                                        Ok(i) if i < presets.len() => {
                                            let name = presets.keys().nth(i).unwrap();
                                            let preset = presets.get(name).unwrap();
                                            
                                            ytapi_key = preset.ytapi_key.clone(); // TODO: Maybe figure out how to do w/o clone
                                            channel_id = preset.channel_id.clone();
                                            break
                                        },
                                        _ => println!("Unknown option.")
                                    }
                                }
                            },
                            SelectState::TryingUpdate => {
                                println!("Selecting a preset to update/create. Enter the index of the preset to select or switch to a different mode.");
                                let input = prompt_stdout("Enter your choice: ");
                                match input.as_str() {
                                    "l" => select_state = SelectState::TryingLoad,
                                    "u" => {},
                                    "n" => select_state = SelectState::TryingNew,
                                    "r" => select_state = SelectState::TryingRemove,
                                    "s" => select_state = SelectState::Skipping,
                                    _ => match input.parse::<usize>() {
                                        Ok(i) if i < presets.len() => {
                                            let name = presets.keys().nth(i).unwrap().clone();
                                            let preset = presets.get_mut(&name).unwrap();

                                            println!("Updating preset called: {}\nPress enter to keep old values.", name);

                                            let mut changed = false;

                                            let ytapi_key_input = rpassword::prompt_password_stdout("Enter a new YT API key for this preset: ").unwrap();
                                            if ytapi_key_input != "" {
                                                preset.ytapi_key = ytapi_key_input;
                                                changed = true;
                                            }

                                            let channel_id_input = prompt_stdout("Enter a new channel id for this preset: ");
                                            if channel_id_input != "" {
                                                preset.channel_id = channel_id_input;
                                                changed = true;
                                            }

                                            let new_name_input = prompt_stdout("Enter a new name for this preset: ");
                                            if new_name_input != "" {
                                                let data = preset.clone();
                                                if presets.contains_key(&new_name_input) {
                                                    println!("A preset with that name already exists. Name change will not occur.")
                                                } else {
                                                    presets.remove(&name).unwrap();
                                                    presets.insert(new_name_input, data);
                                                    changed = true;
                                                    print_presets(&presets);
                                                }
                                            }
                                            if changed {
                                                write_presets(&presets);
                                            }
                                            select_state = SelectState::TryingLoad;
                                        },
                                        _ => println!("Unknown option.")
                                    }
                                }
                            },
                            SelectState::TryingNew => {
                                let name = prompt_stdout("Enter the name of your new preset (just press enter to go back to load): ");
                                if name == "" {
                                    select_state = SelectState::TryingLoad;
                                } else if presets.contains_key(&name) {
                                    println!("There's already a preset with that name.")
                                } else {
                                    presets.insert(name, Preset {
                                        ytapi_key: rpassword::prompt_password_stdout("Please enter your YouTube Data API V3 Key: ").unwrap(),
                                        channel_id: prompt_stdout("Please enter your channel id: ")
                                    });
                                    print_presets(&presets);
                                    write_presets(&presets);
                                    select_state = SelectState::TryingLoad;
                                }
                            },
                            SelectState::TryingRemove => {
                                println!("Selecting a preset to remove. Enter the index of the preset to select or switch to a different mode.");
                                let input = prompt_stdout("Enter your choice: ");
                                match input.as_str() {
                                    "l" => select_state = SelectState::TryingLoad,
                                    "u" => select_state = SelectState::TryingUpdate,
                                    "n" => select_state = SelectState::TryingNew,
                                    "r" => {},
                                    "s" => select_state = SelectState::Skipping,
                                    _ => match input.parse::<usize>() {
                                        Ok(i) if i < presets.len() => {
                                            let name = presets.keys().nth(i).unwrap().clone();
                                            presets.remove(&name);
                                            print_presets(&presets);
                                            write_presets(&presets);
                                        },
                                        _ => println!("Unknown option.")
                                    }
                                }
                            },
                            SelectState::Skipping => {
                                ytapi_key = rpassword::prompt_password_stdout("Please enter your YouTube Data API V3 Key: ").unwrap();
                                channel_id = prompt_stdout("Please enter your channel id: ");
                                break
                            }
                        }
                    }
                },
                Err(_) => {
                    let delete = prompt_stdout("Presets file could not be deserialized (./presets.json). The program will abort. Would you like to delete the presets file? (y/N): ");
                    if delete.eq_ignore_ascii_case("y") {
                        fs::remove_file("./presets.json").expect("Failed to delete presets file");
                    }
                    return
                }
            }
        },
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                let choice = prompt_stdout("Presets file not found. Would you like to create a new preset? (Y/n): ");
                if choice.eq_ignore_ascii_case("n") {
                    println!("Continuing, no preset will be created.");

                    ytapi_key = rpassword::prompt_password_stdout("Please enter your YouTube Data API V3 Key: ").unwrap();
                    channel_id = prompt_stdout("Please enter your channel id: ");
                } else {
                    let name = prompt_stdout("Enter the name of your new preset: ");

                    ytapi_key = rpassword::prompt_password_stdout("Please enter your YouTube Data API V3 Key: ").unwrap();
                    channel_id = prompt_stdout("Please enter your channel id: ");

                    let mut presets = Presets::new();
                    presets.insert(name, Preset {
                        ytapi_key: ytapi_key.clone(),
                        channel_id: channel_id.clone()
                    });
                    write_presets(&presets);
                }
            },
            _ => {
                println!("Failed to read presets file (./presets.json): {}", err);
                return
            }
        }
    }

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

fn prompt_stdout(prompt: &str) -> String {
    let mut out = String::new();
    print!("{}", prompt);
    stdout().flush().unwrap();
    stdin().read_line(&mut out).unwrap();
    let out = out.trim_end();
    return String::from(out)
}

fn write_presets(presets: &Presets) {
    fs::write("./presets.json", serde_json::to_string_pretty(&presets).unwrap()).unwrap();
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