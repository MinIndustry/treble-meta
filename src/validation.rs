use crate::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

/// Stable document path and actionable explanation. Several errors may be reported together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl ValidationErrors {
    fn push(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.0.push(ValidationError {
            path: path.into(),
            message: message.into(),
        });
    }
    fn finish(self) -> Result<(), Self> {
        if self.0.is_empty() { Ok(()) } else { Err(self) }
    }
    fn id(&mut self, path: &str, id: &str, seen: &mut BTreeSet<String>) {
        if id.is_empty() || id.trim() != id {
            self.push(
                path,
                "ID must be nonempty and have no surrounding whitespace",
            );
        }
        if !seen.insert(id.to_owned()) {
            self.push(path, format!("duplicate ID `{id}`"));
        }
    }
}
impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, error) in self.0.iter().enumerate() {
            if i != 0 {
                writeln!(f)?;
            }
            write!(f, "{}: {}", error.path, error.message)?;
        }
        Ok(())
    }
}
impl std::error::Error for ValidationErrors {}

impl ParameterDescriptor {
    /// Validate one supplied base value without applying or clamping it.
    pub fn validate_value(&self, value: &ParameterValue) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::default();
        check_value(self, value, &self.id, &mut errors);
        errors.finish()
    }
}
fn check_value(
    parameter: &ParameterDescriptor,
    value: &ParameterValue,
    path: &str,
    errors: &mut ValidationErrors,
) {
    let number = match (&parameter.value_type, value) {
        (ValueType::Float, ParameterValue::Float(n)) => Some(*n),
        (ValueType::Int, ParameterValue::Int(n)) => Some(f64::from(*n)),
        (ValueType::Bool, ParameterValue::Bool(_)) => None,
        (ValueType::Enum { variants }, ParameterValue::Enum(id)) => {
            if !variants.iter().any(|v| v.id == *id) {
                errors.push(path, format!("unknown enum value `{id}`"));
            }
            None
        }
        _ => {
            errors.push(path, "value does not match parameter type");
            None
        }
    };
    if let Some(n) = number {
        if !n.is_finite() {
            errors.push(path, "numeric value must be finite");
        } else if let Some(d) = &parameter.domain
            && (n < d.min || n > d.max)
        {
            errors.push(path, format!("value {n} is outside [{}, {}]", d.min, d.max));
        }
    }
}

impl ModuleDescriptor {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut e = ValidationErrors::default();
        e.id("type_id", &self.type_id, &mut BTreeSet::new());
        if self.version == 0 {
            e.push("version", "module version must be positive");
        }
        let mut parameters = BTreeSet::new();
        for (i, p) in self.parameters.iter().enumerate() {
            let path = format!("parameters[{i}]");
            e.id(&format!("{path}.id"), &p.id, &mut parameters);
            check_value(p, &p.default, &format!("{path}.default"), &mut e);
            let numeric = matches!(p.value_type, ValueType::Float | ValueType::Int);
            if let Some(d) = &p.domain {
                if !numeric {
                    e.push(
                        format!("{path}.domain"),
                        "only numeric parameters can have a domain",
                    );
                }
                if !d.min.is_finite() || !d.max.is_finite() || d.min >= d.max {
                    e.push(
                        format!("{path}.domain"),
                        "bounds must be finite and min must be less than max",
                    );
                }
                if d.scale == Scale::Logarithmic && d.min <= 0.0 {
                    e.push(
                        format!("{path}.domain.min"),
                        "logarithmic bounds must be positive",
                    );
                }
                if d.step.is_some_and(|s| !s.is_finite() || s <= 0.0) {
                    e.push(
                        format!("{path}.domain.step"),
                        "step must be finite and positive",
                    );
                }
                if p.value_type == ValueType::Int
                    && (d.min.fract() != 0.0
                        || d.max.fract() != 0.0
                        || d.step.is_some_and(|s| s.fract() != 0.0))
                {
                    e.push(
                        format!("{path}.domain"),
                        "integer bounds and step must be whole numbers",
                    );
                }
            }
            if let ValueType::Enum { variants } = &p.value_type {
                if variants.is_empty() {
                    e.push(
                        format!("{path}.value_type.variants"),
                        "enum requires at least one variant",
                    );
                }
                let mut ids = BTreeSet::new();
                for (j, v) in variants.iter().enumerate() {
                    e.id(
                        &format!("{path}.value_type.variants[{j}].id"),
                        &v.id,
                        &mut ids,
                    );
                }
            }
            if p.smoothing_ms.is_some_and(|s| !s.is_finite() || s < 0.0) {
                e.push(
                    format!("{path}.smoothing_ms"),
                    "smoothing must be finite and nonnegative",
                );
            }
            if p.smoothing_ms.is_some() && (!numeric || p.update != UpdateMode::Live) {
                e.push(
                    format!("{path}.smoothing_ms"),
                    "smoothing requires a live numeric parameter",
                );
            }
            if let Some(m) = &p.modulation {
                if !numeric || p.access != ParameterAccess::User || p.update != UpdateMode::Live {
                    e.push(
                        format!("{path}.modulation"),
                        "modulation requires a live user-owned numeric parameter",
                    );
                }
                if m.modes.is_empty() || !m.modes.contains(&m.default_mode) {
                    e.push(
                        format!("{path}.modulation.default_mode"),
                        "default mode must appear in a nonempty modes list",
                    );
                }
                for (j, mode) in m.modes.iter().enumerate() {
                    if m.modes[..j].contains(mode) {
                        e.push(
                            format!("{path}.modulation.modes[{j}]"),
                            "duplicate modulation mode",
                        );
                    }
                }
                if !m.default_depth.is_finite() {
                    e.push(
                        format!("{path}.modulation.default_depth"),
                        "depth must be finite",
                    );
                }
                if m.mapping == ModulationMapping::Normalized && p.domain.is_none() {
                    e.push(
                        format!("{path}.modulation.mapping"),
                        "normalized modulation requires a numeric domain",
                    );
                }
                if m.mapping == ModulationMapping::Octaves
                    && !p.domain.as_ref().is_some_and(|d| d.min > 0.0)
                {
                    e.push(
                        format!("{path}.modulation.mapping"),
                        "octave modulation requires a positive numeric domain",
                    );
                }
            }
        }
        let mut ports = BTreeSet::new();
        let mut indices = BTreeSet::new();
        let mut bound_parameters = BTreeSet::new();
        for (i, p) in self.ports.iter().enumerate() {
            let path = format!("ports[{i}]");
            e.id(&format!("{path}.id"), &p.id, &mut ports);
            if !indices.insert((p.direction == Direction::Input, p.index)) {
                e.push(
                    format!("{path}.index"),
                    "duplicate implementation index within direction",
                );
            }
            if matches!(p.signal, SignalType::Audio { channels: 0 }) {
                e.push(
                    format!("{path}.signal.channels"),
                    "audio requires at least one channel",
                );
            }
            if (p.direction == Direction::Output) != (p.connections == ConnectionPolicy::FanOut) {
                e.push(
                    format!("{path}.connections"),
                    "outputs require fan_out; inputs require single or sum",
                );
            }
            if p.connections == ConnectionPolicy::Sum
                && !matches!(
                    p.signal,
                    SignalType::Audio { .. } | SignalType::Control { .. }
                )
            {
                e.push(
                    format!("{path}.connections"),
                    "sum is only defined for audio and control signals",
                );
            }
            if let PortBinding::Parameter { parameter_id } = &p.binding {
                if !bound_parameters.insert(parameter_id) {
                    e.push(
                        format!("{path}.binding"),
                        "multiple ports cannot bind the same parameter; use a summing input",
                    );
                }
                if p.direction != Direction::Input
                    || !matches!(p.signal, SignalType::Control { .. })
                {
                    e.push(
                        format!("{path}.binding"),
                        "parameter binding requires a control input",
                    );
                }
                match self.parameters.iter().find(|x| x.id == *parameter_id) {
                    None => e.push(
                        format!("{path}.binding.parameter_id"),
                        format!("unknown parameter `{parameter_id}`"),
                    ),
                    Some(parameter) if parameter.modulation.is_none() => e.push(
                        format!("{path}.binding"),
                        "bound parameter must advertise modulation",
                    ),
                    _ => (),
                }
            }
        }
        let mut groups = BTreeSet::new();
        for (i, g) in self.groups.iter().enumerate() {
            let path = format!("groups[{i}]");
            e.id(&format!("{path}.id"), &g.id, &mut groups);
            let mut members = BTreeSet::new();
            for (j, id) in g.members.iter().enumerate() {
                if !parameters.contains(id) {
                    e.push(
                        format!("{path}.members[{j}]"),
                        format!("unknown parameter `{id}`"),
                    );
                }
                if !members.insert(id) {
                    e.push(format!("{path}.members[{j}]"), "duplicate group member");
                }
            }
            if let GroupPresentation::Envelope { bindings: b } = &g.presentation {
                let mut bindings = BTreeSet::new();
                for (role, id) in [
                    ("attack", &b.attack),
                    ("decay", &b.decay),
                    ("sustain", &b.sustain),
                    ("release", &b.release),
                ] {
                    let key = format!("{path}.presentation.bindings.{role}");
                    if !bindings.insert(id) {
                        e.push(&key, "envelope roles must bind distinct parameters");
                    }
                    if !members.contains(id) {
                        e.push(&key, format!("parameter `{id}` must belong to this group"));
                    }
                    if !self.parameters.iter().any(|p| {
                        p.id == *id && matches!(p.value_type, ValueType::Float | ValueType::Int)
                    }) {
                        e.push(key, "envelope binding requires a numeric parameter");
                    }
                }
            }
        }
        e.finish()
    }
}

impl InstrumentDocument {
    /// Structural validation only: incomplete and disconnected instruments remain saveable.
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut e = ValidationErrors::default();
        if self.format_version != INSTRUMENT_FORMAT_VERSION {
            e.push(
                "format_version",
                format!(
                    "unsupported format version {}; expected {INSTRUMENT_FORMAT_VERSION}",
                    self.format_version
                ),
            );
        }
        if matches!(
            self.instrument.voice,
            VoiceConfiguration::Polyphonic { max_voices: 0 }
        ) {
            e.push(
                "instrument.voice.max_voices",
                "voice count must be positive",
            );
        }
        let patch = &self.instrument.patch;
        let mut modules = BTreeSet::new();
        for (i, m) in patch.modules.iter().enumerate() {
            let path = format!("instrument.patch.modules[{i}]");
            e.id(&format!("{path}.id"), &m.id, &mut modules);
            e.id(&format!("{path}.type_id"), &m.type_id, &mut BTreeSet::new());
            if m.type_version == 0 {
                e.push(
                    format!("{path}.type_version"),
                    "module version must be positive",
                );
            }
            for (id, value) in &m.parameters {
                if let ParameterValue::Float(n) = value
                    && !n.is_finite()
                {
                    e.push(
                        format!("{path}.parameters.{id}"),
                        "numeric value must be finite",
                    );
                }
            }
        }
        let mut cables = BTreeSet::new();
        for (i, c) in patch.cables.iter().enumerate() {
            let path = format!("instrument.patch.cables[{i}]");
            e.id(&format!("{path}.id"), &c.id, &mut cables);
            for (side, endpoint) in [("from", &c.from), ("to", &c.to)] {
                check_endpoint(endpoint, &format!("{path}.{side}"), &modules, &mut e);
            }
            if c.modulation.as_ref().is_some_and(|m| !m.depth.is_finite()) {
                e.push(format!("{path}.modulation.depth"), "depth must be finite");
            }
        }
        let interface = &self.instrument.interface;
        if let Some(note) = &interface.note_input {
            check_endpoint(note, "instrument.interface.note_input", &modules, &mut e);
        }
        for (i, output) in interface.audio_outputs.iter().enumerate() {
            check_endpoint(
                output,
                &format!("instrument.interface.audio_outputs[{i}]"),
                &modules,
                &mut e,
            );
        }
        let mut controls = BTreeSet::new();
        for (i, c) in interface.controls.iter().enumerate() {
            let path = format!("instrument.interface.controls[{i}]");
            e.id(&format!("{path}.id"), &c.id, &mut controls);
            if !modules.contains(&c.target.module) {
                e.push(format!("{path}.target.module"), "unknown module instance");
            }
            e.id(
                &format!("{path}.target.parameter"),
                &c.target.parameter,
                &mut BTreeSet::new(),
            );
        }
        e.finish()
    }

    /// Resolve exact descriptor versions and validate values and wiring. This does
    /// not compile DSP, detect feedback, or promise host capability support.
    pub fn validate_against(&self, registry: &[ModuleDescriptor]) -> Result<(), ValidationErrors> {
        let mut e = self.validate().err().unwrap_or_default();
        let mut types = BTreeSet::new();
        for (i, d) in registry.iter().enumerate() {
            if !types.insert((&d.type_id, d.version)) {
                e.push(format!("registry[{i}]"), "duplicate module type/version");
            }
            if let Err(errors) = d.validate() {
                for error in errors.0 {
                    e.push(format!("registry[{i}].{}", error.path), error.message);
                }
            }
        }
        let patch = &self.instrument.patch;
        let mut resolved = BTreeMap::new();
        for (i, m) in patch.modules.iter().enumerate() {
            let path = format!("instrument.patch.modules[{i}]");
            let Some(d) = registry
                .iter()
                .find(|d| d.type_id == m.type_id && d.version == m.type_version)
            else {
                e.push(
                    format!("{path}.type_id"),
                    format!(
                        "unavailable module `{}` version {}",
                        m.type_id, m.type_version
                    ),
                );
                continue;
            };
            resolved.insert(m.id.as_str(), d);
            for (id, value) in &m.parameters {
                let key = format!("{path}.parameters.{id}");
                match d.parameters.iter().find(|p| p.id == *id) {
                    None => e.push(key, "unknown parameter"),
                    Some(p) if p.access == ParameterAccess::Engine => e.push(
                        key,
                        "engine-owned parameters cannot be stored as user values",
                    ),
                    Some(p) => check_value(p, value, &key, &mut e),
                }
            }
        }
        let mut incoming = BTreeMap::new();
        let mut modulation_modes = BTreeMap::new();
        let mut edges = BTreeSet::new();
        for (i, c) in patch.cables.iter().enumerate() {
            let path = format!("instrument.patch.cables[{i}]");
            if !edges.insert((&c.from.module, &c.from.port, &c.to.module, &c.to.port)) {
                e.push(&path, "duplicate cable endpoints");
            }
            let from = resolve_port(&c.from, &format!("{path}.from"), &resolved, &mut e);
            let to = resolve_port(&c.to, &format!("{path}.to"), &resolved, &mut e);
            if let Some(p) = from
                && p.direction != Direction::Output
            {
                e.push(format!("{path}.from"), "cable source must be an output");
            }
            if let Some(p) = to {
                if p.direction != Direction::Input {
                    e.push(format!("{path}.to"), "cable target must be an input");
                }
                let count = incoming.entry((&c.to.module, &c.to.port)).or_insert(0);
                *count += 1;
                if p.connections == ConnectionPolicy::Single && *count > 1 {
                    e.push(format!("{path}.to"), "single input already has a cable");
                }
                match &p.binding {
                    PortBinding::Stream if c.modulation.is_some() => e.push(
                        format!("{path}.modulation"),
                        "modulation settings require a parameter-bound input",
                    ),
                    PortBinding::Parameter { parameter_id } => {
                        if let Some(d) = resolved
                            .get(c.to.module.as_str())
                            .and_then(|d| d.parameters.iter().find(|p| p.id == *parameter_id))
                            .and_then(|p| p.modulation.as_ref())
                        {
                            let mode = c.modulation.as_ref().map_or(&d.default_mode, |m| &m.mode);
                            if !d.modes.contains(mode) {
                                e.push(
                                    format!("{path}.modulation.mode"),
                                    "mode is not supported by the target parameter",
                                );
                            }
                            if let Some(previous) =
                                modulation_modes.insert((&c.to.module, &c.to.port), mode)
                                && previous != mode
                            {
                                e.push(
                                    format!("{path}.modulation.mode"),
                                    "cables sharing a parameter input must use the same mode",
                                );
                            }
                        }
                    }
                    _ => (),
                }
            }
            if let (Some(from), Some(to)) = (from, to)
                && from.signal != to.signal
            {
                e.push(
                    &path,
                    "signal types must match exactly; add an explicit adapter module",
                );
            }
        }
        let interface = &self.instrument.interface;
        if let Some(note) = &interface.note_input
            && let Some(p) =
                resolve_port(note, "instrument.interface.note_input", &resolved, &mut e)
            && (p.direction != Direction::Input || p.signal != SignalType::Note)
        {
            e.push(
                "instrument.interface.note_input",
                "note interface requires a note input",
            );
        }
        for (i, output) in interface.audio_outputs.iter().enumerate() {
            let path = format!("instrument.interface.audio_outputs[{i}]");
            if let Some(p) = resolve_port(output, &path, &resolved, &mut e)
                && (p.direction != Direction::Output
                    || !matches!(p.signal, SignalType::Audio { .. }))
            {
                e.push(path, "audio interface requires an audio output");
            }
        }
        for (i, control) in interface.controls.iter().enumerate() {
            if let Some(d) = resolved.get(control.target.module.as_str())
                && !d
                    .parameters
                    .iter()
                    .any(|p| p.id == control.target.parameter && p.access == ParameterAccess::User)
            {
                e.push(
                    format!("instrument.interface.controls[{i}].target.parameter"),
                    "exposed control requires a user-owned parameter",
                );
            }
        }
        e.finish()
    }
}
fn check_endpoint(
    endpoint: &Endpoint,
    path: &str,
    modules: &BTreeSet<String>,
    e: &mut ValidationErrors,
) {
    if !modules.contains(&endpoint.module) {
        e.push(
            format!("{path}.module"),
            format!("unknown module instance `{}`", endpoint.module),
        );
    }
    e.id(
        &format!("{path}.port"),
        &endpoint.port,
        &mut BTreeSet::new(),
    );
}
fn resolve_port<'a>(
    endpoint: &Endpoint,
    path: &str,
    modules: &BTreeMap<&str, &'a ModuleDescriptor>,
    e: &mut ValidationErrors,
) -> Option<&'a PortDescriptor> {
    let d = modules.get(endpoint.module.as_str())?;
    let port = d.ports.iter().find(|p| p.id == endpoint.port);
    if port.is_none() {
        e.push(
            format!("{path}.port"),
            format!("unknown port `{}`", endpoint.port),
        );
    }
    port
}
