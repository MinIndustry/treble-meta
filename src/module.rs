//! Owned, versioned module contracts. Descriptors advertise capabilities; hosts must
//! reject capabilities their audio runtime does not implement.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModuleDescriptor {
    pub type_id: String,
    pub version: u32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: Vec<ParameterDescriptor>,
    pub ports: Vec<PortDescriptor>,
    pub groups: Vec<ControlGroup>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ParameterDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub value_type: ValueType,
    pub default: ParameterValue,
    pub domain: Option<NumericDomain>,
    pub access: ParameterAccess,
    pub update: UpdateMode,
    pub smoothing_ms: Option<f64>,
    pub modulation: Option<ModulationDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ValueType {
    Float,
    Int,
    Bool,
    Enum { variants: Vec<EnumVariant> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EnumVariant {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ParameterValue {
    Float(f64),
    /// Restricted to the exact integer range shared by JSON and JavaScript.
    Int(i32),
    Bool(bool),
    Enum(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct NumericDomain {
    pub min: f64,
    pub max: f64,
    pub unit: Unit,
    pub scale: Scale,
    pub step: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Unit {
    Hertz,
    Seconds,
    Milliseconds,
    Decibels,
    Ratio,
    Q,
    Semitones,
    Cents,
    Percent,
    Unitless,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Scale {
    Linear,
    Logarithmic,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ParameterAccess {
    User,
    Engine,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum UpdateMode {
    Live,
    Rebuild,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PortDescriptor {
    pub id: String,
    pub name: String,
    pub direction: Direction,
    /// Implementation index within inputs or outputs. Never serialized into a cable.
    pub index: u32,
    pub signal: SignalType,
    pub binding: PortBinding,
    pub connections: ConnectionPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Direction {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SignalType {
    Audio {
        channels: u16,
    },
    Control {
        domain: ControlDomain,
        rate: ControlRate,
    },
    Gate,
    Note,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ControlDomain {
    Unipolar,
    Bipolar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ControlRate {
    Block,
    Sample,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum PortBinding {
    Stream,
    Parameter { parameter_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ConnectionPolicy {
    Single,
    Sum,
    FanOut,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModulationDescriptor {
    pub modes: Vec<ModulationMode>,
    pub default_mode: ModulationMode,
    pub mapping: ModulationMapping,
    pub default_depth: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ModulationMode {
    Add,
    Multiply,
    Replace,
}

/// Coordinate space in which modulation operates. See the crate README for the
/// base/depth formulas. Octaves uses log2(value); normalized uses linear domain
/// position, independently of the display scale. These are contracts, not DSP.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ModulationMapping {
    Native,
    Octaves,
    Normalized,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ControlGroup {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub presentation: GroupPresentation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum GroupPresentation {
    Generic,
    Envelope { bindings: EnvelopeBindings },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EnvelopeBindings {
    pub attack: String,
    pub decay: String,
    pub sustain: String,
    pub release: String,
}

/// Descriptor provider implemented by the forthcoming metadata derive.
/// Registry factories and DSP construction remain separate concerns.
pub trait ModuleMetadata {
    fn descriptor() -> ModuleDescriptor;
}

/// Stable enum identity contract for generated parameter bindings.
pub trait ParameterEnum: Sized {
    fn variants() -> Vec<EnumVariant>;
    fn id(&self) -> &'static str;
    fn from_id(id: &str) -> Option<Self>;
}
