<!-- cargo-rdme start -->

Opentelemetry observable metric implementations based on std::sync::atomic
types.
Opentelemetry has a concept of "observable" metrics that are not reported
as they are updated, but rather, when an update happens, they are polled.
For ease-of-use in code, it is often desirable to have these metrics be
backed by [std::sync::atomic] types, so that they can be easily updated
throughout the code, and fetched whenever a metric reporting poll occurs.
This crate provides the [MeterExt] trait and associated types to make
it easy to use [std::sync::atomic] backed metrics with opentelemetry.

## Example

```rust
use influxive_otel_atomic_obs::MeterExt;

let (my_metric, _) = opentelemetry_api::global::meter("my_meter")
    .u64_observable_gauge_atomic("my_metric", 0)
    .init();

my_metric.set(66); // probably will not be reported
my_metric.set(99); // probably will not be reported
my_metric.set(42); // will be reported next time reporting runs
```

<!-- cargo-rdme end -->
