use crate::Measurement;

#[derive(Debug)]
pub struct Wetter {
    pub till: Measurement,
    pub flur_brightness: Measurement,
}

impl Wetter {
    pub fn new() -> Self {
        Self {
            till: Measurement::Undefined,
            flur_brightness: Measurement::Undefined,
        }
    }
}
