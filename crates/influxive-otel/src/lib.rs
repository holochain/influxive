#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! Opentelemetry metrics bindings for influxive-child-svc.

use influxive_child_svc::*;
use std::sync::Arc;

struct InfluxiveUniMetric<
    T: 'static + std::fmt::Display + Into<DataType> + Send + Sync,
> {
    this: std::sync::Weak<Self>,
    influxive: Arc<Influxive>,
    name: std::borrow::Cow<'static, str>,
    //description: Option<std::borrow::Cow<'static, str>>,
    //unit: Option<opentelemetry_api::metrics::Unit>,
    _p: std::marker::PhantomData<T>,
}

impl<T: 'static + std::fmt::Display + Into<DataType> + Send + Sync>
    InfluxiveUniMetric<T>
{
    pub fn new(
        influxive: Arc<Influxive>,
        name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> Arc<Self> {
        Arc::new_cyclic(|this| {
            Self {
                this: this.clone(),
                influxive,
                name,
                //description,
                //unit,
                _p: std::marker::PhantomData,
            }
        })
    }

    fn report(&self, value: T, attributes: &[opentelemetry_api::KeyValue]) {
        // otel metrics are largely a single measurement... so
        // just applying them to the generic "value" name in influx.
        let mut metric =
            Metric::new(std::time::SystemTime::now(), self.name.to_string())
                .with_field("value", value);

        // these are largely not useful when viewing influx data
        // recommend just using descriptive metric names with
        // units in the name itself.
        /*
        if let Some(description) = &self.description {
            metric = metric.with_tag("description", description.to_string());
        }

        if let Some(unit) = &self.unit {
            metric = metric.with_tag("unit", unit.as_str().to_string());
        }
        */

        // everything else is a tag? would these be better as fields?
        // some kind of naming convention to pick between the two??
        for kv in attributes {
            metric = metric.with_tag(kv.key.to_string(), kv.value.to_string());
        }

        self.influxive.write_metric(metric);
    }
}

impl<T: 'static + std::fmt::Display + Into<DataType> + Send + Sync>
    opentelemetry_api::metrics::SyncCounter<T> for InfluxiveUniMetric<T>
{
    fn add(
        &self,
        _cx: &opentelemetry_api::Context,
        value: T,
        attributes: &[opentelemetry_api::KeyValue],
    ) {
        self.report(value, attributes)
    }
}

impl<T: 'static + std::fmt::Display + Into<DataType> + Send + Sync>
    opentelemetry_api::metrics::SyncUpDownCounter<T> for InfluxiveUniMetric<T>
{
    fn add(
        &self,
        _cx: &opentelemetry_api::Context,
        value: T,
        attributes: &[opentelemetry_api::KeyValue],
    ) {
        self.report(value, attributes)
    }
}

impl<T: 'static + std::fmt::Display + Into<DataType> + Send + Sync>
    opentelemetry_api::metrics::SyncHistogram<T> for InfluxiveUniMetric<T>
{
    fn record(
        &self,
        _cx: &opentelemetry_api::Context,
        value: T,
        attributes: &[opentelemetry_api::KeyValue],
    ) {
        self.report(value, attributes)
    }
}

impl<T: 'static + std::fmt::Display + Into<DataType> + Send + Sync>
    opentelemetry_api::metrics::AsyncInstrument<T> for InfluxiveUniMetric<T>
{
    fn observe(
        &self,
        measurement: T,
        attributes: &[opentelemetry_api::KeyValue],
    ) {
        self.report(measurement, attributes)
    }

    fn as_any(&self) -> Arc<dyn std::any::Any> {
        // this unwrap *should* be safe... so long as no one calls
        // Arc::into_inner() ever, which shouldn't be possible
        // because we're using trait objects everywhere??
        self.this.upgrade().unwrap()
    }
}

struct InfluxiveInstrumentProvider(Arc<Influxive>);

impl opentelemetry_api::metrics::InstrumentProvider
    for InfluxiveInstrumentProvider
{
    fn u64_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Counter<u64>,
    > {
        Ok(opentelemetry_api::metrics::Counter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn f64_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Counter<f64>,
    > {
        Ok(opentelemetry_api::metrics::Counter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn u64_observable_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<u64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableCounter<u64>,
    > {
        Ok(opentelemetry_api::metrics::ObservableCounter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn f64_observable_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<f64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableCounter<f64>,
    > {
        Ok(opentelemetry_api::metrics::ObservableCounter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn i64_up_down_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::UpDownCounter<i64>,
    > {
        Ok(opentelemetry_api::metrics::UpDownCounter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn f64_up_down_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::UpDownCounter<f64>,
    > {
        Ok(opentelemetry_api::metrics::UpDownCounter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn i64_observable_up_down_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<i64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableUpDownCounter<i64>,
    > {
        Ok(opentelemetry_api::metrics::ObservableUpDownCounter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn f64_observable_up_down_counter(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<f64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableUpDownCounter<f64>,
    > {
        Ok(opentelemetry_api::metrics::ObservableUpDownCounter::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn u64_observable_gauge(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<u64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableGauge<u64>,
    > {
        Ok(opentelemetry_api::metrics::ObservableGauge::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn i64_observable_gauge(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<i64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableGauge<i64>,
    > {
        Ok(opentelemetry_api::metrics::ObservableGauge::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn f64_observable_gauge(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<f64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableGauge<f64>,
    > {
        Ok(opentelemetry_api::metrics::ObservableGauge::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn f64_histogram(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Histogram<f64>,
    > {
        Ok(opentelemetry_api::metrics::Histogram::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn u64_histogram(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Histogram<u64>,
    > {
        Ok(opentelemetry_api::metrics::Histogram::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn i64_histogram(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Histogram<i64>,
    > {
        Ok(opentelemetry_api::metrics::Histogram::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        ))
    }

    fn register_callback(
        &self,
        _instruments: &[Arc<dyn std::any::Any>],
        callbacks: Box<
            dyn Fn(&dyn opentelemetry_api::metrics::Observer) + Send + Sync,
        >,
    ) -> opentelemetry_api::metrics::Result<
        Box<dyn opentelemetry_api::metrics::CallbackRegistration>,
    > {
        struct O;

        impl opentelemetry_api::metrics::Observer for O {
            fn observe_f64(
                &self,
                inst: &dyn opentelemetry_api::metrics::AsyncInstrument<f64>,
                measurement: f64,
                attrs: &[opentelemetry_api::KeyValue],
            ) {
                inst.observe(measurement, attrs);
            }

            fn observe_u64(
                &self,
                inst: &dyn opentelemetry_api::metrics::AsyncInstrument<u64>,
                measurement: u64,
                attrs: &[opentelemetry_api::KeyValue],
            ) {
                inst.observe(measurement, attrs);
            }

            fn observe_i64(
                &self,
                inst: &dyn opentelemetry_api::metrics::AsyncInstrument<i64>,
                measurement: i64,
                attrs: &[opentelemetry_api::KeyValue],
            ) {
                inst.observe(measurement, attrs);
            }
        }

        callbacks(&O);

        struct Null;

        impl opentelemetry_api::metrics::CallbackRegistration for Null {
            fn unregister(&mut self) -> opentelemetry_api::metrics::Result<()> {
                Ok(())
            }
        }

        Ok(Box::new(Null))
    }
}

/// InfluxiveDB Opentelemetry Meter Provider.
pub struct InfluxiveMeterProvider(Arc<Influxive>);

impl InfluxiveMeterProvider {
    /// Construct a new InfluxiveMeterProvider instance with a given
    /// "Influxive" InfluxiveDB child process connector.
    pub fn new(influxive: Arc<Influxive>) -> Self {
        Self(influxive)
    }
}

impl opentelemetry_api::metrics::MeterProvider for InfluxiveMeterProvider {
    fn versioned_meter(
        &self,
        _name: impl Into<std::borrow::Cow<'static, str>>,
        _version: Option<impl Into<std::borrow::Cow<'static, str>>>,
        _schema_url: Option<impl Into<std::borrow::Cow<'static, str>>>,
        _attributes: Option<Vec<opentelemetry_api::KeyValue>>,
    ) -> opentelemetry_api::metrics::Meter {
        opentelemetry_api::metrics::Meter::new(Arc::new(
            InfluxiveInstrumentProvider(self.0.clone()),
        ))
    }
}

#[cfg(test)]
mod test;
