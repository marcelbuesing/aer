use crate::*;
use anyhow::{anyhow, Result};
use embedded_graphics::transform::Transform;
use embedded_graphics::{
    drawable::{Drawable, Pixel},
    geometry::Point,
    image::Image,
    pixelcolor::BinaryColor,
    pixelcolor::{Rgb565, RgbColor},
    DrawTarget,
};
use log::*;
use tinybmp::Bmp;

#[cfg(not(feature = "epd7in5bc"))]
fn tmp_graph_height() -> i32 {
    120
}

#[cfg(feature = "epd7in5bc")]
fn tmp_graph_height() -> i32 {
    160
}

fn scale(min: i32, max: i32) -> i32 {
    let tmp = tmp_graph_height() / ((max + 5) - (min - 5));
    debug!("Scale: {}", tmp);
    match tmp_graph_height() / ((max + 5) - (min - 5)) {
        x if x.le(&1) => 1,
        //x if x.ge(&5) => 5,
        x => x,
    }
}

#[cfg(not(feature = "epd7in5bc"))]
fn pos_x(day: usize, slot: usize) -> i32 {
    let mul = 10;
    (day * 8 + slot) as i32 * mul
}

#[cfg(feature = "epd7in5bc")]
fn pos_x(day: usize, slot: usize) -> i32 {
    let mul = 20;
    (day * 8 + slot) as i32 * mul
}

struct Range {
    //min: i32,
    //max: i32,
    scale: i32,
    offset: i32,
}

impl Range {
    fn new(min: i32, max: i32, basic_offset: Option<i32>) -> Self {
        let scale = scale(min, max);
        Self {
            // min,
            // max,
            scale,
            offset: basic_offset.unwrap_or_default()
                + if min.is_negative() {
                    (min - 5).abs() * scale
                } else {
                    0
                },
        }
    }
    fn pos_y(&self, temp: f32) -> i32 {
        //let scale = 4;
        //let dist_from_ground = tmp_graph_height() + 10;
        -(temp as i32 * self.scale) - self.offset //-dist_from_ground
    }
}

#[cfg(not(feature = "epd7in5bc"))]
pub fn weather_forecast<T1: DrawTarget<BinaryColor>, T2: DrawTarget<BinaryColor>>(
    display: &mut T1,
    chromatic_display: &mut T2,
    current_temp: f32,
) {
    let forecast = match openweather::get_5_day_forecast(
        &WEATHER_LOCATION,
        &OPENWEATHER_API_KEY,
        &OPENWEATHER_SETTINGS,
    ) {
        Ok(forecast) => forecast,
        Err(e) => {
            error("Getting 5 Day Forecast", e);
            return;
        }
    };

    let mut abs_min = current_temp;
    let mut abs_max: f32 = current_temp;
    let mut temps: Vec<f32> = Vec::new();

    for (day, day_list) in forecast.list.chunks(8).take(4).enumerate() {
        let mut min = std::f32::MAX;
        let mut max: f32 = std::f32::MIN;

        for h3_slot in day_list.iter() {
            //let tmp = day.main.temp;
            min = min.min(h3_slot.main.temp_min);
            max = max.max(h3_slot.main.temp_max);
            debug!(
                "Day {}: Norm: {} | Min: {} | Max: {}",
                day + 1,
                h3_slot.main.temp,
                h3_slot.main.temp_min,
                h3_slot.main.temp_max
            );
            temps.push(h3_slot.main.temp);
        }
        debug!("Day {}: Min: {} | Max: {}", day + 1, min, max);
        // TODO cleanup
        let image: &Bmp<'static> = weather::WEATHER_ICONS
            .get(&day_list.get(0).unwrap().weather.get(0).unwrap().id)
            .unwrap();
        Image::new(image, Point::zero())
            .translate(Point::new(pos_x(day, 4) - 30, height() - 30))
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

        text_6x8(
            display,
            &format!("{:6.2}°C\n{:6.2}°C", min, max),
            (pos_x(day, 4), height() - 20).into(),
        );

        abs_min = abs_min.min(min);
        abs_max = abs_max.max(max);
    }
    let abs_min: i32 = abs_min as i32;
    let abs_max: i32 = abs_max as i32;
    let basic_x_offset = 35;
    let basic_y_offset = 25;
    let r = Range::new(abs_min, abs_max, Some(basic_y_offset));

    let _ = rectangle(
        Point::new(0, height() - tmp_graph_height() - basic_y_offset),
        Point::new(width(), height() - basic_y_offset),
    )
    .draw(display);

    let mut prev_temp = current_temp;
    for (counter, temp) in temps.iter().enumerate() {
        let _ = line(
            (pos_x(0, counter), r.pos_y(prev_temp)).into(),
            (pos_x(0, counter + 1), r.pos_y(*temp)).into(),
        )
        .translate((basic_x_offset, height()).into())
        .draw(display);
        prev_temp = *temp;
    }

    for temp in (-30..=50)
        .step_by(10)
        .filter(|x| (*x).ge(&(abs_min - 5)) && (*x).le(&(abs_max + 5)))
    {
        text_6x8(
            display,
            &format!("{:3.2}°C", temp),
            (0, height() + r.pos_y(temp as f32)).into(),
        );
        let _ = line(
            (pos_x(0, 0), r.pos_y(temp as f32)).into(),
            (pos_x(3, 8), r.pos_y(temp as f32)).into(),
        )
        .translate(Point::new(basic_x_offset, height()))
        .draw(display);
    }
}

#[cfg(feature = "epd7in5bc")]
pub fn weather_forecast<T1: DrawTarget<BinaryColor>, T2: DrawTarget<BinaryColor>>(
    display: &mut T1,
    chromatic_display: &mut T2,
    current_temp: f32,
) -> Result<()> {
    let forecast = openweather::get_5_day_forecast(
        &WEATHER_LOCATION,
        &OPENWEATHER_API_KEY,
        &OPENWEATHER_SETTINGS,
    )?;

    let mut abs_min = current_temp;
    let mut abs_max: f32 = current_temp;
    let mut temps: Vec<f32> = Vec::new();

    for (day, day_list) in forecast.list.chunks(8).take(4).enumerate() {
        let mut min = std::f32::MAX;
        let mut max: f32 = std::f32::MIN;

        for h3_slot in day_list.iter() {
            //let tmp = day.main.temp;
            min = min.min(h3_slot.main.temp_min);
            max = max.max(h3_slot.main.temp_max);
            debug!(
                "Day {}: Norm: {} | Min: {} | Max: {}",
                day + 1,
                h3_slot.main.temp,
                h3_slot.main.temp_min,
                h3_slot.main.temp_max
            );
            temps.push(h3_slot.main.temp);
        }
        debug!("Day {}: Min: {} | Max: {}", day + 1, min, max);
        // TODO cleanup
        let weather_id = day_list.get(0).unwrap().weather.get(0).unwrap().id;
        let image: Bmp<'static> = weather::weather_icon(weather_id)?;
        Image::new(&image, Point::zero())
            .translate(Point::new(pos_x(day, 4) - 50, height() - 60))
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
            .map_err(|_| anyhow!("Failed to draw weather icon"))?;

        let dt = day_list.get(0).unwrap().dt;
        let dt = Utc.timestamp(dt as i64, 0);

        text_8x16(
            chromatic_display,
            &format!("{}", dt.weekday()),
            (pos_x(day, 4), height() - 70).into(),
        );

        text_6x8(
            display,
            &format!("{:6.2}°C\n\n{:6.2}°C", min, max),
            (pos_x(day, 4) + 15, height() - 40).into(),
        );

        abs_min = abs_min.min(min);
        abs_max = abs_max.max(max);
    }
    let abs_min: i32 = abs_min as i32;
    let abs_max: i32 = abs_max as i32;
    let basic_x_offset = 35;
    let basic_y_offset = 80;
    let r = Range::new(abs_min, abs_max, Some(basic_y_offset));

    let _ = rectangle(
        Point::new(0, height() - tmp_graph_height() - basic_y_offset),
        Point::new(width(), height() - basic_y_offset),
    )
    .draw(display);

    let mut prev_temp = current_temp;
    for (counter, temp) in temps.iter().enumerate() {
        let _ = line(
            (pos_x(0, counter), r.pos_y(prev_temp)).into(),
            (pos_x(0, counter + 1), r.pos_y(*temp)).into(),
        )
        .translate((basic_x_offset, height()).into())
        .draw(chromatic_display);
        prev_temp = *temp;
    }

    for temp in (-20..=40)
        .step_by(10)
        .filter(|x| (*x).ge(&(abs_min - 5)) && (*x).le(&(abs_max + 5)))
    {
        text_6x8(
            display,
            &format!("{:3.2}°C", temp),
            (0, height() + r.pos_y(temp as f32)).into(),
        );
        let _ = line(
            (pos_x(0, 0), r.pos_y(temp as f32)).into(),
            (pos_x(3, 8), r.pos_y(temp as f32)).into(),
        )
        .translate(Point::new(basic_x_offset, height()))
        .draw(display);
    }

    Ok(())
}
