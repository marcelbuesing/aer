use crate::*;
use anyhow::{anyhow, Result};
use embedded_weather_icons as icons;
use log::*;
use openweather::WeatherReportCurrent;
use tinybmp::Bmp;

use embedded_graphics::{
    drawable::{Drawable, Pixel},
    geometry::Point,
    image::Image,
    pixelcolor::{BinaryColor, Rgb565, RgbColor},
    transform::Transform,
};

pub fn weather_icon(id: u32) -> Result<Bmp<'static>> {
    let r = match id {
        200 => icons::wi_day_thunderstorm_64x64(),
        201 => icons::wi_day_thunderstorm_64x64(),
        202 => icons::wi_day_thunderstorm_64x64(),
        210 => icons::wi_day_lightning_64x64(),
        211 => icons::wi_day_lightning_64x64(),
        212 => icons::wi_day_lightning_64x64(),
        221 => icons::wi_day_lightning_64x64(),
        230 => icons::wi_day_thunderstorm_64x64(),
        231 => icons::wi_day_thunderstorm_64x64(),
        232 => icons::wi_day_thunderstorm_64x64(),
        300 => icons::wi_day_sprinkle_64x64(),
        301 => icons::wi_day_sprinkle_64x64(),
        302 => icons::wi_day_rain_64x64(),
        310 => icons::wi_day_rain_64x64(),
        311 => icons::wi_day_rain_64x64(),
        312 => icons::wi_day_rain_64x64(),
        313 => icons::wi_day_rain_64x64(),
        314 => icons::wi_day_rain_64x64(),
        321 => icons::wi_day_sprinkle_64x64(),
        500 => icons::wi_day_sprinkle_64x64(),
        501 => icons::wi_day_rain_64x64(),
        502 => icons::wi_day_rain_64x64(),
        503 => icons::wi_day_rain_64x64(),
        504 => icons::wi_day_rain_64x64(),
        511 => icons::wi_day_rain_mix_64x64(),
        520 => icons::wi_day_showers_64x64(),
        521 => icons::wi_day_showers_64x64(),
        522 => icons::wi_day_showers_64x64(),
        531 => icons::wi_day_storm_showers_64x64(),
        600 => icons::wi_day_snow_64x64(),
        601 => icons::wi_day_sleet_64x64(),
        602 => icons::wi_day_snow_64x64(),
        611 => icons::wi_day_rain_mix_64x64(),
        612 => icons::wi_day_rain_mix_64x64(),
        615 => icons::wi_day_rain_mix_64x64(),
        616 => icons::wi_day_rain_mix_64x64(),
        620 => icons::wi_day_rain_mix_64x64(),
        621 => icons::wi_day_snow_64x64(),
        622 => icons::wi_day_snow_64x64(),
        701 => icons::wi_showers_64x64(),
        711 => icons::wi_smoke_64x64(),
        721 => icons::wi_day_haze_64x64(),
        731 => icons::wi_dust_64x64(),
        741 => icons::wi_fog_64x64(),
        761 => icons::wi_dust_64x64(),
        762 => icons::wi_dust_64x64(),
        781 => icons::wi_tornado_64x64(),
        800 => icons::wi_day_sunny_64x64(),
        801 => icons::wi_day_cloudy_gusts_64x64(),
        802 => icons::wi_day_cloudy_gusts_64x64(),
        803 => icons::wi_day_cloudy_gusts_64x64(),
        804 => icons::wi_day_cloudy_64x64(),
        900 => icons::wi_tornado_64x64(),
        901 => icons::wi_storm_showers_64x64(),
        902 => icons::wi_hurricane_64x64(),
        903 => icons::wi_snowflake_cold_64x64(),
        904 => icons::wi_hot_64x64(),
        905 => icons::wi_windy_64x64(),
        906 => icons::wi_hail_64x64(),
        957 => icons::wi_strong_wind_64x64(),
        _ => Err(()),
    };

    r.map_err(|_| anyhow!("Failed to read weather image id: {}", id))
}

fn cardinal_wind_direction(degree: f32) -> Result<&'static str> {
    match degree {
        x if x >= 0.0 && x < 11.25 || x >= 348.75 && x <= 360.0 => Ok("N"),
        x if x >= 11.25 && x < 33.75 => Ok("NNE"),
        x if x >= 33.75 && x < 56.25 => Ok("NE"),
        x if x >= 56.26 && x < 78.75 => Ok("ENE"),
        x if x >= 78.75 && x < 101.25 => Ok("E"),
        x if x >= 101.25 && x < 123.75 => Ok("ESE"),
        x if x >= 123.75 && x < 146.25 => Ok("SE"),
        x if x >= 146.25 && x < 168.75 => Ok("SSE"),
        x if x >= 168.75 && x < 191.25 => Ok("S"),
        x if x >= 191.25 && x < 213.75 => Ok("SSW"),
        x if x >= 213.75 && x < 236.25 => Ok("SW"),
        x if x >= 236.25 && x < 258.75 => Ok("WSW"),
        x if x >= 258.75 && x < 281.25 => Ok("W"),
        x if x >= 303.75 && x < 326.25 => Ok("NW"),
        x if x >= 326.25 && x < 348.75 => Ok("NW"),
        _ => Err(anyhow!("Invalid wind direction: {}", degree)),
    }
}

pub fn weather<T1: DrawTarget<BinaryColor>, T2: DrawTarget<BinaryColor>>(
    display: &mut T1,
    chromatic_display: &mut T2,
) -> Result<()> {
    debug!("Weather report current");
    let weather = openweather::get_current_weather(
        &WEATHER_LOCATION,
        &OPENWEATHER_API_KEY,
        &OPENWEATHER_SETTINGS,
    )?;

    info!(
        "In {}, {} it is {}°C",
        weather.name, weather.sys.country, weather.main.temp
    );
    draw_temp(display, chromatic_display, &weather)?;

    #[cfg(any(feature = "epd4in2", feature = "epd7in5bc"))]
    weather_forecast(display, chromatic_display, weather.main.temp)?;

    sunrise_and_sunset(
        display,
        weather.sys.sunrise as i64,
        weather.sys.sunset as i64,
    )?;

    Ok(())
}

fn sunrise_and_sunset<T: DrawTarget<BinaryColor>>(
    display: &mut T,
    sunrise: i64,
    sunset: i64,
) -> Result<()> {
    // Construct a datetime from epoch:
    let sunrise: DateTime<Local> = Utc.timestamp(sunrise as i64, 0).into();
    let sunset: DateTime<Local> = Utc.timestamp(sunset as i64, 0).into();
    // println!("{}", sunrise.to_rfc2822());
    // println!("{}", sunset.to_rfc2822());
    //assert_eq!(dt.to_rfc2822(), "Fri, 14 Jul 2017 02:40:00 +0000");

    draw_sunset(display, sunrise, sunset)?;

    Ok(())
}

#[cfg(feature = "epd4in2")]
fn draw_temp<T1: DrawTarget<BinaryColor>, T2: DrawTarget<BinaryColor>>(
    display: &mut T1,
    chromatic_display: &mut T2,
    temp: f32,
    weather_id: u32,
) -> Result<()> {
    let image: Bmp<'static> = weather::weather_icon(weather_id)?;
    Image::new(&image, Point::zero())
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
        (width() - 7 * 24, 100).into(),
    );

    Ok(())
}

#[cfg(feature = "epd2in9")]
fn draw_temp<T1: DrawTarget<BinaryColor>, T2: DrawTarget<BinaryColor>>(
    display: &mut T1,
    chromatic_display: &mut T2,
    temp: f32,
    weather_id: u32,
) -> Result<()> {
    text_24x32(
        display,
        &format!("{:5.1}°", temp),
        (width() - 6 * 24, height() - 32).into(),
    );

    Ok(())
}

#[cfg(feature = "epd7in5bc")]
fn draw_temp<T1: DrawTarget<BinaryColor>, T2: DrawTarget<BinaryColor>>(
    display: &mut T1,
    chromatic_display: &mut T2,
    weather_report: &WeatherReportCurrent,
) -> Result<()> {
    let weather = weather_report.weather.get(0).unwrap();

    let image: Bmp<'static> = weather_icon(weather.id)?;
    Image::new(&image, Point::zero())
        .translate((60, 50).into())
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
        .draw(chromatic_display)
        .map_err(|_| anyhow!("Failed to draw weather icon"))?;

    text_24x32(
        chromatic_display,
        &format!("{:5.1}°C", weather_report.main.temp),
        (34, 110).into(),
    );

    let wind_direction = weather_report
        .wind
        .deg
        .and_then(|x| cardinal_wind_direction(x).ok())
        .unwrap_or_default();

    text_12x16(
        display,
        &format!(
            "{}\n\n{:<8.2}km/h {}\n{:<8.2}%\n{:<8.2}hPa",
            weather.description,
            weather_report.wind.speed * 3.6,
            wind_direction,
            weather_report.main.humidity,
            weather_report.main.pressure,
        ),
        (190, 50).into(),
    );

    Ok(())
}

#[cfg(feature = "epd4in2")]
fn draw_sunset<T: DrawTarget<BinaryColor>>(
    display: &mut T,
    sunrise: DateTime<Local>,
    sunset: DateTime<Local>,
) -> Result<()> {
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

    Ok(())
}

#[cfg(feature = "epd2in9")]
fn draw_sunset<T: DrawTarget<BinaryColor>>(
    display: &mut T,
    sunrise: DateTime<Local>,
    sunset: DateTime<Local>,
) -> Result<()> {
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

    Ok(())
}

#[cfg(feature = "epd7in5bc")]
fn draw_sunset<T: DrawTarget<BinaryColor>>(
    display: &mut T,
    sunrise: DateTime<Local>,
    sunset: DateTime<Local>,
) -> Result<()> {
    text_12x16(
        display,
        &format!(
            "{:2}:{:02} | {:2}:{:02}",
            sunrise.hour(),
            sunrise.minute(),
            sunset.hour(),
            sunset.minute()
        ),
        (width() - 150i32, 0).into(),
    );

    Image::new(&icons::wi_sunrise_16x16().unwrap(), Point::zero())
        .translate((width() - 160i32, 0).into())
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
        .draw(display)
        .map_err(|_| anyhow!("Failed to draw sunrise icon"))?;

    Image::new(&icons::wi_sunset_16x16().unwrap(), Point::zero())
        .translate((width() - 20i32, 0).into())
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
        .draw(display)
        .map_err(|_| anyhow!("Failed to draw sunset icon"))?;

    Ok(())
}
