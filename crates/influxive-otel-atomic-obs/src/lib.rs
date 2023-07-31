#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! Opentelemetry observable metric implementations based on std::sync::atomic
//! types.
//! Opentelemetry has a concept of "observable" metrics that are not reported
//! as they are updated, but rather, when an update happens, they are polled.
//! For ease-of-use in code, it is often desirable to have these metrics be
//! backed by [std::sync::atomic] types, so that they can be easily updated
//! throughout the code, and fetched whenever a metric reporting poll occurs.
//! This crate provides the [MeterExt] trait and associated types to make
//! it easy to use [std::sync::atomic] backed metrics with opentelemetry.
//!
//! ## Example
//!
//! ```
//! use influxive_otel_atomic_obs::MeterExt;
//!
//! let (my_metric, _) = opentelemetry_api::global::meter("my_meter")
//!     .u64_observable_gauge_atomic("my_metric", 0)
//!     .init();
//!
//! my_metric.set(66); // probably will not be reported
//! my_metric.set(99); // probably will not be reported
//! my_metric.set(42); // will be reported next time reporting runs
//! ```

use opentelemetry_api::metrics::Result;
use std::borrow::Cow;
use std::sync::atomic::*;
use std::sync::Arc;

#[inline(always)]
fn f64_to_u64(v: f64) -> u64 {
    u64::from_le_bytes(v.to_le_bytes())
}

#[inline(always)]
fn u64_to_f64(v: u64) -> f64 {
    f64::from_le_bytes(v.to_le_bytes())
}

/// Metric builder.
pub struct AtomicObservableInstrumentBuilder<'a, C, I, M>(
    C,
    opentelemetry_api::metrics::AsyncInstrumentBuilder<'a, I, M>,
)
where
    I: opentelemetry_api::metrics::AsyncInstrument<M>;

impl<'a, C, I, M> AtomicObservableInstrumentBuilder<'a, C, I, M>
where
    I: TryFrom<
        opentelemetry_api::metrics::AsyncInstrumentBuilder<'a, I, M>,
        Error = opentelemetry_api::metrics::MetricsError,
    >,
    I: opentelemetry_api::metrics::AsyncInstrument<M>,
{
    /// Set a description.
    pub fn with_description(
        self,
        description: impl Into<Cow<'static, str>>,
    ) -> Self {
        let Self(core, builder) = self;
        Self(core, builder.with_description(description))
    }

    /// Set a unit.
    pub fn with_unit(self, unit: opentelemetry_api::metrics::Unit) -> Self {
        let Self(core, builder) = self;
        Self(core, builder.with_unit(unit))
    }

    /// Initialize the metric.
    pub fn try_init(self) -> Result<(C, I)> {
        let Self(core, builder) = self;
        builder.try_init().map(move |r| (core, r))
    }

    /// Initialize the metric.
    pub fn init(self) -> (C, I) {
        let Self(core, builder) = self;
        (core, builder.init())
    }
}

/// Observable counter based on std::sync::atomic::AtomicU64
/// (but storing f64 bits).
#[derive(Debug, Clone)]
pub struct AtomicObservableCounterF64(Arc<AtomicU64>);

impl AtomicObservableCounterF64 {
    /// Construct a new AtomicObservableCounterF64, and associated
    /// opentelemetry metric.
    /// Note: If you would like any attributes applied to the metric reporting,
    /// please set them with the versioned_meter api before passing the meter
    /// into this constructor.
    pub fn new(
        meter: &opentelemetry_api::metrics::Meter,
        name: impl Into<std::borrow::Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableCounterF64,
        opentelemetry_api::metrics::ObservableCounter<f64>,
        f64,
    > {
        let data = Arc::new(AtomicU64::new(f64_to_u64(initial_value)));
        let builder = meter.f64_observable_counter(name);
        let data2 = data.clone();
        let builder = builder.with_callback(
            move |metric: &dyn opentelemetry_api::metrics::AsyncInstrument<
                f64,
            >| {
                metric.observe(u64_to_f64(data2.load(Ordering::SeqCst)), &[]);
            },
        );
        AtomicObservableInstrumentBuilder(Self(data), builder)
    }

    /// Add to the current value of the up down counter.
    /// a negative value is a no-op.
    pub fn add(&self, value: f64) {
        if value <= 0.0 {
            return;
        }

        // note: we don't care about the ABA problem,
        // because it will still end up with the same correct value.
        let _ = self
            .0
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                Some(f64_to_u64(u64_to_f64(v) + value))
            });
    }

    /// Get the current value of the up down counter.
    pub fn get(&self) -> f64 {
        u64_to_f64(self.0.load(Ordering::SeqCst))
    }
}

/// Observable up down counter based on std::sync::atomic::AtomicU64
/// (but storing f64 bits).
#[derive(Debug, Clone)]
pub struct AtomicObservableUpDownCounterF64(Arc<AtomicU64>);

impl AtomicObservableUpDownCounterF64 {
    /// Construct a new AtomicObservableUpDownCounterF64,
    /// and associated opentelemetry metric.
    /// Note: If you would like any attributes applied to the metric reporting,
    /// please set them with the versioned_meter api before passing the meter
    /// into this constructor.
    pub fn new(
        meter: &opentelemetry_api::metrics::Meter,
        name: impl Into<std::borrow::Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableUpDownCounterF64,
        opentelemetry_api::metrics::ObservableUpDownCounter<f64>,
        f64,
    > {
        let data = Arc::new(AtomicU64::new(f64_to_u64(initial_value)));
        let builder = meter.f64_observable_up_down_counter(name);
        let data2 = data.clone();
        let builder = builder.with_callback(
            move |metric: &dyn opentelemetry_api::metrics::AsyncInstrument<
                f64,
            >| {
                metric.observe(u64_to_f64(data2.load(Ordering::SeqCst)), &[]);
            },
        );
        AtomicObservableInstrumentBuilder(Self(data), builder)
    }

    /// Add to (or subtract from) the current value of the up down counter.
    pub fn add(&self, value: f64) {
        // note: we don't care about the ABA problem,
        // because it will still end up with the same correct value.
        let _ = self
            .0
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                Some(f64_to_u64(u64_to_f64(v) + value))
            });
    }

    /// Get the current value of the up down counter.
    pub fn get(&self) -> f64 {
        u64_to_f64(self.0.load(Ordering::SeqCst))
    }
}

/// Observable gauge based on std::sync::atomic::AtomicU64
/// (but storing f64 bits).
#[derive(Debug, Clone)]
pub struct AtomicObservableGaugeF64(Arc<AtomicU64>);

impl AtomicObservableGaugeF64 {
    /// Construct a new AtomicObservableGaugeF64, and associated opentelemetry metric.
    /// Note: If you would like any attributes applied to the metric reporting,
    /// please set them with the versioned_meter api before passing the meter
    /// into this constructor.
    pub fn new(
        meter: &opentelemetry_api::metrics::Meter,
        name: impl Into<std::borrow::Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeF64,
        opentelemetry_api::metrics::ObservableGauge<f64>,
        f64,
    > {
        let data = Arc::new(AtomicU64::new(f64_to_u64(initial_value)));
        let builder = meter.f64_observable_gauge(name);
        let data2 = data.clone();
        let builder = builder.with_callback(
            move |metric: &dyn opentelemetry_api::metrics::AsyncInstrument<
                f64,
            >| {
                metric.observe(u64_to_f64(data2.load(Ordering::SeqCst)), &[]);
            },
        );
        AtomicObservableInstrumentBuilder(Self(data), builder)
    }

    /// Set the current value of the gauge.
    pub fn set(&self, value: f64) {
        self.0.store(f64_to_u64(value), Ordering::SeqCst);
    }

    /// Get the current value of the gauge.
    pub fn get(&self) -> f64 {
        u64_to_f64(self.0.load(Ordering::SeqCst))
    }
}

/// Observable gauge based on std::sync::atomic::AtomicI64.
#[derive(Debug, Clone)]
pub struct AtomicObservableGaugeI64(Arc<AtomicI64>);

impl AtomicObservableGaugeI64 {
    /// Construct a new ObsGaugeAtomicI64, and associated opentelemetry metric.
    /// Note: If you would like any attributes applied to the metric reporting,
    /// please set them with the versioned_meter api before passing the meter
    /// into this constructor.
    pub fn new(
        meter: &opentelemetry_api::metrics::Meter,
        name: impl Into<std::borrow::Cow<'static, str>>,
        initial_value: i64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeI64,
        opentelemetry_api::metrics::ObservableGauge<i64>,
        i64,
    > {
        let data = Arc::new(AtomicI64::new(initial_value));
        let builder = meter.i64_observable_gauge(name);
        let data2 = data.clone();
        let builder = builder.with_callback(
            move |metric: &dyn opentelemetry_api::metrics::AsyncInstrument<
                i64,
            >| {
                metric.observe(data2.load(Ordering::SeqCst), &[]);
            },
        );
        AtomicObservableInstrumentBuilder(Self(data), builder)
    }

    /// Set the current value of the gauge.
    pub fn set(&self, value: i64) {
        self.0.store(value, Ordering::SeqCst);
    }

    /// Get the current value of the gauge.
    pub fn get(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Observable up down counter based on std::sync::atomic::AtomicI64.
#[derive(Debug, Clone)]
pub struct AtomicObservableUpDownCounterI64(Arc<AtomicI64>);

impl AtomicObservableUpDownCounterI64 {
    /// Construct a new AtomicObservableUpDownCounterI64,
    /// and associated opentelemetry metric.
    /// Note: If you would like any attributes applied to the metric reporting,
    /// please set them with the versioned_meter api before passing the meter
    /// into this constructor.
    pub fn new(
        meter: &opentelemetry_api::metrics::Meter,
        name: impl Into<std::borrow::Cow<'static, str>>,
        initial_value: i64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableUpDownCounterI64,
        opentelemetry_api::metrics::ObservableUpDownCounter<i64>,
        i64,
    > {
        let data = Arc::new(AtomicI64::new(initial_value));
        let builder = meter.i64_observable_up_down_counter(name);
        let data2 = data.clone();
        let builder = builder.with_callback(
            move |metric: &dyn opentelemetry_api::metrics::AsyncInstrument<
                i64,
            >| {
                metric.observe(data2.load(Ordering::SeqCst), &[]);
            },
        );
        AtomicObservableInstrumentBuilder(Self(data), builder)
    }

    /// Add to (or subtract from) the current value of the gauge.
    pub fn add(&self, value: i64) {
        self.0.fetch_add(value, Ordering::SeqCst);
    }

    /// Get the current value of the gauge.
    pub fn get(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Observable counter based on std::sync::atomic::AtomicU64.
#[derive(Debug, Clone)]
pub struct AtomicObservableCounterU64(Arc<AtomicU64>);

impl AtomicObservableCounterU64 {
    /// Construct a new AtomicObservableCounterU64,
    /// and associated opentelemetry metric.
    /// Note: If you would like any attributes applied to the metric reporting,
    /// please set them with the versioned_meter api before passing the meter
    /// into this constructor.
    pub fn new(
        meter: &opentelemetry_api::metrics::Meter,
        name: impl Into<std::borrow::Cow<'static, str>>,
        initial_value: u64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableCounterU64,
        opentelemetry_api::metrics::ObservableCounter<u64>,
        u64,
    > {
        let data = Arc::new(AtomicU64::new(initial_value));
        let builder = meter.u64_observable_counter(name);
        let data2 = data.clone();
        let builder = builder.with_callback(
            move |metric: &dyn opentelemetry_api::metrics::AsyncInstrument<
                u64,
            >| {
                metric.observe(data2.load(Ordering::SeqCst), &[]);
            },
        );
        AtomicObservableInstrumentBuilder(Self(data), builder)
    }

    /// Add to the current value of the gauge.
    pub fn add(&self, value: u64) {
        self.0.fetch_add(value, Ordering::SeqCst);
    }

    /// Get the current value of the gauge.
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Observable gauge based on std::sync::atomic::AtomicU64.
#[derive(Debug, Clone)]
pub struct AtomicObservableGaugeU64(Arc<AtomicU64>);

impl AtomicObservableGaugeU64 {
    /// Construct a new AtomicObservableGaugeU64,
    /// and associated opentelemetry metric.
    /// Note: If you would like any attributes applied to the metric reporting,
    /// please set them with the versioned_meter api before passing the meter
    /// into this constructor.
    pub fn new(
        meter: &opentelemetry_api::metrics::Meter,
        name: impl Into<std::borrow::Cow<'static, str>>,
        initial_value: u64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeU64,
        opentelemetry_api::metrics::ObservableGauge<u64>,
        u64,
    > {
        let data = Arc::new(AtomicU64::new(initial_value));
        let builder = meter.u64_observable_gauge(name);
        let data2 = data.clone();
        let builder = builder.with_callback(
            move |metric: &dyn opentelemetry_api::metrics::AsyncInstrument<
                u64,
            >| {
                metric.observe(data2.load(Ordering::SeqCst), &[]);
            },
        );
        AtomicObservableInstrumentBuilder(Self(data), builder)
    }

    /// Set the current value of the gauge.
    pub fn set(&self, value: u64) {
        self.0.store(value, Ordering::SeqCst);
    }

    /// Get the current value of the gauge.
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Extension trait for opentelemetry_api::metrics::Meter
pub trait MeterExt {
    /// Get an observable f64 up down counter backed by a
    /// std::atomic::AtomicU64.
    fn f64_observable_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableCounterF64,
        opentelemetry_api::metrics::ObservableCounter<f64>,
        f64,
    >;

    /// Get an observable f64 gauge backed by a std::atomic::AtomicU64.
    fn f64_observable_gauge_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeF64,
        opentelemetry_api::metrics::ObservableGauge<f64>,
        f64,
    >;

    /// Get an observable f64 up down counter backed by a
    /// std::atomic::AtomicU64.
    fn f64_observable_up_down_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableUpDownCounterF64,
        opentelemetry_api::metrics::ObservableUpDownCounter<f64>,
        f64,
    >;

    /// Get an observable i64 gauge backed by a std::atomic::AtomicI64.
    fn i64_observable_gauge_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: i64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeI64,
        opentelemetry_api::metrics::ObservableGauge<i64>,
        i64,
    >;

    /// Get an observable i64 up down counter backed by a std::atomic::AtomicI64.
    fn i64_observable_up_down_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: i64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableUpDownCounterI64,
        opentelemetry_api::metrics::ObservableUpDownCounter<i64>,
        i64,
    >;

    /// Get an observable u64 counter backed by a std::atomic::AtomicU64.
    fn u64_observable_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: u64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableCounterU64,
        opentelemetry_api::metrics::ObservableCounter<u64>,
        u64,
    >;

    /// Get an observable u64 gauge backed by a std::atomic::AtomicU64.
    fn u64_observable_gauge_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: u64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeU64,
        opentelemetry_api::metrics::ObservableGauge<u64>,
        u64,
    >;
}

impl MeterExt for opentelemetry_api::metrics::Meter {
    fn f64_observable_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableCounterF64,
        opentelemetry_api::metrics::ObservableCounter<f64>,
        f64,
    > {
        AtomicObservableCounterF64::new(self, name, initial_value)
    }

    fn f64_observable_gauge_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeF64,
        opentelemetry_api::metrics::ObservableGauge<f64>,
        f64,
    > {
        AtomicObservableGaugeF64::new(self, name, initial_value)
    }

    fn f64_observable_up_down_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: f64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableUpDownCounterF64,
        opentelemetry_api::metrics::ObservableUpDownCounter<f64>,
        f64,
    > {
        AtomicObservableUpDownCounterF64::new(self, name, initial_value)
    }

    fn i64_observable_gauge_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: i64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeI64,
        opentelemetry_api::metrics::ObservableGauge<i64>,
        i64,
    > {
        AtomicObservableGaugeI64::new(self, name, initial_value)
    }

    fn i64_observable_up_down_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: i64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableUpDownCounterI64,
        opentelemetry_api::metrics::ObservableUpDownCounter<i64>,
        i64,
    > {
        AtomicObservableUpDownCounterI64::new(self, name, initial_value)
    }

    fn u64_observable_counter_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: u64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableCounterU64,
        opentelemetry_api::metrics::ObservableCounter<u64>,
        u64,
    > {
        AtomicObservableCounterU64::new(self, name, initial_value)
    }

    fn u64_observable_gauge_atomic(
        &self,
        name: impl Into<Cow<'static, str>>,
        initial_value: u64,
    ) -> AtomicObservableInstrumentBuilder<
        '_,
        AtomicObservableGaugeU64,
        opentelemetry_api::metrics::ObservableGauge<u64>,
        u64,
    > {
        AtomicObservableGaugeU64::new(self, name, initial_value)
    }
}
