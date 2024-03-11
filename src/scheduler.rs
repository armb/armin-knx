use std::error::Error;
use std::fs::File;
use std::time::Duration;
use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};

pub struct Scheduler {
    waiting_events: Vec<Entry>
}

#[derive(Debug)]
struct Entry {
    time: NaiveTime,
    #[allow(unused)]
    actor: String,
    #[allow(unused)]
    command: String,
    timebase: Timebase
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Timebase {
    Local,
    Sunrise,
    Sunset
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScheduleFileEvent {
    time: Option<String>,
    actor: String,
    command: String,
    timebase: String
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

        // fill waiting_events from json file
        for e in json {

            let actor = e.actor;
            let command = e.command;
            let timebase = match e.timebase.as_str() {
                "local" => Timebase::Local,
                "sunset" => Timebase::Sunset,
                "sunrise" => Timebase::Sunrise,
                t => panic!("invalid timebase '{}'", t)
            };
            let parsed_time = if let Some(date) = e.time {
                NaiveTime::parse_from_str(&date, "%H:%M:%S").expect("parse time")
            } else {
                NaiveTime::default()
            };
            let time = match timebase {
                Timebase::Local => parsed_time,
                Timebase::Sunrise => {
                    let day = sunrise::SolarDay::new(
                        52.035806, 10.307611,
                    2024, 3, 8);

                    parsed_time },
                Timebase::Sunset => parsed_time
            };
            waiting_events.push(Entry { time, actor, command, timebase });

        }

        eprintln!("waiting_events: {waiting_events:?}");
        eprintln!("----------------------------------");
        Ok( Scheduler{ waiting_events } )
    }

    pub async fn thread_function(&mut self) -> Result<(), String> {
        eprintln!("----- scheduler thread_function BEGIN");
        while let Ok(ok) = self.handle_next().await {
            eprintln!("----- scheduler thread_function: ok={ok:?}");
        }
        eprintln!("----- scheduler thread_function END");
        Ok( () )
    }

    async fn handle_next(&mut self) -> Result<(), ()> {
        let n = self.find_next();
        eprintln!("next is: {n:?}");
        match n {
            Some(index) => {
                let event = self.waiting_events.remove(index);
                let now = Local::now().time();
                let diff = event.time - now;
                let seconds = diff.num_seconds();
                if seconds > 0 {
                    eprintln!("--- need to sleep {seconds} seconds.");
                    tokio::time::sleep(tokio::time::Duration::from_secs(seconds as u64)).await;
                }
                eprintln!("--- execute: actor={}, command={}", event.actor, event.command);
                eprintln!("--- removed index {index}");
                Ok( () )
            },
            None => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Err( () )
            }
        }
    }

    fn find_next(&self) -> Option<usize> {
        let mut result: Option<(usize,&Entry)> = None;
        let now = chrono::Local::now().naive_local().time();
        for i in 0..self.waiting_events.len() {
            let a = self.waiting_events.get(i).unwrap();
            // not in future
            if a.time < now {
                continue;
            }
            // skip later events
            if result.is_some() && a.time > result.unwrap().1.time {
                continue;
            }
            result = Some( (i,&a) );
            eprintln!("-- a: {a:?}");
        }
        match result { Some((i,_)) => Some(i), None => None }
    }
}