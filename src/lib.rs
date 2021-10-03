use std::io::Write;
use std::thread;
use std::sync::mpsc;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::sync::Mutex;
use inputbot::{KeybdKey::*, handle_input_events};

pub struct TvCommandBinding {
    name: String,
    command_bytes: [u8; 24],
    key: inputbot::KeybdKey,
    tv_commands_sender: mpsc::Sender<[u8; 24]>
}

impl TvCommandBinding {
    fn new(name: String, command_bytes: [u8; 24], key: inputbot::KeybdKey, tv_commands_sender: mpsc::Sender<[u8; 24]>) -> TvCommandBinding {
        TvCommandBinding {
            name,
            command_bytes,
            key,
            tv_commands_sender
        }
    }

    pub fn create_command_dispatcher(&self) -> Box<dyn Fn() -> () + Send + Sync> {
        let cloned_sender = Mutex::new(self.tv_commands_sender.clone());
        let cloned_name = self.name.clone();
        let cloned_bytes = self.command_bytes.clone();

        let commands_dispatcher = move || {
            println!("Sending command {}", cloned_name);

            match cloned_sender.lock() {
                Ok(sender) => {
                    match sender.send(cloned_bytes) {
                        Ok(_) => (),
                        Err(_) => { println!("Error sending command {}. Receiving end likely hung up.", cloned_name) },
                    }
                },
                Err(_) => { println!("Error acquiring lock to send command {}", cloned_name) }
            }
        };

        Box::new(commands_dispatcher)
    }
}

impl Clone for TvCommandBinding {
    fn clone(&self) -> Self {
        TvCommandBinding {
            name: self.name.clone(),
            command_bytes: self.command_bytes.clone(),
            key: self.key.clone(),
            tv_commands_sender: self.tv_commands_sender.clone()
        }
    }
}

pub struct TvCommandsManager {}

impl TvCommandsManager {
    pub fn new(tv_address: SocketAddr) -> TvCommandsManager {
        let (tv_commands_sender, tv_commands_receiver): (mpsc::Sender<[u8; 24]>, mpsc::Receiver<[u8; 24]>) = mpsc::channel();

        let key_bindings = [
            TvCommandBinding::new(String::from("powerOff"), *b"*SCPOWR0000000000000000\n", F1Key, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("powerOn"), *b"*SCPOWR0000000000000001\n", F2Key, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("home"), *b"*SCIRCC0000000000000006\n", HKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("netflix"), *b"*SCIRCC0000000000000056\n", NKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("up"), *b"*SCIRCC0000000000000009\n", UpKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("up"), *b"*SCIRCC0000000000000009\n", WKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("down"), *b"*SCIRCC0000000000000010\n", DownKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("down"), *b"*SCIRCC0000000000000010\n", SKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("right"), *b"*SCIRCC0000000000000011\n", DKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("right"), *b"*SCIRCC0000000000000011\n", RightKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("left"), *b"*SCIRCC0000000000000012\n", AKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("left"), *b"*SCIRCC0000000000000012\n", LeftKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("enter"), *b"*SCIRCC0000000000000013\n", EnterKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("return"), *b"*SCIRCC0000000000000008\n", BackspaceKey, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("hdmi1"), *b"*SCINPT0000000100000001\n", Numrow1Key, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("hdmi2"), *b"*SCINPT0000000100000002\n", Numrow2Key, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("hdmi3"), *b"*SCINPT0000000100000003\n", Numrow3Key, tv_commands_sender.clone()),
            TvCommandBinding::new(String::from("hdmi4"), *b"*SCINPT0000000100000004\n", Numrow4Key, tv_commands_sender.clone()),
        ];

        let mut stream = match TcpStream::connect(tv_address) {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("Error when connecting to TV. Is the TV still turned on and connected to the network? Error: {}", error);
                panic!()
            }
        };         

        thread::spawn(move || loop {
            let tv_command_bytes = match tv_commands_receiver.recv() {
                Ok(bytes) => bytes,
                Err(_) => {
                    panic!("Error while listening for incoming commands. It is likely there are no more command dispatchers.")
                },
            };

            match stream.write_all(&tv_command_bytes) {
                Ok(_) => (),
                Err(error) => {
                    eprintln!("Error when trying to send command {:?} to TV. Is the TV still turned on and connected to the network? Error: {}", tv_command_bytes, error);
                    panic!()
                }
            };
        });

        let (activation_sender, activation_receiver): (mpsc::Sender<u8>, mpsc::Receiver<u8>) = mpsc::channel();   

        let activation_sender = Mutex::new(activation_sender);

        ScrollLockKey.block_bind(move || {
            match activation_sender.lock() {
                Ok(sender) => {
                    match sender.send(1) {
                        Ok(_) => (),
                        Err(_) => { println!("Error sending key bindings activation command. Receiving end likely hung up.") },
                    }
                },
                Err(_) => { println!("Error acquiring lock to send key bindings activation command") }
            }
        });

        let cloned_key_bindings = key_bindings.to_vec();

        thread::spawn(move || {
            let mut key_binds_allocated = false;

            loop {
                match activation_receiver.recv() {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Keyboard Input Manager has disconnected. Exiting...");
                        break;
                    }
                };

                key_binds_allocated = !key_binds_allocated;

                println!("Listening is {}", key_binds_allocated);
                
                if key_binds_allocated {
                    for key_binding in cloned_key_bindings.iter() {
                        key_binding.key.block_bind(key_binding.create_command_dispatcher());
                    }
                } else {
                    for key_binding in cloned_key_bindings.iter() {
                        key_binding.key.unbind();
                    }
                }
            }
        });

        TvCommandsManager {}
    }

    pub fn start(&self) {
        handle_input_events();
    }
}