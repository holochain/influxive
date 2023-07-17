#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! Opentelemetry metrics bindings for influxive-child-svc.

use influxive_child_svc::*;
use std::sync::Arc;

struct InfluxiveUniMetric<T: std::fmt::Display + Into<DataType>> {
    influxive: Arc<Influxive>,
    name: std::borrow::Cow<'static, str>,
    //description: Option<std::borrow::Cow<'static, str>>,
    //unit: Option<opentelemetry_api::metrics::Unit>,
    _p: std::marker::PhantomData<T>,
}

impl<T: std::fmt::Display + Into<DataType>> InfluxiveUniMetric<T> {
    pub fn new(
        influxive: Arc<Influxive>,
        name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> Self {
        Self {
            influxive,
            name,
            //description,
            //unit,
            _p: std::marker::PhantomData,
        }
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

impl<T: std::fmt::Display + Into<DataType>>
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

impl<T: std::fmt::Display + Into<DataType>>
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
        Ok(opentelemetry_api::metrics::Counter::new(Arc::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        )))
    }

    fn f64_counter(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Counter<f64>,
    > {
        todo!()
    }

    fn u64_observable_counter(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<u64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableCounter<u64>,
    > {
        todo!()
    }

    fn f64_observable_counter(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<f64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableCounter<f64>,
    > {
        todo!()
    }

    fn i64_up_down_counter(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::UpDownCounter<i64>,
    > {
        todo!()
    }

    fn f64_up_down_counter(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::UpDownCounter<f64>,
    > {
        todo!()
    }

    fn i64_observable_up_down_counter(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<i64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableUpDownCounter<i64>,
    > {
        todo!()
    }

    fn f64_observable_up_down_counter(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<f64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableUpDownCounter<f64>,
    > {
        todo!()
    }

    fn u64_observable_gauge(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<u64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableGauge<u64>,
    > {
        todo!()
    }

    fn i64_observable_gauge(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<i64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableGauge<i64>,
    > {
        todo!()
    }

    fn f64_observable_gauge(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
        _callback: Vec<opentelemetry_api::metrics::Callback<f64>>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::ObservableGauge<f64>,
    > {
        todo!()
    }

    fn f64_histogram(
        &self,
        name: std::borrow::Cow<'static, str>,
        description: Option<std::borrow::Cow<'static, str>>,
        unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Histogram<f64>,
    > {
        Ok(opentelemetry_api::metrics::Histogram::new(Arc::new(
            InfluxiveUniMetric::new(self.0.clone(), name, description, unit),
        )))
    }

    fn u64_histogram(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Histogram<u64>,
    > {
        todo!()
    }

    fn i64_histogram(
        &self,
        _name: std::borrow::Cow<'static, str>,
        _description: Option<std::borrow::Cow<'static, str>>,
        _unit: Option<opentelemetry_api::metrics::Unit>,
    ) -> opentelemetry_api::metrics::Result<
        opentelemetry_api::metrics::Histogram<i64>,
    > {
        todo!()
    }

    fn register_callback(
        &self,
        _instruments: &[Arc<dyn std::any::Any>],
        _callbacks: Box<
            dyn Fn(&dyn opentelemetry_api::metrics::Observer) + Send + Sync,
        >,
    ) -> opentelemetry_api::metrics::Result<
        Box<dyn opentelemetry_api::metrics::CallbackRegistration>,
    > {
        todo!()
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
