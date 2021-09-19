# sonyBraviaControl
This small utility program allows to control a sony BRAVIA smart TV with your keyboard.

## Implementation
The utility communicates with the TV with a TCP socket. It sends BRAVIA's simple IP control commands, that can be found [here](https://pro-bravia.sony.net/develop/integrate/ssip/command-definitions/index.html).

The keybind listening is done with [InputBot](https://github.com/obv-mikhail/InputBot)

## Building
To build for windows: `cargo build --target x86_64-pc-windows-gnu`