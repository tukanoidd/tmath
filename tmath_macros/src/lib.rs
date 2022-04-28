use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, FieldsUnnamed,
    Lit, Type, TypeArray,
};

// ----- Helpers START -----
// ----- Macros START -----
macro_rules! str_ident {
    ($str:expr) => {
        Ident::new($str, Span::call_site())
    };
}
// ----- Macros END -----
// ----- Helpers END -----

// ----- Derives START -----
// ----- Vector START -----
// ----- Consts START -----
const VEC_VAR_COUNT: usize = 4;
const VEC_VAR_NAMES: [&str; VEC_VAR_COUNT] = ["x", "y", "z", "w"];

const VEC_VALUE_OPS_NAMES: [&str; 5] = ["Add", "Sub", "Mul", "Div", "Rem"];
const VEC_VECTOR_OPS_NAMES: [&str; 2] = ["Add", "Sub"];
const VEC_DOT_OP_NAME: &str = "BitOr";
const VEC_CROSS_OP_NAME: &str = "BitXor";
// ----- Consts END -----

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
    match &arr_field.ty {
        Type::Array(TypeArray {
            elem: var_ty, len, ..
        }) => {
            let struct_name_str = struct_name.to_string();

            let var_ty_str = format!("{}", var_ty.to_token_stream());
            let is_int = var_ty_str.starts_with('i') || var_ty_str.starts_with('u');
            let is_signed = !var_ty_str.starts_with('u');

            let len = match len {
                Expr::Lit(ExprLit { lit, .. }) => match lit {
                    Lit::Int(lit_int) => lit_int
                        .base10_parse::<usize>()
                        .expect("Can't parse len as usize"),
                    _ => panic!("Len has to be an int literal"),
                },
                _ => panic!("Len has to be a literal"),
            };

            let var_names_str = &VEC_VAR_NAMES[..len.min(VEC_VAR_COUNT)];
            let var_names_ident = var_names_str
                .iter()
                .map(|var_name_str| str_ident!(var_name_str))
                .collect::<Vec<_>>();
            let var_names_ident_ty = var_names_ident.iter().map(|var_name_ident| {
                quote! {
                    #var_name_ident: #var_ty
                }
            });

            let bytes = &var_ty_str[1..];
            let as_float = Type::Verbatim(
                proc_macro2::TokenStream::from_str(&format!("f{}", bytes))
                    .expect("Couldn't create a token stream for the cast type"),
            );
            let var_cast = if is_int {
                quote! { as #as_float }
            } else {
                quote! {}
            };

            // ----- New START -----
            let new = quote! {
                impl #struct_name {
                    #[inline]
                    pub fn new(#(#var_names_ident_ty),*) -> Self {
                        Self([#(#var_names_ident),*])
                    }
                }
            };
            // ----- New END -----

            // ----- Consts START -----
            let consts = quote! {
                impl #struct_name {
                    pub const LEN: usize = #len;
                }
            };
            // ----- Consts END -----

            // ----- Getters Setters START -----
            let getters_setters = {
                let funcs = var_names_ident
                    .iter()
                    .enumerate()
                    .map(|(index, name_ident)| {
                        let get = name_ident;
                        let get_mut = format_ident!("{}_mut", name_ident);
                        let set = format_ident!("set_{}", name_ident);

                        quote! {
                            #[inline]
                            pub fn #get(&self) -> &#var_ty {
                                &self[#index]
                            }

                            #[inline]
                            pub fn #get_mut(&mut self) -> &mut #var_ty {
                                &mut self[#index]
                            }

                            #[inline]
                            pub fn #set(&mut self, val: #var_ty) {
                                self[#index] = val;
                            }
                        }
                    });

                quote! {
                    impl #struct_name {
                        #(#funcs)*
                    }
                }
            };
            // ----- Getters Setters END -----

            // ----- Vector Specific START -----
            let vector_specific = {
                let dot = {
                    let op_trait = str_ident!(VEC_DOT_OP_NAME);
                    let op_fun = str_ident!(&VEC_DOT_OP_NAME.to_lowercase());

                    let ops = (0..len).map(|index| {
                        quote! {
                            self[#index] * rhs[#index]
                        }
                    });

                    quote! {
                        impl #struct_name {
                            #[inline]
                            pub fn dot(&self, rhs: &Self) -> #var_ty {
                                #(#ops)+*
                            }
                        }

                        impl std::ops::#op_trait for #struct_name {
                            type Output = #var_ty;

                            #[inline]
                            fn #op_fun(self, rhs: Self) -> Self::Output {
                                self.dot(&rhs)
                            }
                        }
                    }
                };

                let cross = match len {
                    len if len == 3 => {
                        let op_name_lower = VEC_CROSS_OP_NAME.to_lowercase();

                        let op_trait = str_ident!(VEC_CROSS_OP_NAME);
                        let op_fun = str_ident!(&op_name_lower);

                        let op_trait_assign = format_ident!("{}Assign", VEC_CROSS_OP_NAME);
                        let op_fun_assign = format_ident!("{}_assign", op_name_lower);

                        quote! {
                            impl #struct_name {
                                pub fn cross(&self, rhs: &Self) -> Self {
                                    Self([
                                        self[1] * rhs[2] - self[2] * rhs[1],
                                        self[2] * rhs[0] - self[0] * rhs[2],
                                        self[0] * rhs[1] - self[1] * rhs[0]
                                    ])
                                }
                            }

                            impl std::ops::#op_trait for #struct_name {
                                type Output = Self;

                                #[inline]
                                fn #op_fun(self, rhs: Self) -> Self::Output {
                                    self.cross(&rhs)
                                }
                            }

                            impl std::ops::#op_trait_assign for #struct_name {
                                #[inline]
                                fn #op_fun_assign(&mut self, rhs: Self) {
                                    *self = self.cross(&rhs);
                                }
                            }
                        }
                    }
                    _ => quote! {},
                };

                let return_ty = if is_int { &as_float } else { var_ty };

                let magnitude = {
                    let sq = (0..len).map(|index| {
                        quote! {
                            self[#index] * self[#index]
                        }
                    });
                    let magnitude_sq = quote! {
                        #[inline]
                        pub fn magnitude_sq(&self) -> #var_ty {
                            #(#sq)+*
                        }
                    };

                    let magnitude = quote! {
                        #[inline]
                        fn magnitude(&self) -> #return_ty {
                            (self.magnitude_sq() #var_cast).sqrt()
                        }
                    };

                    quote! {
                        impl #struct_name {
                            #magnitude_sq
                            #magnitude
                        }
                    }
                };

                let normalize = match is_int {
                    false => {
                        let ops = (0..len).map(|index| {
                            quote! {
                                self[#index] / mag
                            }
                        });

                        quote! {
                            impl #struct_name {
                                pub fn normalize(&mut self) {
                                    let mag = self.magnitude();

                                    if mag > 0.0 {
                                        *self /= mag;
                                    }
                                }

                                pub fn normalized(self) -> Self {
                                    let mag = self.magnitude();

                                    if mag > 0.0 {
                                        Self([#(#ops),*])
                                    } else {
                                        self
                                    }
                                }
                            }
                        }
                    }
                    true => quote! {},
                };

                let distance = {
                    quote! {
                        impl #struct_name {
                            #[inline]
                            pub fn distance(&self, rhs: &Self) -> #return_ty {
                                (self - rhs).magnitude()
                            }
                        }
                    }
                };

                let angle = if is_int {
                    quote! {}
                } else {
                    quote! {
                        impl #struct_name {
                            pub fn angle(&self, rhs: &Self) -> #return_ty {
                                let mag_s = self.magnitude();
                                let mag_r = rhs.magnitude();

                                2.0 * (self * mag_r - rhs * mag_s)
                                    .magnitude()
                                    .atan2(
                                        (self * mag_r + rhs * mag_s)
                                        .magnitude()
                                    )
                            }
                        }
                    }
                };

                quote! {
                    #dot
                    #cross
                    #magnitude
                    #normalize
                    #distance
                    #angle
                }
            };
            // ----- Vector Specific END -----

            // ----- Indexing START -----
            let indexing = {
                quote! {
                    impl std::ops::Index<usize> for #struct_name {
                        type Output = #var_ty;

                        #[inline]
                        fn index(&self, index: usize) -> &#var_ty {
                            &self.0[index]
                        }
                    }

                    impl std::ops::IndexMut<usize> for #struct_name {
                        #[inline]
                        fn index_mut(&mut self, index: usize) -> &mut #var_ty {
                            &mut self.0[index]
                        }
                    }
                }
            };
            // ----- Indexing END -----

            // ----- Ops START -----
            let ops = {
                let value_impls = VEC_VALUE_OPS_NAMES.map(|op_name| {
                    let op_name_lower = op_name.to_lowercase();

                    let op_trait = str_ident!(op_name);
                    let op_fun = str_ident!(&op_name_lower);

                    let op_trait_assign = format_ident!("{}Assign", op_name);
                    let op_fun_assign = format_ident!("{}_assign", op_name_lower);

                    let no_ref = {
                        let ops = (0..len).map(|index| {
                            quote! {
                                self[#index].#op_fun(rhs)
                            }
                        });
                        let ops_assign = (0..len).map(|index| {
                            quote! {
                                self[#index].#op_fun_assign(rhs);
                            }
                        });

                        quote! {
                            impl std::ops::#op_trait<#var_ty> for #struct_name {
                                type Output = Self;

                                #[inline]
                                fn #op_fun(self, rhs: #var_ty) -> Self::Output {
                                    Self([#(#ops),*])
                                }
                            }

                            impl std::ops::#op_trait_assign<#var_ty> for #struct_name {
                                #[inline]
                                fn #op_fun_assign(&mut self, rhs: #var_ty) {
                                    #(#ops_assign)*
                                }
                            }
                        }
                    };

                    let with_ref = {
                        let ops = (0..len).map(|index| {
                            quote! {
                                self[#index].#op_fun(rhs)
                            }
                        });

                        quote! {
                            impl<'a> std::ops::#op_trait<#var_ty> for &'a #struct_name {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: #var_ty) -> Self::Output {
                                    #struct_name([#(#ops),*])
                                }
                            }
                        }
                    };

                    quote! {
                        #no_ref
                        #with_ref
                    }
                });

                let vector_impls = VEC_VECTOR_OPS_NAMES.map(|op_name| {
                    let op_name_lower = op_name.to_lowercase();

                    let op_trait = str_ident!(op_name);
                    let op_fun = str_ident!(&op_name_lower);

                    let op_trait_assign = format_ident!("{}Assign", op_name);
                    let op_fun_assign = format_ident!("{}_assign", op_name_lower);

                    let no_ref = {
                        let ops = (0..len).map(|index| {
                            quote! {
                            self[#index].#op_fun(rhs[#index])
                        }
                        });
                        let ops_assign = (0..len).map(|index| {
                            quote! {
                            self[#index].#op_fun_assign(rhs[#index]);
                        }
                        });

                        quote! {
                            impl std::ops::#op_trait for #struct_name {
                                type Output = Self;

                                #[inline]
                                fn #op_fun(self, rhs: Self) -> Self::Output {
                                    Self([#(#ops),*])
                                }
                            }

                            impl std::ops::#op_trait_assign for #struct_name {
                                #[inline]
                                fn #op_fun_assign(&mut self, rhs: Self) {
                                    #(#ops_assign)*
                                }
                            }
                        }
                    };

                    let with_ref = {
                        let ops = (0..len).map(|index| {
                            quote! {
                                self[#index].#op_fun(rhs[#index])
                            }
                        });
                        let ops_assign = (0..len).map(|index| {
                            quote! {
                                self[#index].#op_fun_assign(rhs[#index]);
                            }
                        });

                        quote! {
                            impl<'a, 'b> std::ops::#op_trait<&'b #struct_name> for &'a #struct_name {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: &'b #struct_name) -> Self::Output {
                                    #struct_name([#(#ops),*])
                                }
                            }

                            impl<'a> std::ops::#op_trait<&'a #struct_name> for &'a #struct_name {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: &'a #struct_name) -> Self::Output {
                                    #struct_name([#(#ops),*])
                                }
                            }

                            impl<'b> std::ops::#op_trait_assign<&'b #struct_name> for #struct_name {
                                #[inline]
                                fn #op_fun_assign(&mut self, rhs: &'b #struct_name) {
                                    #(#ops_assign)*
                                }
                            }
                        }
                    };

                    quote! {
                        #no_ref
                        #with_ref
                    }
                });

                let neg = match is_signed {
                    true => {
                        let negs = (0..len).map(|index| quote! { -self[#index] });

                        quote! {
                            impl std::ops::Neg for #struct_name {
                                type Output = Self;

                                fn neg(self) -> Self {
                                    Self([#(#negs),*])
                                }
                            }
                        }
                    }
                    false => quote! {},
                };

                quote! {
                    #(#value_impls)*
                    #(#vector_impls)*
                    #neg
                }
            };
            // ----- Ops END -----

            // ----- From START -----
            let from = {
                let repeat_var_types = vec![var_ty; len];

                quote! {
                    impl From<[#var_ty; #len]> for #struct_name {
                        #[inline]
                        fn from(rhs: [#var_ty; #len]) -> Self {
                            Self(rhs)
                        }
                    }

                    impl From<(#(#repeat_var_types),*)> for #struct_name {
                        fn from((#(#var_names_ident),*): (#(#repeat_var_types),*)) -> Self {
                            Self([#(#var_names_ident),*])
                        }
                    }
                }
            };
            // ----- From END -----

            // ----- Display/Debug START -----
            let display_debug = {
                let debug = {
                    let var_val = var_names_str
                        .iter()
                        .map(|var_name| Literal::string(var_name))
                        .enumerate()
                        .map(|(index, var_name_lit)| {
                            quote! {
                                .field(#var_name_lit, &self[#index])
                            }
                        });

                    quote! {
                        impl std::fmt::Debug for #struct_name {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                f.debug_struct(#struct_name_str)
                                #(
                                    #var_val
                                )*.finish()
                            }
                        }
                    }
                };

                let display = {
                    let curly_brackets_repetition = {
                        let str = format!(
                            "({})",
                            (0..len).map(|_| "{}").collect::<Vec<_>>().join(", ")
                        );
                        Literal::string(&str)
                    };

                    let vals = (0..len).map(|index| quote! { self[#index] });

                    quote! {
                        impl std::fmt::Display for #struct_name {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                write!(f, #curly_brackets_repetition, #(#vals),*)
                            }
                        }
                    }
                };

                quote! {
                    #debug
                    #display
                }
            };
            // ----- Display/Debug END -----

            (quote! {
                #new
                #consts
                #getters_setters
                #vector_specific
                #indexing
                #ops
                #from
                #display_debug
            })
            .into()
        }
        _ => panic!("First struct member has to be an array"),
    }
}
// ----- Vector END -----
// ----- Derives END -----
