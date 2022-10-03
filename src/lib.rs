use std::thread::{self, JoinHandle};

use crossbeam_channel::{bounded, Receiver, Sender};
use gdnative::prelude::*;
use inputbot::{KeybdKey, MouseButton};

const INPUT_RECEIVED: &str = "input_received";

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register)]
struct InputForwarder {
    join_handle: JoinHandle<()>,
    receiver: Receiver<String>,
}

#[methods]
impl InputForwarder {
    fn new(_o: &Node) -> Self {
        let (sender, receiver) = bounded::<String>(1);

        let k_s = sender.clone();
        KeybdKey::bind_all(move |event| {
            k_s.send(format!("{:?}", event))
                .expect("Unable to send keyboard key");
        });

        let m_s = sender.clone();
        MouseButton::bind_all(move |event| {
            m_s.send(format!("{:?}", event))
                .expect("Unable to send mouse button");
        });

        let handle: JoinHandle<()> = thread::spawn(move || loop {
            inputbot::handle_input_events();
        });

        InputForwarder {
            join_handle: handle,
            receiver: receiver,
        }
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder.signal(INPUT_RECEIVED).done();
    }

    #[method]
    fn _process(&self, #[base] owner: &Node, delta: f32) {
        let input = self.poll();
        if !input.is_empty() {
            owner.emit_signal(INPUT_RECEIVED, &[input.to_variant()]);
        }
    }

    fn poll(&self) -> GodotString {
        if self.receiver.is_empty() {
            return "".into();
        }
        match self.receiver.recv() {
            Ok(m) => GodotString::from(m.as_str()),
            Err(e) => GodotString::from(e.to_string()),
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<InputForwarder>();
}

godot_init!(init);
