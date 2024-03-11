use std::collections::{HashMap, LinkedList};
use std::time::SystemTime;
use crate::config::{Sensor, Switch};

use String as SensorId;

#[derive(Debug, Clone, Copy)]
pub enum Unit {
    One,
    Celsius,
    Lux,
    _Watts,
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
    #[allow(unused)]
    pub(crate) dimension: Dimension,
    #[allow(unused)]
    pub(crate) unit: Unit,
    pub(crate) value: Option<f32>,
}

#[derive()]
pub struct Data {
    // sensor-id
    pub measurements: HashMap<SensorId, Measurement>,
    pub log: HashMap<SensorId, LinkedList<Measurement> >,
    pub sqlite: sqlite::Connection
}


impl Data {
    pub fn new() -> Self {
        // let o = sqlite::open(":memory:").unwrap();
        let o = sqlite::open("a.sqlite").unwrap();
        let query_create = "CREATE TABLE data (time INTEGER, name TEXT, value INTEGER);";
        match o.execute(query_create) {
            Ok(..) => eprintln!("created new table"),
            Err(..) => eprintln!("could not create new table")
        }
        Self {
            measurements: HashMap::new(),
            log: HashMap::new(),
            sqlite: o
        }
    }
    pub fn insert(&mut self, id: &SensorId, value: f32) {
        let time = chrono::Local::now().timestamp();
        self.sqlite.execute(
            format!("INSERT INTO data VALUES ({time}, '{id}', {value});")
        ).expect("sqlite insert");
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
    pub fn add_switch(&mut self, id: &SensorId, _switch: &Switch) -> Result<(), String> {
        if self.measurements.contains_key(id) {
            Err("measurement entry for sensor already created".into())
        } else {
            let initial = Measurement { timestamp: SystemTime::now(), unit: Unit::One, dimension: Dimension::OnOff, value: None};
            self.measurements.insert(id.to_string(), initial);
            Ok(())
        }
    }
}
