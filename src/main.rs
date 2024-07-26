#![no_std]
#![no_main]

mod ws2812b;

use {
    defmt::*,
    defmt_rtt as _,
    embassy_executor::Spawner,
    embassy_rp::{
        bind_interrupts,
        gpio::{AnyPin, Level, Output},
        peripherals::PIO0,
        pio::{InterruptHandler, Pio},
    },
    embassy_time::{Duration, Ticker, Timer},
    panic_probe as _,
    smart_leds::RGB8,
    ws2812b::Ws2812,
};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common,
        sm0,
        sm1,
        ..
    } = Pio::new(p.PIO0, Irqs);

    const NUM_LEDS: usize = 10;
    let mut data = [RGB8::default(); NUM_LEDS];

    let mut onboard_pixel = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_16);
    let mut led_strip = Ws2812::new(&mut common, sm1, p.DMA_CH1, p.PIN_29);

    // Loop forever making RGB values and pushing them out to the WS2812.
    let mut ticker = Ticker::every(Duration::from_millis(10));
    loop {
        for j in 0..(256 * 5) {
            debug!("New Colors:");
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
                debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
            }
            onboard_pixel.write(&data).await;
            led_strip.write(&data).await;

            ticker.next().await;
        }
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
