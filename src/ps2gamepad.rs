#![no_std]
#![no_main]
use emdl_ps2device as ps2device;

use arduino_hal::prelude::*;
use panic_halt as _;
use ps2device::prelude::*;

const UART_BRATE: u32 = 115200; // [115200, 57600]

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, UART_BRATE);

    ufmt::uwriteln!(
        &mut serial,
        "Firmware: {}!\nCrate: {} v{}!",
        file!(),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    ).unwrap();

    arduino_hal::delay_ms(200);
    // gamepad
    let mut gamepad = create_psx_controller(pins.d7.into_pull_up_input(),
        pins.d6.into_output(),
        pins.d5.into_output(),
        pins.d4.into_output(),
        arduino_hal::Delay::new(),
    );

    gamepad.connect();
    {
        let str_gp_type: &str = match gamepad.ctype {
            Ps2DeviceType::Unknown => "Unknown",
            Ps2DeviceType::DualShock1 => "DualShock1",
            Ps2DeviceType::DualShock2 => "DualShock2",
            // Ps2DeviceType::GuitarHero => "GuitarHero",
        };
        let str_gp_state: &str = match gamepad.state {
            Ps2DeviceState::Connected => "connected",
            _ => "connection error",
        };
        let str_gp_mode: &str = match gamepad.is_analog_led {
            true => "Analog",
            _ => "Digital",
        };
        arduino_hal::delay_ms(10);
        ufmt::uwriteln!(
            &mut serial,
            "Gamepad {} {} ({})",
            str_gp_type,
            str_gp_state,
            str_gp_mode
        )
        .unwrap();
        ufmt::uwrite!(&mut serial, "unknown1: 0x ").unwrap();
        aux::write_hex(&mut serial, &gamepad.info.unknown1[..], " ");
        ufmt::uwrite!(&mut serial, "\nunknown2: 0x ").unwrap();
        aux::write_hex(&mut serial, &gamepad.info.unknown2[..], " ");
        ufmt::uwrite!(&mut serial, "\nunknown3: 0x ").unwrap();
        aux::write_hex(&mut serial, &gamepad.info.unknown3[..], " ");
        ufmt::uwrite!(&mut serial, "\n");
    }

    ufmt::uwriteln!(&mut serial, "Start polling...").unwrap();

    loop {
        arduino_hal::delay_ms(10);
        let state = gamepad.state;
        gamepad.poll();
        if state != gamepad.state {
            ufmt::uwriteln!(&mut serial, "Connection: {}", gamepad.state == Ps2DeviceState::Connected);
        }

        if gamepad.is_down(Ps2Button::Select) {
            ufmt::uwriteln!(&mut serial, "Select down");
        }
        if gamepad.is_up(Ps2Button::LJoyBtn) {
            ufmt::uwriteln!(&mut serial, "LJoyBtn up");
        }
        if gamepad.is_pressed(Ps2Button::RJoyBtn) {
            ufmt::uwriteln!(&mut serial, "RJoyBtn pressed");
        }
        if gamepad.is_changed(Ps2Button::Start) {
            ufmt::uwriteln!(&mut serial, "Start changed");
        }

        if gamepad.is_down(Ps2Button::Up) {
            ufmt::uwriteln!(&mut serial, "Up down");
        }
        if gamepad.is_up(Ps2Button::Right) {
            ufmt::uwriteln!(&mut serial, "Right up");
        }
        if gamepad.is_pressed(Ps2Button::Down) {
            ufmt::uwriteln!(&mut serial, "Down pressed");
        }
        if gamepad.is_changed(Ps2Button::Left) {
            ufmt::uwriteln!(&mut serial, "Left changed");
        }

        if gamepad.is_down(Ps2Button::LTrigger) {
            ufmt::uwriteln!(&mut serial, "LTrigger down");
        }
        if gamepad.is_up(Ps2Button::RTrigger) {
            ufmt::uwriteln!(&mut serial, "RTrigger up");
        }
        if gamepad.is_pressed(Ps2Button::LButton) || gamepad.is_pressed(Ps2Button::RButton) {
            let a = gamepad.analog_sticks();
            ufmt::uwriteln!(
                &mut serial,
                "Analog = [Lx: {}, Ly: {}, Rx: {}, Ry: {}]",
                a.lx,
                a.ly,
                a.rx,
                a.ry
            );
        }
        if gamepad.is_changed(Ps2Button::RButton) {
            ufmt::uwriteln!(&mut serial, "RButton changed");
        }

        if gamepad.is_down(Ps2Button::Square) {
            ufmt::uwriteln!(&mut serial, "Square down");
        }
        if gamepad.is_up(Ps2Button::Cross) {
            ufmt::uwriteln!(&mut serial, "Cross up");
        }
        if gamepad.is_pressed(Ps2Button::Circle) {
            ufmt::uwriteln!(&mut serial, "Circle pressed");
        }
        if gamepad.is_changed(Ps2Button::Triangle) {
            ufmt::uwriteln!(&mut serial, "Triangle changed");
        }
    }
}

mod aux {
    pub fn write_hex<S: ufmt::uWrite>(serial: &mut S, bytes: &[u8], delim: &str) {
        if bytes.len() == 0 {
            return;
        }
        write_hex_byte(serial, bytes[0]);
        for i in 1..bytes.len() {
            ufmt::uwrite!(serial, "{}", delim);
            write_hex_byte(serial, bytes[i]);
        }
    }

    pub fn write_hex_byte<S: ufmt::uWrite>(serial: &mut S, b: u8) {
        let (h, l) = (hex_char(b >> 4), hex_char(b));
        ufmt::uwrite!(serial, "{}{}", h, l);
    }

    pub fn hex_char(b: u8) -> char {
        match b & 0x0f {
            0x0 => '0',
            0x1 => '1',
            0x2 => '2',
            0x3 => '3',
            0x4 => '4',
            0x5 => '5',
            0x6 => '6',
            0x7 => '7',
            0x8 => '8',
            0x9 => '9',
            0xa => 'a',
            0xb => 'b',
            0xc => 'c',
            0xd => 'd',
            0xe => 'e',
            0xf => 'f',
            _ => '?',
        }
    }
}
