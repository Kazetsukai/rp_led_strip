#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

mod state;
mod usb_device;
mod ws2812b;

const NUM_LEDS: usize = 391;

use {
    core::iter::repeat,
    defmt::*,
    defmt_rtt as _,
    embassy_executor::Spawner,
    embassy_rp::{
        bind_interrupts,
        dma::{AnyChannel, Channel},
        gpio::{AnyPin, Level, Output},
        peripherals::{DMA_CH0, PIN_14, PIN_16, PIN_23, PIN_5, PIN_8, PIN_9, PIO0, USB},
        pio::{self, Instance, Pio, PioPin},
        usb::{self, Driver},
        Peripheral,
    },
    embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex},
    embassy_time::{Duration, Ticker, Timer},
    panic_probe as _,
    smart_leds::{brightness, gamma, RGB8},
    state::{AppState, LedControls, SharedState},
    static_cell::make_static,
    ws2812b::Ws2812,
};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Starting up");

    let atx_ps_on_pin = p.PIN_8;
    let onboard_led_pin = p.PIN_23;
    let led_strip_pin = p.PIN_14;

    let shared_state = make_static!(state::SharedState(make_static!(Mutex::new(
        state::LedControls {
            color: RGB8::default().into(),
            power: false,
        }
    ))));

    let led_colors = make_static!(Mutex::new([RGB8::default(); NUM_LEDS]));

    Timer::after_millis(100).await;

    spawner
        .spawn(heartbeat(
            p.PIO0,
            p.DMA_CH0.into(),
            led_strip_pin,
            shared_state,
            led_colors,
        ))
        .unwrap();

    spawner
        .spawn(usb_device::be_usb_device(
            spawner,
            p.USB,
            shared_state,
            led_colors,
        ))
        .unwrap();

    let _atx_ps_on = Output::new(atx_ps_on_pin, Level::Low);

    let mut ticker = Ticker::every(Duration::from_millis(1000));
    loop {
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn heartbeat(
    pio: PIO0,
    dma: AnyChannel,
    pin: PIN_14,
    shared_state: &'static state::SharedState,
    led_colors: &'static Mutex<CriticalSectionRawMutex, [RGB8; NUM_LEDS]>,
) {
    let Pio {
        mut common, sm0, ..
    } = Pio::new(pio, Irqs);
    let mut ws2812 = Ws2812::new(&mut common, sm0, dma, pin);

    // Loop forever making RGB values and pushing them out to the WS2812.
    let mut ticker = Ticker::every(Duration::from_millis(10));
    loop {
        /*let SharedState(leds) = shared_state;
        let LedControls { power, color } = *leds.lock().await;
        let rgb: RGB8 = color.into();
        if power {
            gamma(brightness(
                repeat(RGB8::new(rgb.r, rgb.g, rgb.b)).take(NUM_LEDS),
                j + 30,
            ))
            .enumerate()
            .for_each(|(i, d)| data[i] = d);
        } else {
            repeat(RGB8::default())
                .take(NUM_LEDS)
                .enumerate()
                .for_each(|(i, d)| data[i] = d);
        }*/

        let mut corrected_colors = [RGB8::default(); NUM_LEDS];
        {
            let raw_colors = *led_colors.lock().await;
            for (i, color) in gamma(raw_colors.into_iter()).enumerate() {
                corrected_colors[i] = color;
            }
        }
        ws2812.write(&corrected_colors).await;

        ticker.next().await;
    }
}
