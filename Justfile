# List of all recipes
default:
    @just --list

# Arduino Uno: Led 13 blink
uno-ledbl: (uno-build "led-blink")

# Arduino Uno: PS2 dualshock gamepad
uno-ps2gp: (uno-build "ps2gamepad")

[private]
uno-build APP: (ard-build APP "uno" "115200")
[private]
ard-build APP BOARD BR:
    cargo build --release --bin {{APP}}
    ravedude -cb {{BR}} {{BOARD}} target/avr-atmega328p/release/{{APP}}.elf
