use super::*;

/// String type handling various string types usable by InfluxDB.
#[derive(Debug, Clone)]
pub enum StringType {
    /// String value.
    String(Cow<'static, str>),

    /// String value.
    ArcString(Arc<str>),
}

impl StringType {
    pub(crate) fn into_string(self) -> String {
        match self {
            StringType::String(s) => s.into_owned(),
            StringType::ArcString(s) => s.to_string(),
        }
    }
}

macro_rules! stringtype_from_impl {
    ($($f:ty, $i:ident, $b:block,)*) => {$(
        impl From<$f> for StringType {
            fn from($i: $f) -> Self $b
        }
    )*};
}

stringtype_from_impl! {
    String, f, { StringType::String(Cow::Owned(f)) },
    &'static String, f, { StringType::String(Cow::Borrowed(f.as_str())) },
    &'static str, f, { StringType::String(Cow::Borrowed(f)) },
    Cow<'static, str>, f, { StringType::String(f) },
    Arc<str>, f, { StringType::ArcString(f) },
}

/// Field-type enum for sending data to InfluxDB.
#[derive(Debug, Clone)]
pub enum DataType {
    /// Bool value.
    Bool(bool),

    /// Float value.
    F64(f64),

    /// Signed integer value.
    I64(i64),

    /// Unsigned integer value.
    U64(u64),

    /// String value.
    String(StringType),
}

impl DataType {
    pub(crate) fn into_type(self) -> influxdb::Type {
        match self {
            DataType::Bool(b) => influxdb::Type::Boolean(b),
            DataType::F64(f) => influxdb::Type::Float(f),
            DataType::I64(i) => influxdb::Type::SignedInteger(i),
            DataType::U64(u) => influxdb::Type::UnsignedInteger(u),
            DataType::String(s) => influxdb::Type::Text(s.into_string()),
        }
    }
}

macro_rules! datatype_from_impl {
    ($($f:ty, $i:ident, $b:block,)*) => {$(
        impl From<$f> for DataType {
            fn from($i: $f) -> Self $b
        }
    )*};
}

datatype_from_impl! {
    bool, f, { DataType::Bool(f) },
    f64, f, { DataType::F64(f) },
    f32, f, { DataType::F64(f as f64) },
    i8, f, { DataType::I64(f as i64) },
    i16, f, { DataType::I64(f as i64) },
    i32, f, { DataType::I64(f as i64) },
    i64, f, { DataType::I64(f as i64) },
    u8, f, { DataType::U64(f as u64) },
    u16, f, { DataType::U64(f as u64) },
    u32, f, { DataType::U64(f as u64) },
    u64, f, { DataType::U64(f as u64) },
    String, f, { DataType::String(f.into()) },
    &'static String, f, { DataType::String(f.into()) },
    &'static str, f, { DataType::String(f.into()) },
    Cow<'static, str>, f, { DataType::String(f.into()) },
    Arc<str>, f, { DataType::String(f.into()) },
}

/// A metric to record in the influxdb instance.
#[derive(Debug)]
pub struct Metric {
    /// The timestamp for this metric report.
    pub timestamp: std::time::SystemTime,

    /// The name of this metric report.
    pub name: StringType,

    /// The fields associated with this metric report.
    pub fields: Vec<(StringType, DataType)>,

    /// The tags associated with this metric report.
    pub tags: Vec<(StringType, DataType)>,
}

impl Metric {
    /// Construct a new metric report to be sent to InfluxDB.
    pub fn new<N: Into<StringType>>(
        timestamp: std::time::SystemTime,
        name: N,
    ) -> Metric {
        Self {
            timestamp,
            name: name.into(),
            fields: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Add a field to this metric report.
    pub fn with_field<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<StringType>,
        V: Into<DataType>,
    {
        self.fields.push((name.into(), value.into()));
        self
    }

    /// Add a tag to this metric report.
    pub fn with_tag<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<StringType>,
        V: Into<DataType>,
    {
        self.tags.push((name.into(), value.into()));
        self
    }
}
