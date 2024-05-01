use std::error::Error;
use std::fs::File;
use std::ops::Mul;
use std::sync::{Arc};
use std::time::{Duration, SystemTime};
use chrono::{Datelike, DateTime, Local, NaiveTime, TimeDelta, Timelike};
use serde::{Deserialize, Serialize};
use sunrise::{DawnType, SolarEvent};
use crate::config::Config;
use crate::knx::{Command, KnxSocket};

pub struct Scheduler {
    waiting_events: Vec<Entry>,
    config: Arc<Config>,
    knx: crate::knx::KnxSocket
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
    Sunset,
    Dusk, // Abenddämmerung
    Dawn // Morgenddämmerung
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScheduleFileEvent {
    disabled: Option<bool>,
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
    pub fn new(config: Arc<Config>, knx: KnxSocket) -> Result<Scheduler, Box<dyn Error>> {

        let waiting_events = vec![];

        Ok( Scheduler{ waiting_events, knx, config } )
    }

    fn read_file(path: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
        let file = File::open(path).unwrap();
        let json: Vec<ScheduleFileEvent> = serde_json::from_reader(file)?;

        let mut waiting_events: Vec<Entry> = vec![];

        eprintln!("----------------------------------");

        // fill waiting_events from json file
        for e in json {
            if e.disabled == Some(true) { continue; }
            let actor = e.actor;
            let command = e.command;
            let timebase = match e.timebase.as_str() {
                "local" => Timebase::Local,
                "sunset" => Timebase::Sunset,
                "sunrise" => Timebase::Sunrise,
                "dusk" => Timebase::Dusk,
                "dawn" => Timebase::Dawn,
                t => panic!("invalid timebase '{}'", t)
            };
            let parsed_time = if let Some(mut date) = e.time {
                // todo: negative values like -00:01:00
                println!("-- {date}");
                let mut is_negative = false;
                while date.starts_with('-') {
                    date.remove(0);
                    is_negative = !is_negative;
                }
                println!("++ {date}");
                let time =
                    NaiveTime::parse_from_str(&date, "%T")
                        .expect(format!("parse time ('{}')", date).as_str())
                        .num_seconds_from_midnight();
                let time = TimeDelta::new(time as i64, 0).expect("time string");
                if is_negative { time.mul(-1) } else { time }
            } else {
                TimeDelta::default()
            };
            let time = match timebase {
                Timebase::Local => (DateTime::<chrono::Local>::default() + parsed_time).time(),
                Timebase::Sunrise|Timebase::Sunset|Timebase::Dusk|Timebase::Dawn => {
                    let now = DateTime::<chrono::Local>::from(SystemTime::now());
                    let today = now.date_naive();
                    let day = sunrise::SolarDay::new(
                        52.035806, 10.307611,
                        today.year(), today.month(), today.day());
                    // day.with_altitude(140f64);
                    let seconds = day.event_time(match timebase {
                        Timebase::Sunset => SolarEvent::Sunset,
                        Timebase::Sunrise => SolarEvent::Sunrise,
                        Timebase::Dusk => SolarEvent::Dusk(DawnType::Civil),
                        Timebase::Dawn => SolarEvent::Dawn(DawnType::Civil),
                        _ => panic!()
                    });
                    // timezone offset in seconds
                    let tz_offset = Local::now().offset().local_minus_utc() as i64;
                    eprintln!("tz_offset: {tz_offset}");

                    let d = DateTime::from_timestamp(
                        seconds + tz_offset, 0).unwrap()
                        + parsed_time;
                    eprintln!("date --> {d:?} ({})", d.timestamp());
                    d.time()
                },
            };
            waiting_events.push(Entry { time, actor, command, timebase });
        }

        eprintln!("waiting_events: {waiting_events:?}");
        eprintln!("----------------------------------");

        Ok( waiting_events )
    }

    pub async fn thread_function(&mut self) -> Result<(), String> {
        eprintln!("----- scheduler thread_function BEGIN");
        loop {
            self.waiting_events = Scheduler::read_file(self.config.schedule_file.as_str()).expect("scheduler file");
            while let Some(n) = self.find_next() {
//            eprintln!("next is: {n:?}");
                self.handle_next(n).await.expect("handle_next() await");
                // eprintln!("-----");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn handle_next(&mut self, index: usize) -> Result<(), ()> {
        let event = self.waiting_events.remove(index);
        let now = Local::now().time();
        let diff = event.time - now;
        let seconds = diff.num_seconds();
        if seconds > 0 {
            eprintln!("--- need to sleep {seconds} seconds.");
            tokio::time::sleep(tokio::time::Duration::from_secs(seconds as u64)).await;
        }
        eprintln!("--- execute: actor={}, command={}", event.actor, event.command);
        let mut addr = None;
        if let Some(actor) = self.config.actors.get(&event.actor) {
            // check if known command for Actor
            if actor.commands.contains(&event.command) {
                addr = Some(actor.eibaddr.clone());
            } else if let  Some(switch) = self.config.switches.get(&event.actor) {
                if switch.commands.contains(&event.command) {
                    // check if known command for switch
                    addr = Some(switch.eibaddr_command.clone());
                }
            }
            let command = Command::from_str(&event.command).expect("command");
            self.knx.send(&addr.unwrap(), &command).expect("knx send");
        }
        eprintln!("--- removed index {index}");
        Ok( () )
    }

    fn find_next(&self) -> Option<usize> {
        let mut result: Option<(usize,&Entry)> = None;
        let now = chrono::Local::now().naive_local().time();
        for i in 0..self.waiting_events.len() {
            let a = self.waiting_events.get(i).unwrap();
            // not in future
            if a.time < now - Duration::from_secs(10) {
                continue;
            }
            // skip later events
            if result.is_some() && a.time > result.unwrap().1.time {
                continue;
            }
            result = Some((i ,a));
 //           eprintln!("-- a: {a:?}");
        }
        match result { Some((i,_)) => Some(i), None => None }
    }
}