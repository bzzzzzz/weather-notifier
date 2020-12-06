mod config;
mod forecast_client;
mod measures;
mod notification;

use crate::config::FlyingSite;
use crate::forecast_client::{
    DayWeatherForecast, HourWeatherForecast, OpenWeatherMapClient, TimeOfDay,
};
use crate::measures::{Temperature, WindSpeed};
use crate::notification::TelegramClient;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use clap::{App, Arg};
use std::path::Path;

impl FlyingSite {
    fn is_flyable(&self, hour: &HourWeatherForecast) -> bool {
        !(hour.pop > 0.3
            || hour.time_of_day != TimeOfDay::DAY
            || self.min_flyable_wind_degree > hour.wind_deg
            || hour.wind_deg > self.max_flyable_wind_degree
            || self.min_flyable_wind > hour.wind_speed
            || hour.wind_speed > self.max_flyable_wind)
    }
}

#[derive(Debug)]
struct SiteFlyablePeriod {
    start: DateTime<FixedOffset>,
    duration_hours: i64,
    wind_min: WindSpeed,
    wind_max: WindSpeed,
    wind_degree_min: i16,
    wind_degree_max: i16,
    temp_min: Temperature,
    temp_max: Temperature,
}

impl SiteFlyablePeriod {
    fn from_hour(hour: &HourWeatherForecast) -> Self {
        Self {
            start: hour.time,
            duration_hours: 1,
            wind_min: hour.wind_speed,
            wind_max: hour.wind_speed,
            wind_degree_min: hour.wind_deg,
            wind_degree_max: hour.wind_deg,
            temp_min: hour.temperature,
            temp_max: hour.temperature,
        }
    }

    fn is_next_hour(&self, hour: &HourWeatherForecast) -> bool {
        self.start + Duration::hours(self.duration_hours) == hour.time
    }

    fn add_hour(&mut self, hour: &HourWeatherForecast) {
        self.duration_hours += 1;
        if self.wind_min > hour.wind_speed {
            self.wind_min = hour.wind_speed;
        }
        if self.wind_max < hour.wind_speed {
            self.wind_max = hour.wind_speed;
        }
        if self.wind_degree_min > hour.wind_deg {
            self.wind_degree_min = hour.wind_deg;
        }
        if self.wind_degree_max < hour.wind_deg {
            self.wind_degree_max = hour.wind_deg;
        }
        if self.temp_min > hour.temperature {
            self.temp_min = hour.temperature;
        }
        if self.temp_max < hour.temperature {
            self.temp_max = hour.temperature;
        }
    }
}

#[derive(Debug)]
struct SiteFlyAbilityReport {
    site: FlyingSite,
    periods: Vec<SiteFlyablePeriod>,
}

impl SiteFlyAbilityReport {
    fn as_string(&self) -> String {
        let mut repr = format!("{name} is flyable tomorrow:", name = self.site.name);
        for period in &self.periods {
            let period_descr = format!(
                "\n- Starting at {time} for {duration} hours. \
            Wind from {min_wind:.1} to {max_wind:.1} MPH. \
            Direction from {min_deg:.1} to {max_deg:.1} degrees. \
            Temperature from {min_t:.1}F to {max_t:.1}F",
                time = period.start.format("%H:%M"),
                duration = period.duration_hours,
                min_wind = period.wind_min.miles_per_hour(),
                max_wind = period.wind_max.miles_per_hour(),
                min_deg = period.wind_degree_min,
                max_deg = period.wind_degree_max,
                min_t = period.temp_min.fahrenheit(),
                max_t = period.temp_max.fahrenheit(),
            );
            repr.push_str(&period_descr[..]);
        }
        repr
    }
}

fn prepare_report_for_site(
    forecasts: Vec<DayWeatherForecast>,
    site: FlyingSite,
) -> Option<SiteFlyAbilityReport> {
    if forecasts.is_empty() {
        return None;
    }

    let tz = forecasts[0].date.timezone();
    let tomorrow = (Utc::now().with_timezone(&tz) + Duration::days(1)).date();
    let forecast = forecasts.into_iter().find(|f| f.date == tomorrow);
    forecast.as_ref()?;

    let forecast = forecast.unwrap();
    let mut flying_hours = vec![];
    for hour in forecast.hourly {
        if site.is_flyable(&hour) {
            flying_hours.push(hour);
        }
    }
    if flying_hours.is_empty() {
        return None;
    }
    let mut periods = vec![];
    let mut current_period = SiteFlyablePeriod::from_hour(&flying_hours[0]);
    for hour in flying_hours.iter().skip(1) {
        if current_period.is_next_hour(hour) {
            current_period.add_hour(hour);
        } else {
            periods.push(current_period);
            current_period = SiteFlyablePeriod::from_hour(hour);
        }
    }
    periods.push(current_period);
    Some(SiteFlyAbilityReport { site, periods })
}

async fn check_sites(
    client: &OpenWeatherMapClient,
    sites: Vec<FlyingSite>,
) -> Result<Vec<SiteFlyAbilityReport>, Box<dyn std::error::Error>> {
    let mut reports: Vec<SiteFlyAbilityReport> = vec![];
    for site in sites {
        let forecast = client.get_forecast(site.latitude, site.longitude).await?;
        let report = prepare_report_for_site(forecast, site);
        if let Some(sfar) = report {
            reports.push(sfar);
        }
    }
    Ok(reports)
}

async fn send_notifications(
    client: &TelegramClient,
    user_ids: Vec<String>,
    reports: Vec<SiteFlyAbilityReport>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut message = String::from("");
    for report in reports {
        message.push_str(&report.as_string()[..]);
    }
    for user_id in user_ids {
        client.notify(user_id, &message).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Weather Forecast Notifier Service")
        .about("Notifies subscribers about wind conditions on specific sites")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .required(true)
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();
    let config_path = matches.value_of("config").unwrap();

    let app_config = config::load_config(&Path::new(config_path));
    let client = forecast_client::OpenWeatherMapClient::new(
        app_config.weather_api_url,
        app_config.weather_api_token,
    );
    let sites = app_config.sites;
    let reports = check_sites(&client, sites).await?;
    if !reports.is_empty() {
        let telegram_client = TelegramClient::new(app_config.telegram.bot_token);
        send_notifications(&telegram_client, app_config.telegram.chat_ids, reports).await?;
    }

    Ok(())
}
