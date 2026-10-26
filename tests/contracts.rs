use treble_meta::*;

fn document() -> InstrumentDocument {
    serde_json::from_str(include_str!("../fixtures/basic.trbl")).unwrap()
}
fn descriptor() -> ModuleDescriptor {
    ModuleDescriptor {
        type_id: "example.oscillator".into(),
        version: 1,
        name: "Oscillator".into(),
        description: "Contract fixture".into(),
        category: "Sources".into(),
        parameters: vec![
            ParameterDescriptor {
                id: "frequency".into(),
                name: "Frequency".into(),
                description: "Pitch".into(),
                value_type: ValueType::Float,
                default: ParameterValue::Float(440.0),
                domain: Some(NumericDomain {
                    min: 20.0,
                    max: 20000.0,
                    unit: Unit::Hertz,
                    scale: Scale::Logarithmic,
                    step: None,
                }),
                access: ParameterAccess::User,
                update: UpdateMode::Live,
                smoothing_ms: Some(5.0),
                modulation: Some(ModulationDescriptor {
                    modes: vec![ModulationMode::Add],
                    default_mode: ModulationMode::Add,
                    mapping: ModulationMapping::Octaves,
                    default_depth: 1.0,
                }),
            },
            ParameterDescriptor {
                id: "waveform".into(),
                name: "Waveform".into(),
                description: String::new(),
                value_type: ValueType::Enum {
                    variants: vec![
                        EnumVariant {
                            id: "sine".into(),
                            name: "Sine".into(),
                        },
                        EnumVariant {
                            id: "saw".into(),
                            name: "Saw".into(),
                        },
                    ],
                },
                default: ParameterValue::Enum("sine".into()),
                domain: None,
                access: ParameterAccess::User,
                update: UpdateMode::Live,
                smoothing_ms: None,
                modulation: None,
            },
        ],
        ports: vec![
            PortDescriptor {
                id: "out".into(),
                name: "Output".into(),
                direction: Direction::Output,
                index: 0,
                signal: SignalType::Audio { channels: 2 },
                binding: PortBinding::Stream,
                connections: ConnectionPolicy::FanOut,
            },
            PortDescriptor {
                id: "frequency_cv".into(),
                name: "Pitch CV".into(),
                direction: Direction::Input,
                index: 0,
                signal: SignalType::Control {
                    domain: ControlDomain::Bipolar,
                    rate: ControlRate::Sample,
                },
                binding: PortBinding::Parameter {
                    parameter_id: "frequency".into(),
                },
                connections: ConnectionPolicy::Single,
            },
        ],
        groups: vec![],
    }
}

#[test]
fn owned_trbl_round_trip_preserves_editor_and_enum_ids() {
    let doc = document();
    doc.validate_against(&[descriptor()]).unwrap();
    let encoded = serde_json::to_string_pretty(&doc).unwrap();
    assert_eq!(
        serde_json::from_str::<InstrumentDocument>(&encoded).unwrap(),
        doc
    );
    let mut d = descriptor();
    if let ValueType::Enum { variants } = &mut d.parameters[1].value_type {
        variants.reverse();
    }
    doc.validate_against(&[d]).unwrap();
}

#[test]
fn disconnected_and_unknown_modules_remain_saveable_but_require_registry_resolution() {
    let mut doc = document();
    doc.instrument.patch.modules[0].type_id = "plugin.missing".into();
    doc.validate().unwrap();
    let e = doc.validate_against(&[descriptor()]).unwrap_err();
    assert!(e.to_string().contains("unavailable module"));
    serde_json::to_string(&doc).unwrap();
}

#[test]
fn future_formats_are_rejected_without_silent_migration() {
    let mut doc = document();
    doc.format_version = 99;
    assert_eq!(doc.validate().unwrap_err().0[0].path, "format_version");
}

#[test]
fn descriptor_collects_precise_independent_errors() {
    let mut d = descriptor();
    d.parameters[0].default = ParameterValue::Float(30000.0);
    d.parameters[0].domain.as_mut().unwrap().min = -1.0;
    d.parameters[0].modulation.as_mut().unwrap().default_depth = f64::NAN;
    d.parameters.push(d.parameters[1].clone());
    d.ports[1].binding = PortBinding::Parameter {
        parameter_id: "missing".into(),
    };
    let e = d.validate().unwrap_err();
    for path in [
        "parameters[0].default",
        "parameters[0].domain.min",
        "parameters[0].modulation.default_depth",
        "parameters[2].id",
        "ports[1].binding.parameter_id",
    ] {
        assert!(e.0.iter().any(|e| e.path == path), "missing {path}: {e}");
    }
}

#[test]
fn parameter_checks_reject_coercion_and_invalid_values() {
    let d = descriptor();
    let p = &d.parameters[0];
    for v in [
        ParameterValue::Int(440),
        ParameterValue::Float(f64::INFINITY),
        ParameterValue::Float(0.0),
    ] {
        assert!(p.validate_value(&v).is_err());
    }
    assert!(
        d.parameters[1]
            .validate_value(&ParameterValue::Enum("square".into()))
            .is_err()
    );
    p.validate_value(&ParameterValue::Float(440.0)).unwrap();
}

fn cable() -> Cable {
    Cable {
        id: "c1".into(),
        from: Endpoint {
            module: "lfo".into(),
            port: "cv".into(),
        },
        to: Endpoint {
            module: "osc-1".into(),
            port: "frequency_cv".into(),
        },
        modulation: Some(ModulationSettings {
            mode: ModulationMode::Add,
            depth: 2.0,
        }),
    }
}
fn modulated() -> (InstrumentDocument, Vec<ModuleDescriptor>) {
    let mut doc = document();
    let mut lfo = descriptor();
    lfo.type_id = "example.lfo".into();
    lfo.ports = vec![PortDescriptor {
        id: "cv".into(),
        name: "CV".into(),
        direction: Direction::Output,
        index: 0,
        signal: SignalType::Control {
            domain: ControlDomain::Bipolar,
            rate: ControlRate::Sample,
        },
        binding: PortBinding::Stream,
        connections: ConnectionPolicy::FanOut,
    }];
    doc.instrument.patch.modules.push(ModuleInstance {
        id: "lfo".into(),
        type_id: lfo.type_id.clone(),
        type_version: 1,
        parameters: Default::default(),
    });
    doc.instrument.patch.cables.push(cable());
    (doc, vec![descriptor(), lfo])
}

#[test]
fn cables_validate_types_cardinality_and_modulation_without_mutating_base() {
    let (mut doc, registry) = modulated();
    doc.validate_against(&registry).unwrap();
    assert_eq!(
        doc.instrument.patch.modules[0].parameters["frequency"],
        ParameterValue::Float(440.0)
    );
    let mut second = cable();
    second.id = "c2".into();
    doc.instrument.patch.cables.push(second);
    let errors = doc.validate_against(&registry).unwrap_err().to_string();
    assert!(errors.contains("single input already has a cable"));
    assert!(errors.contains("duplicate cable endpoints"));
    doc.instrument.patch.cables.pop();
    doc.instrument.patch.cables[0]
        .modulation
        .as_mut()
        .unwrap()
        .mode = ModulationMode::Replace;
    assert!(
        doc.validate_against(&registry)
            .unwrap_err()
            .to_string()
            .contains("mode is not supported")
    );
    doc.instrument.patch.cables[0].from = Endpoint {
        module: "osc-1".into(),
        port: "out".into(),
    };
    assert!(
        doc.validate_against(&registry)
            .unwrap_err()
            .to_string()
            .contains("signal types must match")
    );
}

#[test]
fn stable_ids_survive_descriptor_reordering() {
    let (doc, mut registry) = modulated();
    for d in &mut registry {
        d.parameters.reverse();
        d.ports.reverse();
    }
    doc.validate_against(&registry).unwrap();
}

#[test]
fn context_values_and_bad_interfaces_are_rejected() {
    let mut d = descriptor();
    d.parameters[1].access = ParameterAccess::Engine;
    let mut doc = document();
    doc.instrument.interface.audio_outputs[0].port = "frequency_cv".into();
    doc.instrument.interface.note_input = Some(Endpoint {
        module: "osc-1".into(),
        port: "out".into(),
    });
    let errors = doc.validate_against(&[d]).unwrap_err().to_string();
    assert!(errors.contains("engine-owned parameters"));
    assert!(errors.contains("audio interface requires"));
    assert!(errors.contains("note interface requires"));
}

#[test]
fn envelope_bindings_are_validated_against_numeric_members() {
    let mut d = descriptor();
    d.groups.push(ControlGroup {
        id: "env".into(),
        name: "Envelope".into(),
        members: vec!["frequency".into(), "waveform".into()],
        presentation: GroupPresentation::Envelope {
            bindings: EnvelopeBindings {
                attack: "frequency".into(),
                decay: "waveform".into(),
                sustain: "missing".into(),
                release: "frequency".into(),
            },
        },
    });
    let errors = d.validate().unwrap_err().to_string();
    assert!(errors.contains("requires a numeric parameter"));
    assert!(errors.contains("must belong to this group"));
    assert!(errors.contains("must bind distinct"));
}

#[cfg(feature = "ts")]
#[test]
fn typescript_exports_preserve_tagged_contracts() {
    use ts_rs::TS;
    let value = ParameterValue::decl(&ts_rs::Config::default());
    assert!(value.contains("kind"));
    assert!(value.contains("float"));
    assert!(value.contains("value"));
    let doc = InstrumentDocument::decl(&ts_rs::Config::default());
    assert!(doc.contains("format_version"));
    assert!(doc.contains("editor"));
    assert!(SignalType::decl(&ts_rs::Config::default()).contains("channels"));
}

#[test]
fn structural_validation_reports_broken_references_ids_and_voice_count() {
    let mut doc = document();
    doc.instrument.voice = VoiceConfiguration::Polyphonic { max_voices: 0 };
    doc.instrument
        .patch
        .modules
        .push(doc.instrument.patch.modules[0].clone());
    doc.instrument.patch.cables.push(cable());
    doc.instrument.patch.cables[0]
        .modulation
        .as_mut()
        .unwrap()
        .depth = f64::NAN;
    let errors = doc.validate().unwrap_err();
    for path in [
        "instrument.voice.max_voices",
        "instrument.patch.modules[1].id",
        "instrument.patch.cables[0].from.module",
        "instrument.patch.cables[0].modulation.depth",
    ] {
        assert!(
            errors.0.iter().any(|e| e.path == path),
            "missing {path}: {errors}"
        );
    }
}

#[test]
fn port_and_modulation_invariants_are_checked_before_registry_use() {
    let mut d = descriptor();
    d.ports.push(d.ports[1].clone());
    d.ports[0].connections = ConnectionPolicy::Single;
    d.ports[0].signal = SignalType::Audio { channels: 0 };
    d.parameters[0].modulation.as_mut().unwrap().modes.clear();
    let errors = d.validate().unwrap_err().to_string();
    for message in [
        "duplicate ID",
        "duplicate implementation index",
        "multiple ports cannot bind",
        "outputs require fan_out",
        "at least one channel",
        "default mode must appear",
    ] {
        assert!(errors.contains(message), "{message}: {errors}");
    }
}

#[test]
fn summing_parameter_inputs_require_one_shared_modulation_mode() {
    let (mut doc, mut registry) = modulated();
    registry[0].ports[1].connections = ConnectionPolicy::Sum;
    registry[0].parameters[0]
        .modulation
        .as_mut()
        .unwrap()
        .modes
        .push(ModulationMode::Replace);
    let mut second_source = doc.instrument.patch.modules[1].clone();
    second_source.id = "lfo-2".into();
    doc.instrument.patch.modules.push(second_source);
    let mut second = cable();
    second.id = "c2".into();
    second.from.module = "lfo-2".into();
    doc.instrument.patch.cables.push(second);
    doc.validate_against(&registry).unwrap();
    doc.instrument.patch.cables[1]
        .modulation
        .as_mut()
        .unwrap()
        .mode = ModulationMode::Replace;
    assert!(
        doc.validate_against(&registry)
            .unwrap_err()
            .to_string()
            .contains("must use the same mode")
    );
}
