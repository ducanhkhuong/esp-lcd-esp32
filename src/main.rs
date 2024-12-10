#![no_std]
#![no_main]

use esp_hal::{
    delay::Delay,
    gpio::{self, Io, Level, Output},
    prelude::*,
    spi::{master::Spi, SpiMode},
};

extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;


use mipidsi::{
    models::GC9A01, options::{ColorInversion, ColorOrder}, Builder
};


use display_interface_spi::SPIInterface;
use embedded_hal_bus::spi::ExclusiveDevice;

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::{
        raw::BigEndian,
        Rgb565
    },
    prelude::*,
};

mod asset;
   

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);
    let mut delay = Delay::new();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let dc = Output::new(io.pins.gpio27, Level::Low);
    let sck = io.pins.gpio14;
    let miso = io.pins.gpio12;
    let mosi = io.pins.gpio15;
    let cs = io.pins.gpio5;

    //80 MHz
    let spi = Spi::new(peripherals.SPI2, 50_000_000_u32.Hz(), SpiMode::Mode0).with_pins(
        sck,
        mosi,
        miso,
        gpio::NoPin,
    );

    let cs_output = Output::new(cs, Level::High);
    let mut rst = Output::new(io.pins.gpio33, Level::Low);
    rst.set_high();

    let spi_device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();
    let di = SPIInterface::new(spi_device, dc);
    
    let mut display: mipidsi::Display<
        SPIInterface<ExclusiveDevice<_, Output<'_>, embedded_hal_bus::spi::NoDelay>, Output<'_>>,
        GC9A01,
        Output<'_>,
    > = Builder::new(GC9A01, di)
        .display_size(240, 240)
        .reset_pin(rst)
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();

    //display.clear(Rgb565::RED).unwrap();

    let raw_image1 = ImageRaw::<Rgb565, BigEndian>::new(asset::img1::img1::DATA, 240);
    let raw_image2 = ImageRaw::<Rgb565, BigEndian>::new(asset::img2::img2::DATA, 240);

    let image1 = Image::new(&raw_image1, Point::zero());
    let image2 = Image::new(&raw_image2, Point::zero());

    loop{
        image1.draw(&mut display).unwrap();
        delay.delay_millis(3000);
        image2.draw(&mut display).unwrap();
        delay.delay_millis(3000);
    }
 //loop {}
}
