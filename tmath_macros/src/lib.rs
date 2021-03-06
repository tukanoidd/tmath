use std::str::FromStr;

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream, Peek},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Comma, Paren},
    Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, FieldsUnnamed, Lit, Path, Type,
    TypeArray,
};

// ----- Helpers START -----
// ----- Macros START -----
macro_rules! str_ident {
    ($str:expr) => {
        Ident::new($str, Span::call_site())
    };
}
// ----- Macros END -----

// ----- Functions START -----
fn parse_until<E: Peek>(input: ParseStream, end: E) -> syn::Result<TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();

    while !input.is_empty() && !input.peek(end) {
        let next: proc_macro2::TokenTree = input.parse()?;
        tokens.extend(Some(next));
    }

    Ok(tokens.into())
}

fn parse_punctuated_comma<S: Clone + Parse>(input: ParseStream) -> syn::Result<Vec<S>> {
    let mut res = Punctuated::<S, Comma>::new();
    let first = parse_until(input, Comma)?;

    res.push_value(syn::parse(first)?);

    while input.peek(Comma) {
        res.push_punct(input.parse()?);

        let next = parse_until(input, Comma)?;
        res.push_value(syn::parse(next)?);
    }

    Ok(res.iter().cloned().collect())
}
// ----- Functions END -----
// ----- Helpers END -----

// ----- Vector START -----
// ----- Consts START -----
const VEC_VAR_COUNT: usize = 4;
const VEC_VAR_NAMES: [&str; VEC_VAR_COUNT] = ["x", "y", "z", "w"];

const VEC_VALUE_OPS_NAMES: [&str; 5] = ["Add", "Sub", "Mul", "Div", "Rem"];
const VEC_VECTOR_OPS_NAMES: [&str; 5] = ["Add", "Sub", "Mul", "Div", "Rem"];
const VEC_DOT_OP_NAME: &str = "BitOr";
const VEC_CROSS_OP_NAME: &str = "BitXor";
const VEC_TYPES: [&str; 2] = ["Point", "Color"];
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
            let var_names_ident_ty_const = var_names_ident_ty.clone();

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
            let new = {
                let vals = (0..len).map(|_| quote! { val });
                let vals_const = vals.clone();
                let rands = (0..len).map(|_| quote! { rand::Rng::gen(&mut rng) });
                let rands_range =
                    (0..len).map(|_| quote! { rand::Rng::gen_range(&mut rng, min..max) });
                let rands_vec_range =
                    (0..len).map(|i| quote! { rand::Rng::gen_range(&mut rng, min[#i]..max[#i]) });
                let zeros = if len > 2 {
                    (0..len - 2)
                        .map(|index| {
                            let comma = if index == 0 {
                                quote! { , }
                            } else {
                                quote! {}
                            };

                            quote! { #comma 0.0 }
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![quote! {}]
                };

                let randoms_non_int = if !is_int {
                    quote! {
                        #[inline]
                        pub fn random_unit() -> Self {
                            Self::random_range(-1 as #var_ty, 1 as #var_ty).normalized()
                        }

                        #[inline]
                        pub fn random_in_hemisphere(normal: &Self) -> Self {
                            let in_unit_sphere = Self::random_unit();

                            if (in_unit_sphere | normal) > 0.0 {
                                in_unit_sphere
                            } else {
                                -in_unit_sphere
                            }
                        }

                        pub fn random_in_unit_disk() -> Self {
                            let mut rng = rand::thread_rng();
                            let p = Self::new(
                                rand::Rng::gen_range(&mut rng, -1.0..1.0),
                                rand::Rng::gen_range(&mut rng, -1.0..1.0)
                                #(#zeros),*
                            );

                            p.normalized()
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    impl #struct_name {
                        #[inline]
                        pub fn new(#(#var_names_ident_ty),*) -> Self {
                            Self([#(#var_names_ident),*])
                        }

                        #[inline]
                        pub const fn new_const(#(#var_names_ident_ty_const),*) -> Self {
                            Self([#(#var_names_ident),*])
                        }

                        #[inline]
                        pub fn new_val(val: #var_ty) -> Self {
                            Self([#(#vals),*])
                        }

                        #[inline]
                        pub const fn new_val_const(val: #var_ty) -> Self {
                            Self([#(#vals_const),*])
                        }

                        pub fn random() -> Self {
                            let mut rng = rand::thread_rng();
                            Self([#(#rands),*])
                        }

                        pub fn random_range(min: #var_ty, max: #var_ty) -> Self {
                            let mut rng = rand::thread_rng();
                            Self([#(#rands_range),*])
                        }

                        pub fn random_vec_range(min: &Self, max: &Self) -> Self {
                            let mut rng = rand::thread_rng();
                            Self([#(#rands_vec_range),*])
                        }

                        #randoms_non_int

                        #[inline]
                        pub fn clear(&mut self) {
                            *self = Self::default();
                        }
                    }
                }
            };
            // ----- New END -----

            // ----- Consts START -----
            let consts = quote! {
                impl #struct_name {
                    pub const LEN: usize = #len;
                    pub const ZERO: Self = Self([0 as #var_ty; #len]);
                    pub const ONE: Self = Self([1 as #var_ty; #len]);
                }
            };
            // ----- Consts END -----

            // ----- Getters Setters START -----
            let getters_setters = if (2..=4).contains(&len) {
                let funcs =
                    var_names_ident
                        .iter()
                        .enumerate()
                        .clone()
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
            } else {
                quote! {}
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

                        impl<'a> std::ops::#op_trait<#struct_name> for &'a #struct_name {
                            type Output = #var_ty;

                            #[inline]
                            fn #op_fun(self, rhs: #struct_name) -> Self::Output {
                                self.dot(&rhs)
                            }
                        }

                        impl<'b> std::ops::#op_trait<&'b #struct_name> for #struct_name {
                            type Output = #var_ty;

                            #[inline]
                            fn #op_fun(self, rhs: &'b #struct_name) -> Self::Output {
                                self.dot(rhs)
                            }
                        }

                        impl<'a, 'b> std::ops::#op_trait<&'b #struct_name> for &'a #struct_name {
                            type Output = #var_ty;

                            #[inline]
                            fn #op_fun(self, rhs: &'b #struct_name) -> Self::Output {
                                self.dot(rhs)
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
                        pub fn magnitude(&self) -> #return_ty {
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

                let near_zero = if !is_int {
                    let less_s = (0..len).map(|index| {
                        quote! {
                            self[#index].abs() < 1e-8 as #var_ty
                        }
                    });

                    quote! {
                        impl #struct_name {
                            #[inline]
                            pub fn near_zero(&self) -> bool {
                                #(#less_s)&&*
                            }
                        }
                    }
                } else {
                    quote! {}
                };

                let reflect = quote! {
                    impl #struct_name {
                        pub fn reflect(&self, rhs: &Self) -> Self {
                            self - 2 as #var_ty * (self | rhs) * rhs
                        }
                    }
                };

                let refract = if !is_int {
                    quote! {
                        impl #struct_name {
                            pub fn refract(&self, rhs: &Self, etai_over_etat: #var_ty) -> Self {
                                let cos_theta = (1 as #var_ty).min(-self | rhs);
                                let r_out_perp = etai_over_etat * (self + cos_theta * rhs);
                                let r_out_parallel = -(1 as #var_ty - r_out_perp.magnitude_sq())
                                    .abs()
                                    .sqrt() * rhs;

                                r_out_perp + r_out_parallel
                            }
                        }
                    }
                } else {
                    quote! {}
                };

                let min_max = {
                    let mins = (0..len).map(|index| quote! { self[#index].min(rhs[#index]) });
                    let maxs = (0..len).map(|index| quote! { self[#index].max(rhs[#index]) });

                    quote! {
                        impl #struct_name {
                            pub fn min(&self, rhs: &Self) -> Self {
                                Self([#(#mins),*])
                            }

                            pub fn max(&self, rhs: &Self) -> Self {
                                Self([#(#maxs),*])
                            }
                        }
                    }
                };

                let abs = if is_signed {
                    let abs_vals = (0..len).map(|index| quote! { self[#index].abs() });

                    quote! {
                        impl #struct_name {
                            pub fn abs(self) -> Self {
                                Self([#(#abs_vals),*])
                            }
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    #dot
                    #cross
                    #magnitude
                    #normalize
                    #distance
                    #angle
                    #near_zero
                    #reflect
                    #refract
                    #min_max
                    #abs
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
                        let ops_vec_val = (0..len).map(|index| {
                            quote! {
                                self[#index].#op_fun(rhs)
                            }
                        });
                        let ops_assign_vec_val = (0..len).map(|index| {
                            quote! {
                                self[#index].#op_fun_assign(rhs);
                            }
                        });

                        quote! {
                            impl std::ops::#op_trait<#var_ty> for #struct_name {
                                type Output = Self;

                                #[inline]
                                fn #op_fun(self, rhs: #var_ty) -> Self::Output {
                                    Self([#(#ops_vec_val),*])
                                }
                            }

                            impl std::ops::#op_trait_assign<#var_ty> for #struct_name {
                                #[inline]
                                fn #op_fun_assign(&mut self, rhs: #var_ty) {
                                    #(#ops_assign_vec_val)*
                                }
                            }

                            impl std::ops::#op_trait<#struct_name> for #var_ty {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: #struct_name) -> Self::Output {
                                    rhs.#op_fun(self)
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

                            impl<'a> std::ops::#op_trait<&'a #struct_name> for #var_ty {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: &'a #struct_name) -> Self::Output {
                                    rhs.#op_fun(self)
                                }
                            }

                            impl<'a, 'b> std::ops::#op_trait<&'b #var_ty> for &'a #struct_name {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: &'b #var_ty) -> Self::Output {
                                    self.#op_fun(*rhs)
                                }
                            }

                            impl<'b> std::ops::#op_trait<&'b #var_ty> for #struct_name {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: &'b #var_ty) -> Self::Output {
                                    self.#op_fun(*rhs)
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

                            impl<'a> std::ops::#op_trait<#struct_name> for &'a #struct_name {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: #struct_name) -> Self::Output {
                                    self.#op_fun(&rhs)
                                }
                            }

                            impl<'b> std::ops::#op_trait<&'b #struct_name> for #struct_name {
                                type Output = #struct_name;

                                #[inline]
                                fn #op_fun(self, rhs: &'b #struct_name) -> Self::Output {
                                    (&self).#op_fun(rhs)
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
                        let negs_no_ref = (0..len).map(|index| quote! { -self[#index] });
                        let negs_with_ref = negs_no_ref.clone();

                        quote! {
                            impl std::ops::Neg for #struct_name {
                                type Output = Self;

                                fn neg(self) -> Self::Output {
                                    Self([#(#negs_no_ref),*])
                                }
                            }

                            impl<'a> std::ops::Neg for &'a #struct_name {
                                type Output = #struct_name;

                                fn neg(self) -> Self::Output {
                                    #struct_name([#(#negs_with_ref),*])
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
                let tuple_vals = (0..len).map(|index| quote! { rhs[#index] });
                let tuple_vals_ref = tuple_vals.clone();

                quote! {
                    impl From<#var_ty> for #struct_name {
                        #[inline]
                        fn from(rhs: #var_ty) -> Self {
                            Self::new_val(rhs)
                        }
                    }

                    impl From<[#var_ty; #len]> for #struct_name {
                        #[inline]
                        fn from(rhs: [#var_ty; #len]) -> Self {
                            Self(rhs)
                        }
                    }

                    impl From<#struct_name> for [#var_ty; #len] {
                        #[inline]
                        fn from(rhs: #struct_name) -> Self {
                            rhs.0
                        }
                    }

                    impl<'b> From<&'b #struct_name> for [#var_ty; #len] {
                        #[inline]
                        fn from(rhs: &'b #struct_name) -> Self {
                            rhs.0
                        }
                    }

                    impl From<(#(#repeat_var_types),*)> for #struct_name {
                        #[inline]
                        fn from((#(#var_names_ident),*): (#(#repeat_var_types),*)) -> Self {
                            Self([#(#var_names_ident),*])
                        }
                    }

                    impl From<#struct_name> for (#(#repeat_var_types),*) {
                        #[inline]
                        fn from(rhs: #struct_name) -> Self {
                            (#(#tuple_vals),*)
                        }
                    }

                    impl<'b> From<&'b #struct_name> for (#(#repeat_var_types),*) {
                        #[inline]
                        fn from(rhs: &'b #struct_name) -> Self {
                            (#(#tuple_vals_ref),*)
                        }
                    }
                }
            };
            // ----- From END -----

            // ----- Hash START -----
            let hash = {
                let hashes = (0..len).map(|index| {
                    if is_int {
                        quote! { self[#index].hash(state); }
                    } else {
                        quote! { self[#index].to_bits().hash(state); }
                    }
                });

                quote! {
                    impl std::hash::Hash for #struct_name {
                        fn hash<H>(&self, state: &mut H)
                        where
                            H: std::hash::Hasher,
                        {
                            #(#hashes)*
                        }
                    }
                }
            };
            // ----- Hash END -----

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

            // ----- Types START -----
            let types = {
                let types = VEC_TYPES
                    .map(|ty| {
                        format_ident!(
                            "{}{}{}",
                            ty,
                            len,
                            match var_ty_str.as_str() {
                                "f64" => "D",
                                "i32" => "I",
                                "i64" => "L",
                                "u32" => "U",
                                "u64" => "UL",
                                _ => "",
                            }
                        )
                    })
                    .map(|type_name| {
                        quote! {
                            pub type #type_name = #struct_name;
                        }
                    });

                quote! {
                    #(#types)*
                }
            };
            // ----- Types END -----

            (quote! {
                #new
                #consts
                #getters_setters
                #vector_specific
                #indexing
                #ops
                #from
                #hash
                #display_debug
                #types
            })
            .into()
        }
        _ => panic!("First struct member has to be an array"),
    }
}

#[derive(Clone)]
struct CastVectorInfo {
    path: Path,
    len: Lit,
    elem_ty: Type,
}

impl Parse for CastVectorInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _paren = parenthesized!(content in input);

        Ok(Self {
            path: {
                let path: Path = content.parse()?;
                let _: Comma = content.parse()?;

                path
            },
            len: {
                let len: Lit = content.parse()?;
                let _: Comma = content.parse()?;

                len
            },
            elem_ty: content.parse()?,
        })
    }
}

impl ToTokens for CastVectorInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CastVectorInfo {
            path, len, elem_ty, ..
        } = self;

        *tokens = quote! {
            (#path, #len, #elem_ty)
        };
    }
}

struct CastAllVectorsInput(Vec<CastVectorInfo>);

impl Parse for CastAllVectorsInput {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(parse_punctuated_comma(input)?))
    }
}

#[proc_macro]
pub fn cast_all_vectors(input: TokenStream) -> TokenStream {
    let CastAllVectorsInput(vector_infos) = parse_macro_input!(input);

    let vec_amount = vector_infos.len();

    let all_casts = (0..(vec_amount - 1))
        .flat_map(|i| {
            let vec1 = &vector_infos[i];

            ((i + 1)..vec_amount)
                .map(|j| {
                    let vec2 = &vector_infos[j];

                    cast_two_vectors(vec1, vec2).into()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    (quote! { #( #all_casts )* }).into()
}

struct CastVectorsInput {
    info_left: CastVectorInfo,
    _comma: Comma,
    info_right: CastVectorInfo,
}

impl Parse for CastVectorsInput {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            info_left: input.parse()?,
            _comma: input.parse()?,
            info_right: input.parse()?,
        })
    }
}

impl ToTokens for CastVectorsInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CastVectorsInput {
            info_left,
            info_right,
            ..
        } = self;

        *tokens = quote! {
            #info_left, #info_right
        };
    }
}

#[proc_macro]
pub fn cast_vectors(input: TokenStream) -> TokenStream {
    let CastVectorsInput {
        info_left,
        info_right,
        ..
    } = parse_macro_input!(input);

    cast_two_vectors(&info_left, &info_right)
}

fn cast_two_vectors(left_info: &CastVectorInfo, right_info: &CastVectorInfo) -> TokenStream {
    let CastVectorInfo {
        path: vec1,
        len: vec1_len,
        elem_ty: vec1_ety,
        ..
    } = left_info;

    let CastVectorInfo {
        path: vec2,
        len: vec2_len,
        elem_ty: vec2_ety,
        ..
    } = right_info;

    match (vec1_len, vec2_len) {
        (Lit::Int(vec1_len_lit_int), Lit::Int(vec2_lit_int)) => {
            let (vec1_len, vec2_len): (usize, usize) = (
                vec1_len_lit_int
                    .base10_parse()
                    .expect("Couldn't parse len 1 to usize"),
                vec2_lit_int
                    .base10_parse()
                    .expect("Couldn't parse len 2 to usize"),
            );

            let (vec2_vec1, vec1_vec2) = {
                let min_len = vec1_len.min(vec2_len);

                let map_fn = |index, ty: &Type| {
                    if index < min_len {
                        quote! { rhs[#index] as #ty }
                    } else {
                        quote! { #ty::default() }
                    }
                };

                let vec2_vec1 = (0..vec1_len)
                    .map(|index| map_fn(index, vec1_ety))
                    .collect::<Vec<_>>();
                let vec1_vec2 = (0..vec2_len)
                    .map(|index| map_fn(index, vec2_ety))
                    .collect::<Vec<_>>();

                (vec2_vec1, vec1_vec2)
            };

            (quote! {
                impl From<#vec2> for #vec1 {
                    #[inline]
                    fn from(rhs: #vec2) -> #vec1 {
                        #vec1::new(#(#vec2_vec1),*)
                    }
                }

                impl From<#vec1> for #vec2 {
                    #[inline]
                    fn from(rhs: #vec1) -> #vec2 {
                        #vec2::new(#(#vec1_vec2),*)
                    }
                }
            })
            .into()
        }
        _ => panic!("Both lengths have to be integer literals"),
    }
}

struct MultipleVectorManipulationInput {
    base_name: Ident,
    lengths: Vec<usize>,
    types: Vec<VectorType>,
}

impl Parse for MultipleVectorManipulationInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let base_name = input.parse()?;
        let _comma: Comma = input.parse()?;

        let length_content;
        let _bracket = bracketed!(length_content in input);
        let lengths = parse_punctuated_comma::<Lit>(&length_content)?
            .iter()
            .map(|lit| match lit {
                Lit::Int(val) => val.base10_parse().unwrap(),
                _ => panic!("Vector length has to be an integer literal"),
            })
            .collect::<Vec<usize>>();

        let _comma: Comma = input.parse()?;

        let types_content;
        let _bracket = bracketed!(types_content in input);
        let types = parse_punctuated_comma(&types_content)?;

        Ok(Self {
            base_name,
            lengths,
            types,
        })
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct VectorType {
    ty: Type,
    name: Option<Ident>,
}

impl Parse for VectorType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _paren: Paren = parenthesized!(content in input);

        Ok(Self {
            ty: {
                let ty = content.parse()?;
                let _comma: Comma = content.parse()?;
                ty
            },
            name: {
                let name = content.parse();
                name.ok()
            },
        })
    }
}

#[proc_macro]
pub fn combinatory_getters_setters(input: TokenStream) -> TokenStream {
    let MultipleVectorManipulationInput {
        base_name,
        lengths,
        types,
    } = parse_macro_input!(input);

    let res = lengths.iter().filter(|&&len| len > 2).map(|&len| {
        let var_names_str = &VEC_VAR_NAMES[..len.min(VEC_VAR_COUNT)];
        let var_names_ident = var_names_str
            .iter()
            .map(|var_name_str| str_ident!(var_name_str))
            .collect::<Vec<_>>();

        let funcs_combinations = types
            .iter()
            .map(|VectorType { name, .. }| {
                let name = name.clone().map_or("".to_string(), |name| name.to_string());
                let struct_name = format_ident!("{}{}{}", base_name, len, name);

                let funcs_combinations = (2..len)
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|l| {
                        var_names_ident
                            .iter()
                            .enumerate()
                            .clone()
                            .combinations(*l)
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .map(|combination| {
                        let l = combination.len();
                        let getter_name = str_ident!(&combination
                            .iter()
                            .map(|(_, var)| var.to_string())
                            .collect::<Vec<_>>()
                            .join(""));
                        let setter_name = format_ident!("set_{}", getter_name);
                        let struct_to_name = format_ident!("Vector{}{}", l, name);

                        let getter_vals =
                            combination.iter().map(|(index, _)| quote! { self[#index] });
                        let setter_vals =
                            combination
                                .iter()
                                .enumerate()
                                .map(|(setter_index, (index, _))| {
                                    quote! {
                                        self[#index] = val[#setter_index];
                                    }
                                });

                        quote! {
                            #[inline]
                            pub fn #getter_name(&self) -> #struct_to_name {
                                [#(#getter_vals),*].into()
                            }

                            pub fn #setter_name(&mut self, val: #struct_to_name) {
                                #(#setter_vals)*
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                quote! {
                    impl #struct_name {
                        #(#funcs_combinations)*
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! { #(#funcs_combinations)* }
    });

    (quote! {
        #(#res)*
    })
    .into()
}
// ----- Vector END -----
