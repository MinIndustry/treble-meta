mod parameters;
pub use parameters::{ListSize, Literal, Parameter};

/// Static metadata describing a filter type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub source_amount: usize,
    pub parameters: Vec<Parameter<&'static str>>,
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
