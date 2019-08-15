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
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_encodable_macro(&ast)
}

#[proc_macro_derive(AbiDecodable)]
pub fn decodable_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_decodable_macro(&ast)
}

fn impl_encodable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if let syn::Data::Struct(data_struct) = &ast.data {
        let field_list = data_struct
            .fields
            .iter()
            .filter(|f| f.attrs.first().is_none())
            .map(|f| {
                let ident = &f.ident.clone().unwrap();
                if let syn::Type::Path(path) = &f.ty {
                    let type_name = &path.path.segments.first().unwrap().value().ident;
                    if type_name == "Bytes" {
                        return quote! {
                            Token::Bytes(self.#ident.to_vec())
                        };
                    } else if type_name == "Integer" {
                        return quote! {
                            Token::Uint(self.#ident.0.into())
                        };
                    } else if type_name == "Address" {
                        return quote! {
                            Token::Address(self.#ident)
                        };
                    } else if type_name == "H256" {
                        return quote! {
                            Token::FixedBytes(self.#ident.as_bytes().to_vec())
                        };
                    } else if type_name == "Property" {
                        return quote! {
                            Token::Bytes(self.#ident.to_abi())
                        };
                    } else {
                        return quote! {
                            Token::Tuple(self.#ident.to_tuple())
                        };
                    }
                }
                quote! {
                    Token::Tuple(self.#ident.to_tuple())
                }
            });
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

fn impl_decodable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if let syn::Data::Struct(data_struct) = &ast.data {
        let param_type_list = data_struct
            .fields
            .iter()
            .filter(|f| f.attrs.first().is_none())
            .map(|f| {
                if let syn::Type::Path(path) = &f.ty {
                    let type_name = &path.path.segments.first().unwrap().value().ident;
                    if type_name == "Bytes" || type_name == "Property" {
                        return quote! {
                            ethabi::ParamType::Bytes
                        };
                    } else if type_name == "Integer" {
                        return quote! {
                            ethabi::ParamType::Uint(256)
                        };
                    } else if type_name == "H256" {
                        return quote! {
                            ethabi::ParamType::FixedBytes(256)
                        };
                    } else if type_name == "Address" {
                        return quote! {
                            ethabi::ParamType::Address
                        };
                    } else {
                        return quote! {
                            ethabi::ParamType::Tuple(#type_name::get_param_types())
                        };
                    }
                }
                quote! {
                    ethabi::ParamType::Address
                }
            });
        let mut count: usize = 0;
        let field_list: &Vec<_> = &data_struct.fields.iter()
            .filter(|f| f.attrs.first().is_none())
            .map(move |f| {
            let ident = &f.ident.clone().unwrap();
            if let syn::Type::Path(path) = &f.ty {
                let type_name = &path.path.segments.first().unwrap().value().ident;
                count += 1;
                let index = count - 1;
                if type_name == "Bytes" {
                    return quote! {
                        let #ident = Bytes::from(tuple[#index].clone().to_bytes().unwrap());
                    };
                } else if type_name == "Integer" {
                    return quote! {
                        let #ident: Integer = Integer(tuple[#index].clone().to_uint().unwrap().as_u64());
                    };
                } else if type_name == "H256" {
                    return quote! {
                        let #ident: H256 = H256::from_slice(&tuple[#index].clone().to_bytes().unwrap().to_vec());
                    };
                } else if type_name == "Address" {
                    return quote! {
                        let #ident: Address = tuple[#index].clone().to_address().unwrap();
                    };
                } else if type_name == "Property" {
                    return quote! {
                        let #ident: #type_name = #type_name::from_abi(&tuple[#index].clone().to_bytes().unwrap()).unwrap();
                    };
                } else {
                    return quote! {
                        let #ident: #type_name = #type_name::from_tuple(&tuple[#index].clone().to_tuple().unwrap()).unwrap();
                    };
                }
            } else {
                panic!("aaa")
            }
        }).collect();
        let create_fields = data_struct.fields.iter().map(|f| {
            let is_none = f.attrs.first().is_none();
            if is_none {
                let ident = &f.ident.clone().unwrap();
                return quote! {
                    #ident
                };
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
    } else {
        panic!("invalid data")
    }
}
