use std::collections::HashMap;
use crate::Received;


#[derive(Debug, Copy, Clone)]
pub enum Measurement {
    _Error,
    Undefined,
    Temperature(Received, f32), // Deg Celsiuspub
    Brightness(Received, f32), // Lux
}

#[derive(Debug)]
pub struct Wetter {
    pub till: Measurement,
    pub flur_brightness: Measurement,
    pub measurements: HashMap<String, Measurement>,
}

impl Wetter {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            till: Measurement::Undefined,
            flur_brightness: Measurement::Undefined,
        }
    }
}
