use std::fs;
use std::io::prelude::*;
use std::time::{Duration, SystemTime};

use dotenvy::dotenv;

fn main() {
    let _ = dotenv();
    let api_key = std::env::var("API_KEY").expect("expects an API key");
    let url: url::Url = format!("wss://api.geops.io/realtime-ws/v1/?key={api_key}").parse().unwrap();

    let mut out_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .read(false)
        .open("s-bahn-munich-live-map.jsonl")
        .expect("needs access to write the contents to the file");

    println!("URL: {url}");

    loop {
        let (mut socket, _) = tungstenite::connect(url.clone()).expect("should be able to connect");
        let _ = socket.send("BBOX 1152072 6048052 1433666 6205578 5 tenant=sbm".into());
        let _ = socket.send("BUFFER 100 100".into());
        let _ = socket.send("GET extra_geoms".into());
        let _ = socket.send("SUB extra_geoms".into());
        let _ = socket.send("GET healthcheck".into());
        let _ = socket.send("SUB healthcheck".into());
        let _ = socket.send("GET sbm_newsticker".into());
        let _ = socket.send("SUB sbm_newsticker".into());
        let _ = socket.send("GET station_schematic".into());
        let _ = socket.send("SUB station_schematic".into());
        let _ = socket.send("GET deleted_vehicles_schematic".into());
        let _ = socket.send("SUB deleted_vehicles_schematic".into());
        let _ = socket.send("GET trajectory_schematic".into());
        let _ = socket.send("SUB trajectory_schematic".into());
        let _ = socket.send("GET station".into());
        let _ = socket.send("SUB station".into());
        let _ = socket.send("GET deleted_vehicles".into());
        let _ = socket.send("SUB deleted_vehicles".into());
        let _ = socket.send("GET trajectory".into());
        let _ = socket.send("SUB trajectory".into());

        let _ = socket.send("PING".into());
        let mut last_ping = SystemTime::now();

        loop {
            match socket.read() {
                Ok(msg) => {
                    match msg {
                        tungstenite::Message::Text(text) => {
                            writeln!(out_file, "{text}")
                                .expect("writing message to file without error");
                        }
                        // tungstenite::Message::Binary(bin) => todo!(),
                        tungstenite::Message::Close(_) => break,
                        // tungstenite::Message::Ping(_) => todo!(),
                        // tungstenite::Message::Pong(_) => todo!(),
                        // tungstenite::Message::Frame(_) => todo!(),
                        _ => {}
                    }
                    // println!("Received: {}", msg);
                    if last_ping.elapsed().expect("Valid time") >= Duration::from_secs(10) {
                        let _ = socket.send("PING".into());
                        last_ping = SystemTime::now();
                    }
                }
                Err(err) => {
                    eprintln!("ERR: {err}");
                    break;
                }
            }
        }
    }
}
