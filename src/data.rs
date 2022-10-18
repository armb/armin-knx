use std::collections::HashMap;
use crate::data::Dimension::{Brightness, Temperature};

#[derive(Debug, Clone, Copy)]
pub enum Unit {
    One,
    Celsius,
    Lux,
}

#[derive(Debug, Clone, Copy)]
pub enum Dimension {
    None,
    Temperature,
    Brightness
}

#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    dimension: Dimension,
    unit: Unit,
    value: f32,
}

#[derive(Debug)]
pub struct Data {
    pub till: Measurement,
    pub flur_brightness: Measurement,
    pub measurements: HashMap<String, Measurement>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            till: Measurement { dimension: Temperature, unit: Unit::One, value: 0.2f32},
            flur_brightness: Measurement { dimension: Brightness, unit: Unit::Lux, value: 0.2f32},
        }
    }
}
