mod parameters;
pub use parameters::{ListSize, Literal, Parameter};

/// Trait for filters that support named parameter modification.
/// Implemented automatically by the `FilterMetaData` derive macro.
pub trait MetaFilter {
    /// Sets a parameter by name. The default implementation is a no-op.
    fn set_parameter(&mut self, _name: &str, _value: f32) {}
}
