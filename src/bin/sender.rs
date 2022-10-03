use std::env;
use std::net::UdpSocket;
use std::thread;
use std::thread::JoinHandle;

use inputbot::{KeybdKey, MouseButton};

use crossbeam_channel::{bounded, Receiver, Sender};

fn main() {
    println!("Hello, world!");

    /*
    Expected args are in format

    1. connection type
        - websocket
        - udp
        - debug (just echos all inputs to stdout)
    2. port to connect to. Host will always be localhost
    */
    let args = env::args().collect::<Vec<String>>();

    assert!(args.len() == 3);

    let connection_type = &args[1];
    let port = &args[2];

    println!("conn: {connection_type}, port: {port}");

    let (key_sender, key_receiver) = bounded::<String>(1);
    let (mouse_sender, mouse_receiver) = bounded::<String>(1);

    let (sender, receiver) = bounded::<String>(0);

    let _handle = match connection_type.to_lowercase().as_str() {
        "websocket" => websocket(receiver, &format!("ws://127.0.0.1:{port}")),
        "udp" => udp(receiver, &format!("127.0.0.1:{port}")),
        "debug" => debug(receiver, &"debug!".to_string()),
        _ => {
            panic!("Not handled")
        }
    };

    KeybdKey::bind_all(move |event| {
        println!("{:?}", &event);
        key_sender
            .send(format!("{:?}", event))
            .expect("Unable to send keyboard key");
    });

    MouseButton::bind_all(move |event| {
        println!("{:?}", &event);
        mouse_sender
            .send(format!("{:?}", event))
            .expect("Unable to send mouse button");
    });

    inputbot::handle_input_events();
}

fn websocket(r: Receiver<String>, address: &String) -> JoinHandle<()> {
    let (mut socket, response) =
        tungstenite::connect(url::Url::parse(address.as_str()).unwrap()).unwrap();

    thread::spawn(move || loop {
        match r.recv() {
            Ok(m) => socket
                .write_message(tungstenite::Message::Text(m))
                .expect("Unable to write to websocket"),
            Err(e) => panic!("{e}"),
        };
    })
}

fn udp(r: Receiver<String>, address: &String) -> JoinHandle<()> {
    let socket = UdpSocket::bind(address).unwrap();

    thread::spawn(move || loop {
        match r.recv() {
            Ok(m) => socket.send(m.as_bytes()).unwrap(),
            Err(e) => panic!("{e}"),
        };
    })
}

fn debug(r: Receiver<String>, address: &String) -> JoinHandle<()> {
    thread::spawn(move || loop {
        match r.recv() {
            Ok(m) => println!("{m}"),
            Err(e) => panic!("{e}"),
        };
    })
}
