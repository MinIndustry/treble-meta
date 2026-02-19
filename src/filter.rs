use serde::{Deserialize, Serialize};
use crate::Parameter;

/// One connectable input port off a filter
/// Port index = position in FilterInfo::inputs
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FilterInput {
    pub label: Option<&'static str>,
    pub parameter: Option<Parameter<&'static str>>
}

/// Static metadata describing a filter type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FilterInfo {
    pub name: &'static str,
    pub description: &'static str,
    /// All input ports in port-index orde: audio inputs first, then parameter ports
    pub inputs: Vec<FilterInput>,
    /// Number of output ports
    pub outputs: usize,
}

impl FilterInfo {
    pub fn audio_port_count(&self) -> usize { self.inputs.iter().filter(|i| i.parameter.is_none()).count() }
    pub fn param_port_count(&self) -> usize { self.inputs.iter().filter(|i| i.parameter.is_some()).count() }
}
