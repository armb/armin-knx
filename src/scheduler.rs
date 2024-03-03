use std::error::Error;
use std::fs::File;
use std::time::Duration;
use chrono::{DateTime, Local, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use crate::knx::Command;

pub struct Scheduler {
    waiting_events: Vec<Entry>
}

#[derive(Debug)]
struct Entry {
    time: NaiveTime,
    actor: String,
    command: Command
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScheduleFileEvent {
    time: Option<String>,
    actor: String,
    command: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScheduleFile {
    events: Vec<ScheduleFileEvent>,
}

impl Scheduler {
    pub fn new(config_file: &str) -> Result<Scheduler, Box<dyn Error>> {
        let file = File::open(config_file).unwrap();
        let json: Vec<ScheduleFileEvent> = serde_json::from_reader(file)?;

        let mut waiting_events: Vec<Entry> = vec![];

        eprintln!("----------------------------------");

        for e in json {
            let command = Command::from_str(&e.command)?;
            if let Some(date) = e.time {
                let time = NaiveTime::parse_from_str(date.as_str(), "%H:%M:%s").expect("parse time");
                waiting_events.push(Entry { time, actor: "".to_string(), command });
            }
        }

        eprintln!("waiting_events: {waiting_events:?}");
        eprintln!("----------------------------------");
        Ok( Scheduler{ waiting_events } )
    }

    pub async fn thread_function(&self) -> Result<(), String> {
        eprintln!("----- scheduler thread_function BEGIN");
        while let Some(n) = self.find_next() {
            eprintln!("next is: {n:?}");
            tokio::time::delay_for(Duration::from_secs(1)).await;
            // eprintln!("-----");
        }
        eprintln!("----- scheduler thread_function END");
        Ok( () )
    }
    fn find_next(&self) -> Option<&Entry> {
        let mut result: Option<&Entry> = None;
        let now = chrono::Local::now().naive_local().time();
        for a in &self.waiting_events {
            // not in future
            if a.time < now {
                continue;
            }
            // skip later events
            if result.is_some() && a.time > result.unwrap().time {
                continue;
            }
            result = Some(a);
            eprintln!("-- a: {a:?}");
        }
        result
    }
}