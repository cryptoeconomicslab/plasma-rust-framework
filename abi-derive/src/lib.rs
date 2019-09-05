extern crate proc_macro;

use crate::proc_macro::TokenStream;
#[allow(unused_imports)]
use bytes::Bytes;
#[allow(unused_imports)]
use ethabi::Token;
use quote::quote;
use syn;

#[proc_macro_derive(AbiEncodable)]
pub fn encodable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_encodable_macro(&ast)
}

#[proc_macro_derive(AbiDecodable)]
pub fn decodable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_decodable_macro(&ast)
}

/// Builds encodable trait implementation
fn impl_encodable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if let syn::Data::Struct(data_struct) = &ast.data {
        let field_list = data_struct
            .fields
            .iter()
            .filter(|f| f.attrs.first().is_none())
            .map(create_tokens);
        let gen = quote! {
            impl Encodable for #name {
                fn to_tuple(&self) -> Vec<Token> {
                    vec![#(#field_list),*]
                }
            }
        };
        gen.into()
    } else {
        panic!("invalid data")
    }
}

/// Builds decodable trait implementation
fn impl_decodable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if let syn::Data::Struct(data_struct) = &ast.data {
        impl_decodable_macro_for_struct(name, data_struct)
    } else {
        panic!("haven't supported except struct")
    }
}

fn impl_decodable_macro_for_struct(
    name: &syn::Ident,
    data_struct: &syn::DataStruct,
) -> TokenStream {
    let fields = &data_struct.fields;
    let param_type_list = fields
        .iter()
        .filter(|f| f.attrs.first().is_none())
        .map(create_param_type_token_stream);
    let mut count = -1;
    let field_list: &Vec<_> = &fields
        .iter()
        .filter(|f| f.attrs.first().is_none())
        .map(move |f| {
            let ident = &f.ident.clone().unwrap();
            if let syn::Type::Path(path) = &f.ty {
                let type_name = &path.path.segments.first().unwrap().ident;
                count += 1;
                create_parse_val_token_stream(ident, type_name, count as usize)
            } else {
                panic!("invalid type")
            }
        })
        .collect();
    let create_fields = fields.iter().map(|f| {
        let is_none = f.attrs.first().is_none();
        if is_none {
            let ident = &f.ident.clone().unwrap();
            return quote! { #ident };
        } else {
            return quote! {
                None
            };
        }
    });
    let gen = quote! {
        impl Decodable for #name {
            type Ok = #name;
            fn from_tuple(tuple: &[Token]) -> Result<Self, plasma_core::data_structure::error::Error> {
                #(#field_list)*
                Ok(#name::new(
                    #(#create_fields),*
                ))
            }
            fn get_param_types() -> Vec<ParamType> {
                vec![#(#param_type_list),*]
            }
        }
    };
    gen.into()
}

fn create_tokens(f: &syn::Field) -> proc_macro2::TokenStream {
    let ident = &f.ident.clone().unwrap();
    if let syn::Type::Path(path) = &f.ty {
        let type_name = &path.path.segments.first().unwrap().ident;
        match &*type_name.to_string() {
            "bool" => quote! {
                Token::Bool(self.#ident)
            },
            "Bytes" => quote! {
                Token::Bytes(self.#ident.to_vec())
            },
            "Integer" => quote! {
                Token::Uint(self.#ident.0.into())
            },
            "H256" => quote! {
                Token::FixedBytes(self.#ident.as_bytes().to_vec())
            },
            "Address" => quote! {
                Token::Address(self.#ident)
            },
            _ => quote! {
                Token::Tuple(self.#ident.to_tuple())
            },
        }
    } else {
        quote! {
            Token::Tuple(self.#ident.to_tuple())
        }
    }
}

fn create_param_type_token_stream(f: &syn::Field) -> proc_macro2::TokenStream {
    if let syn::Type::Path(path) = &f.ty {
        let type_name = &path.path.segments.first().unwrap().ident;
        match &*type_name.to_string() {
            "bool" => quote! {
                ethabi::ParamType::Bool
            },
            "Bytes" => quote! {
                ethabi::ParamType::Bytes
            },
            "Integer" => quote! {
                ethabi::ParamType::Uint(256)
            },
            "H256" => quote! {
                ethabi::ParamType::FixedBytes(32)
            },
            "Address" => quote! {
                ethabi::ParamType::Address
            },
            _ => quote! {
                ethabi::ParamType::Tuple(#type_name::get_param_types())
            },
        }
    } else {
        quote! {
            ethabi::ParamType::Address
        }
    }
}

fn create_parse_val_token_stream(
    ident: &syn::Ident,
    type_name: &syn::Ident,
    index: usize,
) -> proc_macro2::TokenStream {
    match &*type_name.to_string() {
        "bool" => quote! {
            let #ident: bool = tuple[#index].clone().to_bool().unwrap();
        },
        "Bytes" => quote! {
            let #ident = Bytes::from(tuple[#index].clone().to_bytes().unwrap());
        },
        "Integer" => quote! {
            let #ident: Integer = Integer(tuple[#index].clone().to_uint().unwrap().as_u64());
        },
        "H256" => quote! {
            let #ident: H256 = H256::from_slice(&tuple[#index].clone().to_fixed_bytes().unwrap().to_vec());
        },
        "Address" => quote! {
            let #ident: Address = tuple[#index].clone().to_address().unwrap();
        },
        _ => quote! {
            let #ident: #type_name = #type_name::from_tuple(&tuple[#index].clone().to_tuple().unwrap()).unwrap();
        },
    }
}
