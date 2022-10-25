
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};
use chrono::format::strftime;
use hyper::body::{Bytes, HttpBody};
use hyper::http::Error;
use tokio::time::sleep;
use std::io::Write;

pub struct Youless {
}


impl Youless {
    pub fn create() -> Youless {
        Youless {
        }
    }

    pub async fn thread_function(&self) {
        // 1. GET
        let url = hyper::Uri::from_static("http://www.arbu.eu:81/youless/V?h=1");
        let mut file = std::fs::File::options()
            .append(true)
            .open("/home/armin/youless-log.txt")
            .expect("open");
        loop {
            let client = hyper::Client::new();

            let result = client.get(url.clone()).await;
            let page = match result {
                Ok(response) => {
                    let body = response.into_body().data().await;
                    body.expect("no data").expect("bytes").to_vec()
                },
                _ => vec![],
            };
            let string = String::from_utf8(page).expect("utf-8");
            // println!("string={}", string);

            let now : DateTime<Utc> = DateTime::from(std::time::SystemTime::now());
            match string.lines().nth_back(1) {
                Some(l) => {
                    let append = format!("{} {}\n", now.timestamp(), l);
                    file.write(append.as_bytes());

                    println!("{} {}", now.timestamp(), l);
                },
                _ => {}
            }

            sleep(Duration::from_secs(60)).await;
        }
    }

}