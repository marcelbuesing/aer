use std::collections::HashMap;
use embedded_weather_icons as icons;
use crate::*;
use log::*;
use tinybmp::Bmp;
use lazy_static::lazy_static;

// use embedded_graphics::{pixelcolor::BinaryColor, DrawTarget, image::{Image, ImageRaw}, pixelcolor::Rgb888, prelude::*, drawable::{Drawable, Pixel}};
use embedded_graphics::{
    drawable::{Drawable, Pixel},
    geometry::{Dimensions, Point, Size},
    image::Image,
    mock_display::MockDisplay,
    pixelcolor::{BinaryColor, Gray8, GrayColor, Rgb555, Rgb565, Rgb888, RgbColor},
    primitives::Rectangle,
    transform::Transform,
};


lazy_static! {
    pub static ref WEATHER_ICONS: HashMap<u32, Bmp<'static>> = {
        let mut m = HashMap::new();
        m.insert(200, icons::wi_thunderstorm_32x32().unwrap());
        m.insert(800, icons::wi_day_sunny_32x32().unwrap());
        m.insert(801, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(802, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(803, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(804, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(900, icons::wi_tornado_32x32().unwrap());
        m
    };
}

pub fn weather<T: DrawTarget<BinaryColor>>(display: &mut T) {
    debug!("Weather report current");
    let weather = match openweather::get_current_weather(
        &WEATHER_LOCATION,
        &OPENWEATHER_API_KEY,
        &OPENWEATHER_SETTINGS,
    ) {
        Ok(weather) => weather,
        Err(e) => {
            error("Getting Weather", e);
            return;
        }
    };
    info!(
        "In {}, {} it is {}°C",
        weather.name, weather.sys.country, weather.main.temp
    );
    draw_temp(display, weather.main.temp);

    #[cfg(feature = "epd4in2")]
    weather_forecast(display, weather.main.temp);

    sunrise_and_sunset(
        display,
        weather.sys.sunrise as i64,
        weather.sys.sunset as i64,
    );
}

fn sunrise_and_sunset<T: DrawTarget<BinaryColor>>(display: &mut T, sunrise: i64, sunset: i64) {
    // Construct a datetime from epoch:
    let sunrise: DateTime<Local> = Utc.timestamp(sunrise as i64, 0).into();
    let sunset: DateTime<Local> = Utc.timestamp(sunset as i64, 0).into();
    // println!("{}", sunrise.to_rfc2822());
    // println!("{}", sunset.to_rfc2822());
    //assert_eq!(dt.to_rfc2822(), "Fri, 14 Jul 2017 02:40:00 +0000");

    draw_sunset(display, sunrise, sunset);
}

#[cfg(feature = "epd4in2")]
fn draw_temp<T: DrawTarget<BinaryColor>>(display: &mut T, temp: f32) {
    text_24x32(
        display,
        &format!("{:5.1}°C", temp),
        (width() - 7 * 24, 100).into(),
    );
}

#[cfg(feature = "epd2in9")]
fn draw_temp<T: DrawTarget<BinaryColor>>(display: &mut T, temp: f32) {
    text_24x32(
        display,
        &format!("{:5.1}°", temp),
        (width() - 6 * 24, height() - 32).into(),
    );
}

#[cfg(feature = "epd4in2")]
fn draw_sunset<T: DrawTarget<BinaryColor>>(
    display: &mut T,
    sunrise: DateTime<Local>,
    sunset: DateTime<Local>,
) {
    text_12x16(
        display,
        &format!(
            "{:2}:{:02} | {:2}:{:02}",
            sunrise.hour(),
            sunrise.minute(),
            sunset.hour(),
            sunset.minute()
        ),
        (width() / 2 - 90i32, 0).into(),
    );
}

#[cfg(feature = "epd2in9")]
fn draw_sunset<T: DrawTarget<BinaryColor>>(
    display: &mut T,
    sunrise: DateTime<Local>,
    sunset: DateTime<Local>,
) {
    text_6x12(
        display,
        &format!(
            "{:2}:{:02} | {:2}:{:02}",
            sunrise.hour(),
            sunrise.minute(),
            sunset.hour(),
            sunset.minute()
        ),
        (width() / 2 - 40i32, 0).into(),
    );
}
