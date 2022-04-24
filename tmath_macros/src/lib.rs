use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, FieldsUnnamed, Type, TypeArray,
};

// ----- Derives START -----
// ----- Vector START -----
#[proc_macro_derive(Vector)]
pub fn derive_vector(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: struct_name,
        data,
        ..
    } = parse_macro_input!(input);

    match data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let arr_field = unnamed.first().unwrap();
                parse_array_vector(&struct_name, arr_field)
            }
            Fields::Named(_) => panic!("Named structs are not yet supported"),
            Fields::Unit => {
                panic!("Unit structs are not permitted, it has to contain data to work with")
            }
        },
        _ => panic!("Only structs are permitted"),
    }
}

fn parse_array_vector(struct_name: &Ident, arr_field: &Field) -> TokenStream {
    match arr_field.ty {
        Type::Array(TypeArray {
            elem: var_ty, len, ..
        }) => {
            // TODO(tukanoidd): implement
            (quote! {}).into()
        }
        _ => panic!("First struct member has to be an array"),
    }
}
// ----- Vector END -----
// ----- Derives END -----
