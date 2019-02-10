use rocket::{Request, Data, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::Outcome;
use rocket::request::{self, FromRequest};

use std::time::SystemTime;

use prometheus::{Counter, Histogram, BasicAuthentication};



lazy_static! {
    static ref PUSH_COUNTER: Counter = register_counter!(
        "endpoint_request_total",
        "Total number of completed requests."
    )
    .unwrap();
    static ref PUSH_REQ_HISTOGRAM: Histogram = register_histogram!(
        "endpoint_request_duration_milliseconds",
        "The request latencies in seconds."
    )
    .unwrap();
}

pub struct PrometheusLogger {
    pub address: String,
    pub metric_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Copy, Clone)]
struct TimerStart(Option<SystemTime>);

impl Fairing for PrometheusLogger {
    fn info(&self) -> Info {
        Info {
            name: "Request Timer",
            kind: Kind::Request | Kind::Response
        }
    }

    fn on_request(&self, request: &mut Request, _: &Data) {
        request.local_cache(|| TimerStart(Some(SystemTime::now())));
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        let start_time = request.local_cache(|| TimerStart(None));
        if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            let ms = duration.as_secs() * 1000 + duration.subsec_millis() as u64;

            PUSH_REQ_HISTOGRAM.observe(ms as f64);

            PUSH_COUNTER.inc();
            let metric_families = prometheus::gather();

            let method = request.method().as_str().to_owned();
            let status = response.status().to_string();

            if cfg!(feature = "test") {
                add_test_headers(response, ms, method.clone(), status.clone());
            }

            if cfg!(not(feature = "test")) {
                prometheus::push_metrics(
                    &self.metric_name,
                    labels! {
                        "method".to_owned() => method,
                        "status".to_owned() => status,
                    },
                    &self.address,
                    metric_families,
                    Some(BasicAuthentication{
                        username: self.username.to_owned(),
                        password: self.password.to_owned(),
                    }),
                )
                .unwrap();
            } 
        }
    }
}

/// Request guard used to retrieve the start time of a request.
#[derive(Copy, Clone)]
pub struct StartTime(pub SystemTime);

// Allows a route to access the time a request was initiated.
impl<'a, 'r> FromRequest<'a, 'r> for StartTime {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<StartTime, ()> {
        match *request.local_cache(|| TimerStart(None)) {
            TimerStart(Some(time)) => Outcome::Success(StartTime(time)),
            TimerStart(None) => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}

fn add_test_headers(response: &mut Response, duration: u64, method: String, status: String) {
    response.set_raw_header("X-Test-Prometheus-Logger-Duration", format!("{}", duration));
    response.set_raw_header("X-Test-Prometheus-Logger-Method", format!("{}", method));
    response.set_raw_header("X-Test-Prometheus-Logger-Status", format!("{}", status));
}
