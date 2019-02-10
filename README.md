# rocket_prometheus_logger

## Usage

First, import `rocket_prometheus_logger`:

```rust
extern crate rocket_prometheus_logger;
use rocket_prometheus_logger::prometheus_fairing;
```

Then, attach to your rocket!

```rust
rocket::ignite()
    .attach(prometheus_fairing::PrometheusLogger{
        address: String::from("http://127.0.0.1:9091/"),
        metric_name: String::from("endpoint_logging"),
        username: String::from("user"),
        password: String::from("pass"),
    })
```

## Testing

```cargo test --features "test"```

## 0.2.0 Goals

* Pull based logging
* Configurable metrics
