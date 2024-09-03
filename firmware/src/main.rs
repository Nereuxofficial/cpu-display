//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use core::{cell::RefCell, ptr::addr_of_mut, str};

use bsp::entry;
use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::Point,
    text::Text,
};
use hub75_pio::lut::GammaLut;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico::{
    self as bsp,
    hal::{dma::DMAExt, pio::PIOExt, reset},
    pac::Interrupt,
};
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

static mut DISPLAY_BUFFER: hub75_pio::DisplayMemory<64, 32, 12> = hub75_pio::DisplayMemory::new();
static COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0u32));

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let (mut pio, sm0, sm1, sm2, _) = pac.PIO0.split(&mut pac.RESETS);

    let resets = pac.RESETS;
    resets.reset.modify(|_, w| w.dma().set_bit());
    resets.reset.modify(|_, w| w.dma().clear_bit());
    while resets.reset_done.read().dma().bit_is_clear() {}

    let dma = &pac.DMA;
    dma.inte0.write(|w| unsafe { w.bits(1 << 0) });

    let dma = pac.DMA.split(&mut pac.RESETS);

    // Unmask the IO_BANK0 IRQ so that the NVIC interrupt controller
    // will jump to the interrupt function when the interrupt occurs.
    // We do this last so that the interrupt can't go off while
    // it is in the middle of being configured
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::DMA_IRQ_0);
    }

    let lut = {
        let lut: GammaLut<12, _, _> = GammaLut::new();
        lut.init((2.1, 2.1, 2.1))
    };
    let mut display = unsafe {
        hub75_pio::Display::new(
            addr_of_mut!(DISPLAY_BUFFER).as_mut().unwrap(),
            hub75_pio::DisplayPins {
                r1: pins.gpio2.into_function().into_pull_type().into_dyn_pin(),
                g1: pins.gpio3.into_function().into_pull_type().into_dyn_pin(),
                b1: pins.gpio4.into_function().into_pull_type().into_dyn_pin(),
                r2: pins.gpio6.into_function().into_pull_type().into_dyn_pin(),
                g2: pins.gpio7.into_function().into_pull_type().into_dyn_pin(),
                b2: pins.gpio8.into_function().into_pull_type().into_dyn_pin(),
                addra: pins.gpio11.into_function().into_pull_type().into_dyn_pin(),
                addrb: pins.gpio12.into_function().into_pull_type().into_dyn_pin(),
                addrc: pins.gpio13.into_function().into_pull_type().into_dyn_pin(),
                addrd: pins.gpio14.into_function().into_pull_type().into_dyn_pin(),
                clk: pins.gpio15.into_function().into_pull_type().into_dyn_pin(),
                lat: pins.gpio16.into_function().into_pull_type().into_dyn_pin(),
                oe: pins.gpio17.into_function().into_pull_type().into_dyn_pin(),
            },
            &mut pio,
            (sm0, sm1, sm2),
            (dma.ch0, dma.ch1, dma.ch2, dma.ch3),
            false,
            &lut,
        )
    };
    let style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
    let mut last_value = 0;
    let mut hz = 0;

    loop {
        let mut buf = [0u8; 20];
        let buf = hz_to_str(hz, &mut buf);
        let buf = str::from_utf8(&buf).unwrap();
        Text::new(buf, Point::new(12, 19), style)
            .draw(&mut display)
            .unwrap();
        display.commit();
        delay.delay_ms(1000);

        let counter = critical_section::with(|cs| *COUNTER.borrow_ref_mut(cs));
        if counter > last_value {
            hz = counter - last_value;
        }
        last_value = counter;
    }
}

#[interrupt]
fn DMA_IRQ_0() {
    critical_section::with(|cs| {
        COUNTER.replace_with(cs, |counter| (*counter + 1) % 100000000);
    });
    // Clear the DMA interrupt flag
    const INTS: *mut u32 = (0x50000000 + 0x40c) as *mut u32;
    unsafe { ptr::write_volatile(INTS, 0b1) };
}

fn hz_to_str(mut n: u32, buf: &mut [u8]) -> &[u8] {
    if n == 0 {
        return b"0";
    }
    let mut i = 3;
    while n > 0 {
        buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
    }
    buf[0] = 'z' as u8;
    buf[1] = 'H' as u8;
    buf[2] = ' ' as u8;
    let slice = &mut buf[..i];
    slice.reverse();
    &*slice
}
