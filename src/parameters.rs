use quote::ToTokens;
use quote::quote;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a literal value that defines a List parameter's type.
pub enum Literal {
    Toggle(String, bool),
    Range(String, f32, f32, f32),
    Float(String, f32),
    Int(String, i32, Option<i32>, Option<i32>),
}

impl ToTokens for Literal {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Literal::Toggle(title, default) => {
                tokens.extend(quote! {
                    rustic_meta::Literal::Toggle(String::from(#title), #default)
                });
            }
            Literal::Range(title, min, max, default) => {
                tokens.extend(quote! {
                    rustic_meta::Literal::Range(String::from(#title), #min, #max, #default)
                });
            }
            Literal::Float(title, default) => {
                tokens.extend(quote! {
                    rustic_meta::Literal::Float(String::from(#title), #default)
                });
            }
            Literal::Int(title, default, min, max) => {
                let min_tokens = match min {
                    Some(v) => quote! { Some(#v) },
                    None => quote! { None },
                };
                let max_tokens = match max {
                    Some(v) => quote! { Some(#v) },
                    None => quote! { None },
                };
                tokens.extend(quote! {
                    rustic_meta::Literal::Int(String::from(#title), #default, #min_tokens, #max_tokens)
                });
            }
        }
    }
}

/// Determines the size of a List parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ListSize<S> {
    /// Size is determined by the value of another field on the struct.
    Field(S),
    /// Size is a fixed constant.
    Constant(usize),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Parameter<S> {
    Toggle {
        title: S,
        field_name: S,
        default: bool,
        value: bool,
    },
    Range {
        title: S,
        field_name: S,
        min: f32,
        max: f32,
        default: f32,
        value: f32,
    },
    Float {
        title: S,
        field_name: S,
        default: f32,
        value: f32,
    },
    Int {
        title: S,
        field_name: S,
        default: i32,
        value: i32,
        min: Option<i32>,
        max: Option<i32>,
    },
    List {
        title: S,
        field_name: S,
        size: ListSize<S>,
        ltype: Literal,
    },
}

impl<T: quote::ToTokens> ToTokens for ListSize<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ListSize::Field(name) => {
                tokens.extend(quote! {
                    rustic_meta::ListSize::Field(#name)
                });
            }
            ListSize::Constant(n) => {
                tokens.extend(quote! {
                    rustic_meta::ListSize::Constant(#n)
                });
            }
        }
    }
}

impl<T: quote::ToTokens> ToTokens for Parameter<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Parameter::Toggle {
                title,
                field_name,
                default,
                value,
            } => {
                tokens.extend(quote! {
                    rustic_meta::Parameter::Toggle{
                        title: #title,
                        field_name: #field_name,
                        default: #default,
                        value: #value
                    }
                });
            }
            Parameter::Range {
                title,
                field_name,
                min,
                max,
                default,
                value,
            } => {
                tokens.extend(quote! {
                    rustic_meta::Parameter::Range {
                        title: #title,
                        field_name: #field_name,
                        min: #min,
                        max: #max,
                        default: #default,
                        value: #value
                    }
                });
            }
            Parameter::Float {
                title,
                field_name,
                default,
                value,
            } => {
                tokens.extend(quote! {
                    rustic_meta::Parameter::Float {
                        title: #title,
                        field_name: #field_name,
                        default: #default,
                        value: #value
                    }
                });
            }
            Parameter::Int {
                title,
                field_name,
                default,
                value,
                min,
                max,
            } => {
                let min_tokens = match min {
                    Some(v) => quote! { Some(#v) },
                    None => quote! { None },
                };
                let max_tokens = match max {
                    Some(v) => quote! { Some(#v) },
                    None => quote! { None },
                };
                tokens.extend(quote! {
                    rustic_meta::Parameter::Int {
                        title: #title,
                        field_name: #field_name,
                        default: #default,
                        value: #value,
                        min: #min_tokens,
                        max: #max_tokens
                    }
                });
            }
            Parameter::List {
                title,
                field_name,
                size,
                ltype,
            } => {
                tokens.extend(quote! {
                    rustic_meta::Parameter::List {
                        title: #title,
                        field_name: #field_name,
                        size: #size,
                        ltype: #ltype
                    }
                });
            }
        }
    }
}
