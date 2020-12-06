use crate::measures::WindSpeed;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct FlyingSite {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub min_flyable_wind: WindSpeed,
    pub max_flyable_wind: WindSpeed,
    pub min_flyable_wind_degree: i16,
    pub max_flyable_wind_degree: i16,
}

#[derive(Deserialize, Debug)]
pub struct Telegram {
    pub bot_token: String,
    pub chat_ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationConfig {
    pub weather_api_url: String,
    pub weather_api_token: String,
    pub telegram: Telegram,
    pub sites: Vec<FlyingSite>,
}

pub fn load_config(config_path: &Path) -> ApplicationConfig {
    let mut settings = config::Config::default();
    settings.merge(config::File::from(config_path)).unwrap();

    settings.try_into::<ApplicationConfig>().unwrap()
}
