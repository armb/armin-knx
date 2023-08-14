use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::time::SystemTime;
use crate::config::Sensor;
use crate::data::Dimension::{Brightness, Temperature};
use crate::data::Unit::Watts;


use String as SensorId;

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
    Percent,
    OnOff,
}

#[derive(Debug, Copy, Clone)]
pub struct Measurement {
    pub(crate) timestamp:SystemTime,
    pub(crate) dimension: Dimension,
    pub(crate) unit: Unit,
    pub(crate) value: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct Data {
    // sensor-id
    pub measurements: HashMap<SensorId, Measurement>,
    pub log: HashMap<SensorId, LinkedList<Measurement> >,
}

impl Data {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            log: HashMap::new(),
        }
    }
    pub fn get_mut(&mut self, id: &SensorId) -> Option<&mut Measurement> {
        self.measurements.get_mut(id)
    }
    pub fn add_sensor(&mut self, id: &SensorId, sensor: &Sensor) -> Result<(), String> {
        if self.measurements.contains_key(id) {
            Err("measurement entry for sensor already created".into())
        } else {
            let initial = Measurement { timestamp: SystemTime::now(), unit: Unit::One, dimension: sensor.get_dimension(), value: None};
            self.measurements.insert(id.to_string(), initial);
            Ok(())
        }
    }
}
