use quote::ToTokens;
use quote::quote;

// #[derive(Debug, Clone)]
// pub enum Literal {
//     Toggle(String, bool),
//     Range(String, f32, f32, f32),
//     Float(String, f32),
// }

#[derive(Clone, Debug)]
pub enum Parameter {
    Toggle(String, bool),
    Range(String, f32, f32, f32),
    Float(String, f32),
    // List(String, usize, Literal),
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Parameter::Toggle(name, value) => {
                tokens.extend(quote! {
                    rustic_meta::Parameter::Toggle(String::from(#name), *#value)
                });
            }
            Parameter::Range(name, min, max, default) => {
                tokens.extend(quote! {
                    rustic_meta::Parameter::Range(String::from(#name), #min, #max, #default)
                });
            }
            Parameter::Float(name, value) => {
                tokens.extend(quote! {
                    rustic_meta::Parameter::Float(String::from(#name), #value)
                });
            }
        }
    }
}
