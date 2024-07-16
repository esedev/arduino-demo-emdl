#![no_std]
#![no_main]

use arduino_hal::adc;
use arduino_hal::prelude::*;
use avr_heel_motors::brushed::{OutputMotorPwm, SmoothMotorSignal, Wire2 as DcMotorWire};
use avr_heel_motors::commutator_motor::pwm::{
    smooth_signal, MotorDriver, MotorSignal, SmoothDefault,
};
use core::convert::Infallible;
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;

const UART_BRATE: u32 = 57600; // [115200, 57600]

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, UART_BRATE);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    {
        ufmt::uwriteln!(
            &mut serial,
            "Firmware: {}!\nCrate: {} v{}!",
            file!(),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )
        .void_unwrap();

        let (vbg, gnd, tmp) = (
            adc.read_blocking(&adc::channel::Vbg),
            adc.read_blocking(&adc::channel::Gnd),
            adc.read_blocking(&adc::channel::Temperature),
        );
        ufmt::uwriteln!(&mut serial, "Vbandgap: {}", vbg).void_unwrap();
        ufmt::uwriteln!(&mut serial, "Ground: {}", gnd).void_unwrap();
        ufmt::uwriteln!(&mut serial, "Temperature: {}", tmp).void_unwrap();
    }
    arduino_hal::delay_ms(200);

    let pedal = pins.a0.into_analog_input(&mut adc);
    let mut led = pins.d13.into_output();
    led.set_high();

    // pwm pins
    let pwm_mt = {
        let tc2 = dp.TC2;
        tc2.tccr2a.write(|w| {
            w.wgm2()
                .pwm_fast()
                .com2a()
                .match_clear()
                .com2b()
                .match_clear()
        });
        tc2.tccr2b.write(|w| w.cs2().prescale_1024());
        tc2.ocr2b.write(|w| unsafe { w.bits(0x00u8) }); // Pwm3.write()  (Left)
        tc2.ocr2a.write(|w| unsafe { w.bits(0x00u8) }); // Pwm11.write() (Right)
        tc2
    };
    pins.d3.into_output();
    pins.d11.into_output();

    let mut motor = DcMotorWire::new(Pwm3(&pwm_mt), pins.d9.into_output());
    let mut smooth = smooth_signal(16);
    loop {
        let gas = pedal.analog_read(&mut adc) >> 1;
        let signal = match gas {
            v if v >= 280 => MotorSignal::Rotate((gas -1 - u8::MAX as u16) as u8),
            v if v <= 230 => MotorSignal::RotateBack(u8::MAX - gas as u8),
            _ => MotorSignal::Stop,
        };
        let s: i16 = match signal {
            MotorSignal::Rotate(v) => v as i16,
            MotorSignal::RotateBack(v) => -(v as i16),
            MotorSignal::Stop => 0,
            MotorSignal::Block => 1000, 
        };
        ufmt::uwriteln!(&mut serial, "A: {}, S: {}", gas, s).void_unwrap();
        smooth.assign(signal);

        motor.motor_signal(smooth.next());
        arduino_hal::delay_ms(10);
        led.toggle();
    }
}

fn map_pwm(v: u8, lim: (u8, u8)) -> u8 {
    match v {
        0 => 0,
        _ => lim.0 + ((v as u16) * (lim.1 as u16 - lim.0 as u16) / (lim.1 as u16)) as u8,
    }
}

struct Pwm3<'a>(&'a arduino_hal::pac::TC2);
impl<'a> OutputMotorPwm for Pwm3<'a> {
    fn set_power(&self, val: u8) {
        self.0.ocr2b.write(|w| unsafe { w.bits(val) })
    }
}
