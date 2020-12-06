use crate::measures::{Temperature, WindSpeed};
use chrono::{Date, DateTime, Duration, FixedOffset, TimeZone};
use reqwest::{Client, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct WeatherEvent {
    id: u16,
    main: String,
    description: String,
}

#[derive(Deserialize, Debug)]
pub struct HourlyWeather {
    dt: i64,
    temp: f32,
    feels_like: f32,
    wind_speed: f32,
    wind_deg: i16,
    clouds: i16,
    pop: f32,
    weather: Vec<WeatherEvent>,
}

#[derive(Deserialize, Debug)]
pub struct DailyWeather {
    dt: i64,
    sunrise: i64,
    sunset: i64,
}

#[derive(Deserialize, Debug)]
pub struct WeatherForecast {
    lat: f32,
    lon: f32,
    timezone: String,
    timezone_offset: i32,
    daily: Vec<DailyWeather>,
    hourly: Vec<HourlyWeather>,
}

#[derive(Debug, PartialEq)]
pub enum TimeOfDay {
    NIGHT,
    TWILIGHT,
    DAY,
}

#[derive(Debug)]
pub struct HourWeatherForecast {
    pub time: DateTime<FixedOffset>,
    pub time_of_day: TimeOfDay,
    pub temperature: Temperature,
    pub feels_like: Temperature,
    pub wind_speed: WindSpeed,
    pub wind_deg: i16,
    pub pop: f32,
}

#[derive(Debug)]
pub struct DayWeatherForecast {
    pub date: Date<FixedOffset>,
    pub sunrise: DateTime<FixedOffset>,
    pub sunset: DateTime<FixedOffset>,
    pub hourly: Vec<HourWeatherForecast>,
}

pub struct OpenWeatherMapClient {
    url: String,
    app_id: String,
}

impl OpenWeatherMapClient {
    pub fn new(url: String, app_id: String) -> Self {
        OpenWeatherMapClient { url, app_id }
    }

    pub async fn get_forecast(&self, lat: f64, lon: f64) -> Result<Vec<DayWeatherForecast>> {
        let client = Client::new();
        let raw_forecast = client
            .get(&self.url)
            .query(&[
                ("lat", &lat.to_string()[..]),
                ("lon", &lon.to_string()[..]),
                ("appid", &self.app_id[..]),
                ("exclude", "current,minutely,alerts&units=metric"),
                ("units", "metric"),
            ])
            .send()
            .await?
            .json::<WeatherForecast>()
            .await?;
        let tz_offset = FixedOffset::east(raw_forecast.timezone_offset);
        let mut date_to_forecast: HashMap<Date<FixedOffset>, DayWeatherForecast> = HashMap::new();
        for day_forecast in raw_forecast.daily.iter() {
            let date = tz_offset.timestamp(day_forecast.dt, 0).date();
            let sunrise = tz_offset.timestamp(day_forecast.sunrise, 0);
            let sunset = tz_offset.timestamp(day_forecast.sunset, 0);
            date_to_forecast.insert(
                date,
                DayWeatherForecast {
                    date,
                    sunrise,
                    sunset,
                    hourly: vec![],
                },
            );
        }
        for hour_forecast in raw_forecast.hourly.iter() {
            let date_time = tz_offset.timestamp(hour_forecast.dt, 0);
            let day_forecast = date_to_forecast.get_mut(&date_time.date()).unwrap();
            let time_of_day = get_time_of_day(date_time, day_forecast.sunrise, day_forecast.sunset);

            let forecast = HourWeatherForecast {
                time: date_time,
                time_of_day,
                temperature: Temperature::C(hour_forecast.temp),
                feels_like: Temperature::C(hour_forecast.feels_like),
                wind_speed: WindSpeed::MPS(hour_forecast.wind_speed),
                wind_deg: hour_forecast.wind_deg,
                pop: hour_forecast.pop,
            };
            day_forecast.hourly.push(forecast);
        }
        let mut day_forecasts: Vec<DayWeatherForecast> = date_to_forecast
            .into_iter()
            .map(|x| x.1)
            .filter(|x| !x.hourly.is_empty())
            .collect();
        day_forecasts.sort_by_key(|k| k.date);
        Ok(day_forecasts)
    }
}

fn get_time_of_day(
    date_time: DateTime<FixedOffset>,
    sunrise: DateTime<FixedOffset>,
    sunset: DateTime<FixedOffset>,
) -> TimeOfDay {
    let end_hour = date_time + Duration::hours(1);

    if sunset.ge(&end_hour) && sunrise.lt(&date_time) {
        TimeOfDay::DAY
    } else if sunrise.ge(&end_hour) || sunset.lt(&date_time) {
        TimeOfDay::NIGHT
    } else if sunset < end_hour {
        if sunset < date_time + Duration::minutes(14) {
            TimeOfDay::NIGHT
        } else if sunset < date_time + Duration::minutes(45) {
            TimeOfDay::TWILIGHT
        } else {
            TimeOfDay::DAY
        }
    } else if sunrise < date_time + Duration::minutes(14) {
        TimeOfDay::DAY
    } else if sunset < date_time + Duration::minutes(45) {
        TimeOfDay::TWILIGHT
    } else {
        TimeOfDay::NIGHT
    }
}
