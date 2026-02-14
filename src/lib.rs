mod parameters;
pub use parameters::{ListSize, Literal, Parameter};

/// Static metadata describing a filter type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FilterInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub source_amount: usize,
    pub parameters: Vec<Parameter<&'static str>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MetaGenerator {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Vec<Parameter<&'static str>>,
    pub output_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MetaSink {
    pub name: &'static str,
    pub description: &'static str,
    pub input_count: usize,
}

/// Trait for filters that support named parameter modification and metadata.
/// Implemented automatically by the `FilterMetaData` derive macro.
pub trait MetaFilter {
    /// Sets a parameter by name. The default implementation is a no-op.
    fn set_parameter(&mut self, _name: &str, _value: f32) {}

    /// Returns the static metadata for this filter type.
    fn metadata() -> FilterInfo
    where
        Self: Sized;
}

use serde::{Deserialize, Serialize};
