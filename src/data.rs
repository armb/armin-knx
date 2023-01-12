use std::collections::HashMap;
use crate::data::Dimension::{Brightness, Power, Temperature};
use crate::data::Unit::Watts;

#[derive(Debug, Clone, Copy)]
pub enum Unit {
    One,
    Celsius,
    Lux,
    Watts,
}

#[derive(Debug, Clone, Copy)]
pub enum Dimension {
    None,
    Temperature,
    Brightness,
    Power,
}

#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    pub(crate) dimension: Dimension,
    pub(crate) unit: Unit,
    pub(crate) value: f32,
}

// impl Measurement {
//     pub fn empty() -> Measurement {
//         Measurement { dimension: Dimension::None, unit: Unit::One, value: 0f32 }
//     }
// }

#[derive(Debug)]
pub struct Data {
    pub till: Measurement,
    pub flur_brightness: Measurement,
    pub total_power: Measurement,
    pub measurements: HashMap<String, Measurement>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            till: Measurement { dimension: Temperature, unit: Unit::Celsius, value: 0f32},
            flur_brightness: Measurement { dimension: Brightness, unit: Unit::Lux, value: 0f32},
            total_power: Measurement { dimension: Power, unit: Watts, value: 0f32}
        }
    }
}
