# Treble Meta

Portable module descriptions and instrument documents for Treble. Version 0.3.0
introduces owned, versioned contracts for a rack editor while preserving the
legacy filter API for the staged derive/runtime migration.

This crate contains data, serde/TypeScript representations and validation. It does
not process audio, register DSP factories, compile graphs, schedule recompilation,
implement smoothing or modulation, or guarantee that a host supports every
advertised capability.

## Module contracts

`ModuleDescriptor` unifies sources, processors and sinks. Its `type_id` and
`version` identify an exact contract. All parameter, port, enum variant and group
IDs are stable strings independent of Rust field names, vector order or display
labels. Changing a published contract incompatibly requires a module version bump.

- `ParameterDescriptor` separates typed defaults (`Float`, `Int`, `Bool`, stable
  `Enum` IDs), numeric domains/units/scales, user/engine ownership, live/rebuild
  updates, smoothing requests and optional modulation capabilities.
- `PortDescriptor` describes direction, signal type, binding and connection policy.
  Its implementation index is unique within the input/output direction; cables
  refer to its ID. Audio channels, control domain/rate, gate and note are explicit.
  Output ports use `FanOut`; inputs use `Single` or numeric `Sum`.
- `ControlGroup` provides semantic group membership and optional ADSR bindings.
  An editor may ignore presentation hints and show ordinary individual controls.

Every descriptor has owned strings and is deserializable. Numeric values must be
finite. Integers use `i32` so values remain exact across Rust, JSON and TypeScript.
Domains are inclusive and `step` describes a suggested control increment, not an
implicit coercion. Display names/descriptions are independent of stable IDs.

`ModuleDescriptor::validate()` accumulates path-addressed errors for duplicate
IDs/indexes, unsupported bindings, invalid domains/defaults, malformed enums,
modulation declarations and envelope references. `ParameterDescriptor::validate_value`
checks a base value without coercion or clamping. Descriptors and values received
through serde still require explicit validation.

## Modulation contract

The parameter's saved value is always its **base value**. Modulation never writes
it back, so unplugging a cable restores the base. A parameter-bound control input
selects one descriptor's mapping and supported modes. A cable can override mode
and depth; `null` uses the descriptor defaults. Depth must be finite and may be
negative. Summing parameter inputs combine depth-weighted cable contributions;
all cables to that input must use the same mode. Hosts must reject unsupported
combinations.

For base `b`, input `x` and depth `d`, transform the base to coordinate
`q = T(b)`, then apply the selected mode. For a summing input, substitute
`sum(d_i*x_i)` for `d*x`:

| Mode | Modulated coordinate |
| --- | --- |
| Add | `q + d*x` |
| Multiply | `q * (1 + d*x)` |
| Replace | `d*x` |

Mappings define `T` and its inverse:

| Mapping | `T(v)` | Inverse |
| --- | --- | --- |
| Native | `v` | `q` |
| Octaves | `log2(v)` | `2^q` |
| Normalized | `(v-min)/(max-min)` | `min + q*(max-min)` |

Thus additive octave modulation with depth 1 doubles the base at input +1.
Normalized uses linear domain position even if the UI display scale is logarithmic.
After inverse mapping, clamp to the declared domain when present; integer
parameters round to the nearest whole number (halfway values away from zero).
This defines interchange semantics; the later runtime integration must implement
and test it, including nonfinite/intermediate overflow handling. Multiple distinct
ports bound to one parameter are not supported by this contract; use one summing
input or an explicit combining module.

## Instrument documents (`.trbl`)

`InstrumentDocument` is the portable JSON container. See
[`fixtures/basic.trbl`](fixtures/basic.trbl) for a complete illustrative file.
The fixture's `example.*` module IDs are not an installed engine registry.

```rust
use treble_meta::InstrumentDocument;

let document: InstrumentDocument =
    serde_json::from_str(include_str!("fixtures/basic.trbl"))?;
document.validate()?;
let json = serde_json::to_string_pretty(&document)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The envelope contains:

- `format_version`: currently `1`, independent of crate/module versions.
- `instrument`: name, description, patch, external interface and continuous,
  monophonic or bounded polyphonic voice configuration.
- `editor`: optional opaque JSON owned by the app. Rack positions, cable colours,
  slack and viewport belong here; this data round-trips without DSP validation.

A patch stores module instances, explicit user base values and individually
identified cables with stable endpoints. Missing parameter values select module
defaults. The interface exposes a note input, audio outputs and named parameter
controls. Oscillator phase, active notes, audio device settings and delay buffers
are runtime state and are not saved here.

`validate()` checks format version, instance/cable/control IDs, structural
references, finite values and voice count. It permits disconnected or incomplete
patches and unavailable plugin/module types, so work can be saved independently
of playback. Deserialization does not implicitly migrate future formats.

`validate_against(&descriptors)` additionally validates the registry, resolves
exact module versions, checks parameter ownership/types/ranges, endpoint existence,
port direction, exact signal compatibility, input cardinality, modulation settings
and external interface bindings. An explicit adapter module is required for
channel/rate/domain conversions. It does not require an output to be connected,
check feedback cycles or validate runtime capabilities. Those are engine/host
compilation concerns. Keep the last valid running graph when a new document fails.

The application owns dialogs, paths, atomic writes, dirty tracking and automatic
application policy. The runtime owns preparation, graph installation, live updates
and state transitions. Neither is implemented by metadata.

## TypeScript

Enable `ts` for `ts_rs::TS` implementations on all contracts, including the opaque
JSON editor payload. Serde uses snake_case fields and tagged enums:

```json
{"kind":"float","value":440.0}
```

Enum parameter values use the same representation with `"kind":"enum"` and a
stable string in `value`. Signal types use `{"kind":"audio","channels":2}`;
modulation modes/mappings and simple policy enums serialize as snake_case strings.

## Migration and release order

`FilterInfo`, `FilterInput`, `Parameter`, `Literal`, `ListSize`, `MixMode`,
`MetaFilter`, `MetaGenerator` and `MetaSink` retain their 0.2 API, serialization and
token generation. There is no lossy automatic legacy conversion: authors must
choose stable IDs, port semantics and typed enum variants explicitly.

Publish `treble-meta` 0.3.0 first. Update and publish `treble-derive` next, then
update `treble-core` to consume generated descriptors and implement the advertised
runtime semantics. A schema declaration alone does not enable new DSP behaviour.
