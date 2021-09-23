use std::io::Write;
use std::thread;
use std::sync::mpsc;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::sync::Mutex;
use inputbot::{KeybdKey::*, handle_input_events};

pub struct TvCommand {
    name: String,
    command_bytes: [u8; 24]
}

impl TvCommand {
    fn new(name: String, command_bytes: [u8; 24]) -> TvCommand {
        TvCommand {
            name,
            command_bytes
        }
    }
}

impl Clone for TvCommand {
    fn clone(&self) -> Self {
        TvCommand {
            name: self.name.clone(),
            command_bytes: self.command_bytes.clone()
        }
    }
}

pub struct TvCommandsManager {
    commands: [TvCommand; 14],
    sender: mpsc::Sender<[u8; 24]>,
}

impl TvCommandsManager {
    pub fn new(tv_address: SocketAddr) -> TvCommandsManager {
        let commands = [
            TvCommand::new(String::from("powerOff"), *b"*SCPOWR0000000000000000\n"),
            TvCommand::new(String::from("powerOn"), *b"*SCPOWR0000000000000001\n"),
            TvCommand::new(String::from("home"), *b"*SCIRCC0000000000000006\n"),
            TvCommand::new(String::from("netflix"), *b"*SCIRCC0000000000000056\n"),
            TvCommand::new(String::from("up"), *b"*SCIRCC0000000000000009\n"),
            TvCommand::new(String::from("down"), *b"*SCIRCC0000000000000010\n"),
            TvCommand::new(String::from("right"), *b"*SCIRCC0000000000000011\n"),
            TvCommand::new(String::from("left"), *b"*SCIRCC0000000000000012\n"),
            TvCommand::new(String::from("enter"), *b"*SCIRCC0000000000000013\n"),
            TvCommand::new(String::from("return"), *b"*SCIRCC0000000000000008\n"),
            TvCommand::new(String::from("hdmi1"), *b"*SCINPT0000000100000001\n"),
            TvCommand::new(String::from("hdmi2"), *b"*SCINPT0000000100000002\n"),
            TvCommand::new(String::from("hdmi3"), *b"*SCINPT0000000100000003\n"),
            TvCommand::new(String::from("hdmi4"), *b"*SCINPT0000000100000004\n"),
        ];

        let (sender, receiver): (mpsc::Sender<[u8; 24]>, mpsc::Receiver<[u8; 24]>) = mpsc::channel();

        let mut stream = match TcpStream::connect(tv_address) {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("Error when connecting to TV. Is the TV still turned on and connected to the network? Error: {}", error);
                panic!()
            }
        };

        thread::spawn(move || loop {
            let tv_command_bytes = match receiver.recv() {
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

        TvCommandsManager {
            commands,
            sender
        }
    }

    pub fn create_commands_dispatcher(&self) -> Box<dyn Fn(&str) -> () + Send + Sync> {
        let cloned_sender = Mutex::new(self.sender.clone());
        let cloned_commands = self.commands.to_vec();

        let commands_dispatcher = move |tv_command_str: &str| {
            println!("Sending command {}", tv_command_str);

            let tv_command_bytes = match cloned_commands.iter().find(|tv_command| {
                tv_command.name == tv_command_str
            }) {
                Some(tv_command) => tv_command.command_bytes,
                None => {
                    println!("Unknown command");
                    return ()
                }
            };

            match cloned_sender.lock() {
                Ok(sender) => {
                    match sender.send(tv_command_bytes) {
                        Ok(_) => (),
                        Err(_) => { println!("Error sending command {}. Receiving end likely hung up.", tv_command_str) },
                    }
                },
                Err(_) => { println!("Error acquiring lock to send command {}", tv_command_str) }
            }
        };

        Box::new(commands_dispatcher)
    }
}

pub struct KeyboardInputManager {
    sender: Mutex<mpsc::Sender<u8>>
}

impl KeyboardInputManager {
    pub fn new() -> KeyboardInputManager {
        let tv_commands_manager = TvCommandsManager::new(SocketAddr::from(([192, 168, 10, 106], 20060)));
        let (sender, receiver): (mpsc::Sender<u8>, mpsc::Receiver<u8>) = mpsc::channel();

        thread::spawn(move || {
            let mut key_binds_allocated = false;

            loop {
                match receiver.recv() {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Keyboard Input Manager has disconnected. Exiting...");
                        break;
                    }
                };

                key_binds_allocated = !key_binds_allocated;

                println!("Listening is {}", key_binds_allocated);
                
                if key_binds_allocated {
                    KeyboardInputManager::allocate_bindings(&tv_commands_manager);
                } else {
                    KeyboardInputManager::deallocate_bindings()
                }
            }
        });

        KeyboardInputManager {
            sender: Mutex::new(sender)
        }
    }

    pub fn toggle_listening(&self) {
        match self.sender.lock() {
            Ok(sender) => {
                match sender.send(1) {
                    Ok(_) => (),
                    Err(_) => { println!("Error sending key bindings activation command. Receiving end likely hung up.") },
                }
            },
            Err(_) => { println!("Error acquiring lock to send key bindings activation command") }
        }
    }

    pub fn start(keyboard_input_manager: KeyboardInputManager) {
        ScrollLockKey.block_bind(move || {
            keyboard_input_manager.toggle_listening();
        });

        handle_input_events();
    }

    fn allocate_bindings(tv_commands_manager: &TvCommandsManager) {
        WKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("up"), tv_commands_manager));
        UpKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("up"), tv_commands_manager));
        SKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("down"), tv_commands_manager));
        DownKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("down"), tv_commands_manager));
        AKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("left"), tv_commands_manager));
        LeftKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("left"), tv_commands_manager));
        DKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("right"), tv_commands_manager));
        RightKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("right"), tv_commands_manager));
        NKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("netflix"), tv_commands_manager));
        HKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("home"), tv_commands_manager));
        F1Key.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("powerOff"), tv_commands_manager));
        F2Key.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("powerOn"), tv_commands_manager));
        EnterKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("enter"), tv_commands_manager));
        BackspaceKey.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("return"), tv_commands_manager));
        Numrow1Key.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("hdmi1"), tv_commands_manager));
        Numrow2Key.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("hdmi2"), tv_commands_manager));
        Numrow3Key.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("hdmi3"), tv_commands_manager));
        Numrow4Key.block_bind(KeyboardInputManager::create_specialized_dispatcher(String::from("hdmi4"), tv_commands_manager));
    }

    fn deallocate_bindings() {
        WKey.unbind();
        UpKey.unbind();
        SKey.unbind();
        DownKey.unbind();
        AKey.unbind();
        LeftKey.unbind();
        DKey.unbind();
        RightKey.unbind();
        NKey.unbind();
        HKey.unbind();
        F1Key.unbind();
        F2Key.unbind();
        EnterKey.unbind();
        BackspaceKey.unbind();
        Numrow1Key.unbind();
        Numrow2Key.unbind();
        Numrow3Key.unbind();
        Numrow4Key.unbind();
    }

    fn create_specialized_dispatcher(command: String, tv_command_dispatcher: &TvCommandsManager) -> Box<dyn Fn() -> () + Send + Sync> {
        let dispatcher = tv_command_dispatcher.create_commands_dispatcher();
        Box::new(move || {
            dispatcher(&command[..]);
        })
    }
}