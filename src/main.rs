use std::env;
use std::thread;

use inputbot::KeybdKey;

use crossbeam_channel::{bounded, Sender};

fn main() {
    println!("Hello, world!");

    /*
    Expected args are in format

    1. connection type
        - websocket
        - udp
        - rest
    2. port to connect to. Host will always be localhost
    */
    let args = env::args().collect::<Vec<String>>();

    assert!(args.len() == 3);

    let connection_type = &args[1];
    let port = &args[2];

    println!("conn: {connection_type}, port: {port}");

    let (s, r) = bounded::<String>(0);

    let (mut socket, response) =
        tungstenite::connect(url::Url::parse(format!("ws://127.0.0.1:{port}").as_str()).unwrap())
            .unwrap();

    let _handle = thread::spawn(move || loop {
        match r.recv() {
            Ok(m) => socket
                .write_message(tungstenite::Message::Text(m))
                .expect("Unable to write to websocket"),
            Err(_) => return,
        };
    });

    KeybdKey::bind_all(move |event| {
        println!("{:?}", event);
        s.send(format!("{:?}", event))
            .expect("Unable to begin sending on websocket");
    });

    inputbot::handle_input_events();
}
