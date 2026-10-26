//! Portable instrument documents. Parsing and saving do not require a runnable patch.
use crate::{ModulationMode, ParameterValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INSTRUMENT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct InstrumentDocument {
    pub format_version: u32,
    pub instrument: InstrumentDefinition,
    /// Opaque app-owned layout, preserved on round-trip and ignored by DSP validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct InstrumentDefinition {
    pub name: String,
    pub description: String,
    pub patch: Patch,
    pub interface: InstrumentInterface,
    pub voice: VoiceConfiguration,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Patch {
    pub modules: Vec<ModuleInstance>,
    pub cables: Vec<Cable>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModuleInstance {
    pub id: String,
    pub type_id: String,
    pub type_version: u32,
    /// User base values, never mutated by modulation. Missing entries use defaults.
    pub parameters: BTreeMap<String, ParameterValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Cable {
    pub id: String,
    pub from: Endpoint,
    pub to: Endpoint,
    /// None selects the target parameter's advertised defaults, when applicable.
    pub modulation: Option<ModulationSettings>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Endpoint {
    pub module: String,
    pub port: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModulationSettings {
    pub mode: ModulationMode,
    pub depth: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct InstrumentInterface {
    pub note_input: Option<Endpoint>,
    pub audio_outputs: Vec<Endpoint>,
    pub controls: Vec<ExposedControl>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ExposedControl {
    pub id: String,
    pub name: String,
    pub target: ParameterTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ParameterTarget {
    pub module: String,
    pub parameter: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum VoiceConfiguration {
    Continuous,
    Monophonic,
    Polyphonic { max_voices: u16 },
}
