use crate::*;
use embedded_weather_icons as icons;
use lazy_static::lazy_static;
use log::*;
use std::collections::HashMap;
use tinybmp::Bmp;

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
        m.insert(201, icons::wi_thunderstorm_32x32().unwrap());
        m.insert(202, icons::wi_thunderstorm_32x32().unwrap());
        m.insert(210, icons::wi_lightning_32x32().unwrap());
        m.insert(211, icons::wi_lightning_32x32().unwrap());
        m.insert(212, icons::wi_lightning_32x32().unwrap());
        m.insert(221, icons::wi_lightning_32x32().unwrap());
        m.insert(230, icons::wi_thunderstorm_32x32().unwrap());
        m.insert(231, icons::wi_thunderstorm_32x32().unwrap());
        m.insert(232, icons::wi_thunderstorm_32x32().unwrap());
        m.insert(300, icons::wi_sprinkle_32x32().unwrap());
        m.insert(301, icons::wi_sprinkle_32x32().unwrap());
        m.insert(302, icons::wi_rain_32x32().unwrap());
        m.insert(310, icons::wi_rain_32x32().unwrap());
        m.insert(311, icons::wi_rain_32x32().unwrap());
        m.insert(312, icons::wi_rain_32x32().unwrap());
        m.insert(313, icons::wi_rain_32x32().unwrap());
        m.insert(314, icons::wi_rain_32x32().unwrap());
        m.insert(321, icons::wi_sprinkle_32x32().unwrap());
        m.insert(500, icons::wi_sprinkle_32x32().unwrap());
        m.insert(501, icons::wi_rain_32x32().unwrap());
        m.insert(502, icons::wi_rain_32x32().unwrap());
        m.insert(503, icons::wi_rain_32x32().unwrap());
        m.insert(504, icons::wi_rain_32x32().unwrap());
        m.insert(511, icons::wi_rain_mix_32x32().unwrap());
        m.insert(520, icons::wi_showers_32x32().unwrap());
        m.insert(521, icons::wi_showers_32x32().unwrap());
        m.insert(522, icons::wi_showers_32x32().unwrap());
        m.insert(531, icons::wi_storm_showers_32x32().unwrap());
        m.insert(600, icons::wi_snow_32x32().unwrap());
        m.insert(601, icons::wi_sleet_32x32().unwrap());
        m.insert(602, icons::wi_snow_32x32().unwrap());
        m.insert(611, icons::wi_rain_mix_32x32().unwrap());
        m.insert(612, icons::wi_rain_mix_32x32().unwrap());
        m.insert(615, icons::wi_rain_mix_32x32().unwrap());
        m.insert(616, icons::wi_rain_mix_32x32().unwrap());
        m.insert(620, icons::wi_rain_mix_32x32().unwrap());
        m.insert(621, icons::wi_snow_32x32().unwrap());
        m.insert(622, icons::wi_snow_32x32().unwrap());
        m.insert(701, icons::wi_showers_32x32().unwrap());
        m.insert(711, icons::wi_smoke_32x32().unwrap());
        m.insert(721, icons::wi_day_haze_32x32().unwrap());
        m.insert(731, icons::wi_dust_32x32().unwrap());
        m.insert(741, icons::wi_fog_32x32().unwrap());
        m.insert(761, icons::wi_dust_32x32().unwrap());
        m.insert(762, icons::wi_dust_32x32().unwrap());
        m.insert(781, icons::wi_tornado_32x32().unwrap());
        m.insert(800, icons::wi_day_sunny_32x32().unwrap());
        m.insert(801, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(802, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(803, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(804, icons::wi_cloudy_gusts_32x32().unwrap());
        m.insert(804, icons::wi_day_sunny_overcast_32x32().unwrap());
        m.insert(900, icons::wi_tornado_32x32().unwrap());
        m.insert(901, icons::wi_storm_showers_32x32().unwrap());
        m.insert(902, icons::wi_hurricane_32x32().unwrap());
        m.insert(903, icons::wi_snowflake_cold_32x32().unwrap());
        m.insert(904, icons::wi_hot_32x32().unwrap());
        m.insert(905, icons::wi_windy_32x32().unwrap());
        m.insert(906, icons::wi_hail_32x32().unwrap());
        m.insert(957, icons::wi_strong_wind_32x32().unwrap());
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
    draw_temp(
        display,
        weather.main.temp,
        weather.weather.get(0).unwrap().id,
    );

    #[cfg(any(feature = "epd4in2", feature = "epd7in5"))]
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

#[cfg(any(feature = "epd4in2", feature = "epd7in5"))]
fn draw_temp<T: DrawTarget<BinaryColor>>(display: &mut T, temp: f32, weather_id: u32) {
    let image: &Bmp<'static> = weather::WEATHER_ICONS.get(&weather_id).unwrap();
    Image::new(image, Point::zero())
        .translate((width() - 7 * 24 - 26, 110).into())
        .into_iter()
        .map(|Pixel(p, c)| {
            Pixel(
                p,
                match c {
                    Rgb565::WHITE => BinaryColor::Off,
                    Rgb565::BLACK => BinaryColor::On,
                    _ => panic!("Unexpected color in image"),
                },
            )
        })
        .draw(display);

    text_24x32(
        display,
        &format!("{:5.1}°C", temp),
        (width() - 7 * 24, 110).into(),
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

#[cfg(any(feature = "epd4in2", feature = "epd7in5"))]
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
