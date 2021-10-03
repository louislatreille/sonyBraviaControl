# sonyBraviaControl
This small utility program allows to control a sony BRAVIA smart TV with your keyboard.

## How to use
- Download the latest version of the software for your OS (Windows or Linux)
- Create a configuration file named `sony_bravia_control.ini` and add it to your home directory
- Add your TV address and port to the ini file, such as `tv_address = 192.168.1.106:20060`
- Start the controller

### Supported commands
F1: Power off<br/>
F2: Power on<br/>
<br/>
W/Up arrow: Up<br/>
S/Down arrow: Down<br/>
A/Left arrow: Left<br/>
D/Right arrow: Right<br/>
<br/>
H: Home<br/>
N: Netflix<br/>
<br/>
1: HDMI 1<br/>
2: HDMI 2<br/>
3: HDMI 3<br/>
4: HDMI 4<br/>
<br/>
Enter: Enter<br/>
Backspace: Back<br/>

## Implementation
The utility communicates with the TV with a TCP socket. It sends BRAVIA's simple IP control commands, that can be found [here](https://pro-bravia.sony.net/develop/integrate/ssip/command-definitions/index.html).

The keybind listening is done with [InputBot](https://github.com/obv-mikhail/InputBot)

## Building
To build for windows: `cargo build --target x86_64-pc-windows-gnu --release`<br/>
To build for linux: `cargo build --target x86_64-unknown-linux-gnu --release`<br/>