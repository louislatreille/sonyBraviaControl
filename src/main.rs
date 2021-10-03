use sony_bravia_control::TvCommandsManager;
use std::net::SocketAddr;
use configparser::ini::Ini;

fn main() {
    let mut home_dir = home::home_dir().expect("Impossible to get your home dir!");
    home_dir.push("sony_bravia_control.ini");

    let mut config = Ini::new();
    match config.load(home_dir) {
        Ok(_) => (),
        Err(_) => eprintln!("Couldn't load the configuration from your home directory.")
    }

    let tv_address = config.get("default", "tv_address").expect("No tv_address configuration found in the ini file.");

    let socket_addr: SocketAddr = tv_address.parse().expect("Misspelled tv address. It must be a IPv4 addresse followed by a port, such as 192.168.1.1:20060");
    let tv_commands_manager = TvCommandsManager::new(socket_addr);
    tv_commands_manager.start();
}