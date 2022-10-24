
use std::str::FromStr;
use std::time::Duration;
use hyper::body::{Bytes, HttpBody};
use hyper::http::Error;
use tokio::time::sleep;

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

            let now = std::time::SystemTime::now();
            match string.lines().nth_back(1) {
                Some(l) => println!("latest (now={:?}): {}", now, l),
                _ => {}
            }
            // match response.body(). {
            //     Some(Ok(bytes) ) => {
            //         let s = String::from_utf8(bytes.to_vec()).expect("from utf-8");
            //         println!("Data: {}", s);
            //     },
            //     Some(Err(_)) => {},
            //     None => (),
            // }

            sleep(Duration::from_secs(1)).await;

            // println!("http body: {}", body);
        }
    }

}