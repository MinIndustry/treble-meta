mod filter;
mod parameters;

pub use filter::{FilterInfo, FilterInput};
pub use parameters::{ListSize, Literal, Parameter};

/// Strategy for combining multiple audio blocks arriving at the same input port.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum MixMode {
    /// Sum all frames element-wise (default, standard audio mixing).
    #[default]
    Sum,
    /// Average all frames element-wise.
    Average,
    /// Element-wise maximum.
    Max,
    /// Element-wise minimum.
    Min,
}

impl MixMode {
    pub fn from_ordinal(n: usize) -> Self {
        match n {
            1 => Self::Average,
            2 => Self::Max,
            3 => Self::Min,
            _ => Self::Sum,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MetaGenerator {
    /// Human-readable display name shown in the graph editor.
    pub name: &'static str,
    /// Machine-readable identifier sent to the backend when adding a node.
    pub type_id: &'static str,
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
