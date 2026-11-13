use crate::{Parameter, ParameterAccess, Scale};
use serde::{Deserialize, Serialize};

/// One connectable input port off a filter
/// Port index = position in FilterInfo::inputs
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FilterInput {
    pub label: Option<&'static str>,
    pub parameter: Option<Parameter<&'static str>>,
    /// Who owns the parameter's value. `Engine` parameters (such as
    /// `sample_rate`) are injected by the host at build time and must not be
    /// exposed as user or model controls. Absent in 0.3 metadata; defaults to `User`.
    #[serde(default)]
    pub access: ParameterAccess,
    /// Perceptual scale hint for numeric parameters, matching
    /// `NumericDomain::scale`. `None` when undeclared or non-numeric.
    #[serde(default)]
    pub scale: Option<Scale>,
}

/// Static metadata describing a filter type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FilterInfo {
    pub name: &'static str,
    pub type_id: &'static str,
    pub description: &'static str,
    /// All input ports in port-index orde: audio inputs first, then parameter ports
    pub inputs: Vec<FilterInput>,
    /// Number of output ports
    pub outputs: usize,
}

impl FilterInfo {
    pub fn audio_port_count(&self) -> usize {
        self.inputs.iter().filter(|i| i.parameter.is_none()).count()
    }
    pub fn param_port_count(&self) -> usize {
        self.inputs.iter().filter(|i| i.parameter.is_some()).count()
    }
    /// Parameter inputs a user (or model) may set, in port order. Engine-owned
    /// parameters are skipped; port indices are unaffected.
    pub fn user_params(&self) -> impl Iterator<Item = &FilterInput> {
        self.inputs
            .iter()
            .filter(|i| i.parameter.is_some() && i.access == ParameterAccess::User)
    }
}
