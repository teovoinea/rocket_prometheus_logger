#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] 
extern crate rocket;

extern crate rocket_prometheus_logger;
use rocket_prometheus_logger::prometheus_fairing;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(prometheus_fairing::PrometheusLogger{
            address: String::from("http://127.0.0.1:9091/"),
            metric_name: String::from("endpoint_logging"),
            username: String::from("user"),
            password: String::from("pass"),
        })
        .mount("/", routes![hello])
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::local::Client;
    use rocket::http::Status;

    #[test]
    fn test_hello() {
        let client = Client::new(rocket()).unwrap();
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
        assert!(response.headers().contains("X-Test-Prometheus-Logger-Duration"));
        assert_eq!(response.headers().get_one("X-Test-Prometheus-Logger-Method"), Some("GET"));
        assert_eq!(response.headers().get_one("X-Test-Prometheus-Logger-Status"), Some("200 OK"));
    }
}