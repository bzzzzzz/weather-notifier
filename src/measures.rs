use serde::Deserialize;
use std::cmp::Ordering;

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(tag = "type", content = "value")]
pub enum Temperature {
    C(f32),
    F(f32),
}

impl Temperature {
    pub fn celsius(&self) -> f32 {
        match *self {
            Temperature::C(degrees) => degrees,
            Temperature::F(degrees) => (degrees * 1.8) + 32.0,
        }
    }

    pub fn fahrenheit(&self) -> f32 {
        match *self {
            Temperature::C(degrees) => (degrees - 32.0) / 1.8,
            Temperature::F(degrees) => degrees,
        }
    }
}

impl PartialEq for Temperature {
    fn eq(&self, other: &Self) -> bool {
        (self.celsius() * 1000.0).round() == (other.celsius() * 1000.0).round()
    }
}

impl PartialOrd for Temperature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            Some(Ordering::Equal)
        } else if self.celsius() > other.celsius() {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(tag = "type", content = "value")]
pub enum WindSpeed {
    MPH(f32),
    KMPH(f32),
    MPS(f32),
}

const MPS_TO_KMPH: f32 = 3.6;
const MPS_TO_MPH: f32 = 2.236936;
const MPH_TO_KMPH: f32 = 1.609344;

impl WindSpeed {
    pub fn meters_per_second(&self) -> f32 {
        match *self {
            WindSpeed::MPH(mph) => mph / MPS_TO_MPH,
            WindSpeed::KMPH(kmph) => kmph / MPS_TO_KMPH,
            WindSpeed::MPS(mps) => mps,
        }
    }

    pub fn miles_per_hour(&self) -> f32 {
        match *self {
            WindSpeed::MPH(mph) => mph,
            WindSpeed::KMPH(kmph) => kmph / MPH_TO_KMPH,
            WindSpeed::MPS(mps) => mps * MPS_TO_MPH,
        }
    }

    pub fn kilometers_per_second(&self) -> f32 {
        match *self {
            WindSpeed::MPH(mph) => mph * MPH_TO_KMPH,
            WindSpeed::KMPH(kmph) => kmph,
            WindSpeed::MPS(mps) => mps * MPS_TO_KMPH,
        }
    }
}

impl PartialEq for WindSpeed {
    fn eq(&self, other: &Self) -> bool {
        (self.meters_per_second() * 1000.0).round() == (other.meters_per_second() * 1000.0).round()
    }
}

impl PartialOrd for WindSpeed {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            Some(Ordering::Equal)
        } else if self.meters_per_second() > other.meters_per_second() {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}
