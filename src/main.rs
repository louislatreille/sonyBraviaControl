use sony_bravia_control::KeyboardInputManager;

fn main() {
    let keyboard_input = KeyboardInputManager::new();
    KeyboardInputManager::start(keyboard_input);
}