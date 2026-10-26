# Changelog

## 0.3.0

- Add owned unified module descriptors with stable IDs, typed parameters and enums,
  numeric domains/units/scales, engine ownership, update/smoothing declarations,
  typed ports, explicit connection policies and structured envelope groups.
- Define base-preserving modulation mappings, supported modes and cable overrides.
- Add version 1 `.trbl` JSON instrument contracts with module instances, cables,
  external controls, note/audio interface, voice configuration and opaque optional
  editor layout. Runtime state is excluded.
- Add accumulated, path-addressed descriptor/value/document validation with an
  optional registry resolution pass. Incomplete patches remain saveable.
- Support all new contracts through the optional `ts` feature.
- Preserve legacy 0.2 filter metadata and token generation for staged migration.

No runtime, derive, automatic graph compilation or audio behaviour changes are
included. Module and document versions are independent of crate semver.
