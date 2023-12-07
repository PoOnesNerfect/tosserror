use crate::ast::{Enum, Field, Input, Struct};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma};
use syn::{DeriveInput, GenericArgument, Member, PathArguments, Result, Type};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    match Input::from_syn(node)? {
        Input::Struct(input) => Ok(impl_struct(input)),
        Input::Enum(input) => Ok(impl_enum(input)),
    }
}

fn impl_struct(input: Struct) -> TokenStream {
    let ty = &input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let Some(source) = source_field(&input.fields) else {
        return quote!();
    };

    let trait_name = format_ident!("Toss{}", input.ident);

    let method_name = input
        .attrs
        .prefix
        .map(|p| {
            if p == "self" {
                panic!("prefix value must be specified");
            } else {
                format!("{}_{}", snake_case_trimmed(&p), snake_case_trimmed(ty))
            }
        })
        .unwrap_or_else(|| snake_case_trimmed(ty));
    let toss_method = format_ident!("toss_{}", method_name);
    let with_method = format_ident!("toss_{}_with", method_name);

    let generics = {
        use proc_macro2::Span;

        let mut generics = input.generics.clone();
        generics.params.push(syn::GenericParam::Type(
            Ident::new("__RETURN", Span::call_site()).into(),
        ));
        generics
    };
    let (impl_generics, thiserror_ty_generics, _) = generics.split_for_impl();

    let (args, fields, types) = {
        let mut args = Punctuated::<TokenStream, Comma>::new();
        let mut fields = Punctuated::<Ident, Comma>::new();
        let mut types = Punctuated::<&Type, Comma>::new();

        let non_source = |field: &&Field<'_>| {
            if field.attrs.from.is_some()
                || field.attrs.source.is_some()
                || field.attrs.backtrace.is_some()
            {
                return false;
            }
            match &field.member {
                Member::Named(ident) => {
                    !((ident == "source" && source.member == field.member)
                        || (ident == "backtrace"))
                }
                _ => true,
            }
        };

        for (i, field) in input.fields.iter().filter(non_source).enumerate() {
            let field_ty = field.ty;

            let field_name = if let Some(field_name) = field.original.ident.as_ref() {
                field_name.clone()
            } else {
                format_ident!("_{}", i)
            };

            args.push(quote! {
                #field_name : #field_ty
            });
            fields.push(field_name);
            types.push(field_ty);
        }

        (args, fields, types)
    };

    let source_ty = source.ty;

    let new_struct = match &source.member {
        Member::Named(name) => {
            let backtrace = backtrace_field(&input.fields).map(|backtrace_field| {
                let backtrace_member = &backtrace_field.member;
                if type_is_option(backtrace_field.ty) {
                    quote! {
                        #backtrace_member: ::core::option::Option::Some(std::backtrace::Backtrace::capture()),
                    }
                } else {
                    quote! {
                        #backtrace_member: ::core::convert::From::from(std::backtrace::Backtrace::capture()),
                    }
                }
            });

            quote! {
                #ty {
                    #name : e,
                    #backtrace
                    #fields
                }
            }
        }
        Member::Unnamed(index) => {
            let mut fields2 = Punctuated::<Ident, Comma>::new();
            for (i, field) in fields.iter().enumerate() {
                if index.index as usize == i {
                    fields2.push(format_ident!("e"));
                }
                fields2.push(field.clone());
            }
            if index.index as usize == fields.len() {
                fields2.push(format_ident!("e"));
            }

            quote! {
                #ty (#fields2)
            }
        }
    };

    let with_method_decl = (!args.is_empty()).then(|| quote!{
            fn #with_method<F: FnOnce() -> (#types)> (self, f: F) -> Result<__RETURN, #ty #ty_generics> #where_clause;
        });
    let with_method_impl = (!args.is_empty()).then(|| quote!{
            fn #with_method<F: FnOnce() -> (#types)> (self, f: F) -> Result<__RETURN, #ty #ty_generics> #where_clause {
                self.map_err(|e| {
                    let (#fields) = f();
                    #new_struct
                })
            }
        });

    let visibility = input.attrs.visibility;

    let thiserror_export = {
        #[cfg(feature = "thiserror")]
        let mod_name = format_ident!("__import_thiserror_by_{}", snake_case_trimmed(ty));
        #[cfg(feature = "thiserror")]
        quote! {
            #[doc(hidden)]
            mod #mod_name {
                pub use tosserror::thiserror;
            }
            #[allow(unused_imports)]
            use #mod_name::*;
        }

        #[cfg(not(feature = "thiserror"))]
        quote! {}
    };

    quote! {
        #visibility trait #trait_name #impl_generics {
            fn #toss_method (self, #args) -> Result<__RETURN, #ty #ty_generics> #where_clause;
            #with_method_decl
        }
        impl #impl_generics #trait_name #thiserror_ty_generics for Result<__RETURN, #source_ty> #where_clause {
            fn #toss_method (self, #args) -> Result<__RETURN, #ty #ty_generics> #where_clause {
                self.map_err(|e| {
                    #new_struct
                })
            }
            #with_method_impl
        }

        #thiserror_export
    }
}

fn impl_enum(input: Enum) -> TokenStream {
    let ty = &input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let generics = {
        use proc_macro2::Span;

        let mut generics = input.generics.clone();
        generics.params.push(syn::GenericParam::Type(
            Ident::new("__RETURN", Span::call_site()).into(),
        ));
        generics
    };
    let (impl_generics, thiserror_ty_generics, _) = generics.split_for_impl();

    let visibility = input.attrs.visibility;
    let prefix = input.attrs.prefix;

    let impls: Vec<Option<TokenStream>> = input.variants.iter().map(|variant|{
            if let Some(source) = source_field(&variant.fields) {
                let variant_ident = &variant.ident;
                let trait_name = format_ident!("Toss{}{}", input.ident, variant_ident);

                let method_name = variant
                    .attrs
                    .prefix
                    .as_ref()
                    .or_else(|| prefix.as_ref())
                    .map(|p| {
                        let prefix = if p == "self" {
                            snake_case_trimmed(ty)
                        } else {
                            snake_case(&p)
                        };
                        format!("{}_{}", prefix, snake_case_trimmed(variant_ident))
                    })
                    .unwrap_or_else(|| snake_case_trimmed(variant_ident));
                let toss_method = format_ident!("toss_{}", method_name);
                let with_method = format_ident!("toss_{}_with", method_name);

                let (args, fields, types) = {
                    let mut args = Punctuated::<TokenStream, Comma>::new();
                    let mut fields = Punctuated::<Ident, Comma>::new();
                    let mut types = Punctuated::<&Type, Comma>::new();

                    let non_source = |field: &&Field<'_>| {
                        if field.attrs.from.is_some()
                            || field.attrs.source.is_some()
                            || field.attrs.backtrace.is_some()
                        {
                            return false;
                        }
                        match &field.member {
                            Member::Named(ident) => {
                                !((ident == "source" && source.member == field.member) || (ident == "backtrace"))
                            }
                            _ => true,
                        }
                    };

                    for (i, field) in variant.fields.iter().filter(non_source).enumerate() {
                        let field_ty = field.ty;

                        let field_name = if let Some(field_name) = field.original.ident.as_ref() {
                            field_name.clone()
                        } else {
                            format_ident!("_{}", i)
                        };

                        args.push(quote! {
                            #field_name : #field_ty
                        });
                        fields.push(field_name);
                        types.push(field_ty);
                    }

                    (args, fields, types)
                };

                let source_ty = source.ty;

                let new_struct = match &source.member {
                    Member::Named(name) => {
                        let backtrace = backtrace_field(&variant.fields).map(|backtrace_field| {
                            let backtrace_member = &backtrace_field.member;
                            if type_is_option(backtrace_field.ty) {
                                quote! {
                                    #backtrace_member: ::core::option::Option::Some(std::backtrace::Backtrace::capture()),
                                }
                            } else {
                                quote! {
                                    #backtrace_member: ::core::convert::From::from(std::backtrace::Backtrace::capture()),
                                }
                            }
                        });

                        quote! {
                            #ty :: #variant_ident {
                                #name : e,
                                #backtrace
                                #fields
                            }
                        }
                    }
                    Member::Unnamed(index) => {
                        let mut fields2 = Punctuated::<Ident, Comma>::new();
                        for (i, field) in fields.iter().enumerate() {
                            if index.index as usize == i {
                                fields2.push(format_ident!("e"));
                            }
                            fields2.push(field.clone());
                        }
                        if index.index as usize == fields.len() {
                            fields2.push(format_ident!("e"));
                        }

                        quote! {
                            #ty :: #variant_ident (#fields2)
                        }
                    }
                };

                let with_method_decl = (!args.is_empty()).then(|| quote!{
                    fn #with_method<F: FnOnce() -> (#types)> (self, f: F) -> Result<__RETURN, #ty #ty_generics> #where_clause;
                });
                let with_method_impl = (!args.is_empty()).then(|| quote!{
                    fn #with_method<F: FnOnce() -> (#types)> (self, f: F) -> Result<__RETURN, #ty #ty_generics> #where_clause {
                        self.map_err(|e| {
                            let (#fields) = f();
                            #new_struct
                        })
                    }
                });

                let visibility = variant.attrs.visibility.or(visibility);

                Some(quote! {
                    #visibility trait #trait_name #impl_generics {
                        fn #toss_method (self, #args) -> Result<__RETURN, #ty #ty_generics> #where_clause;
                        #with_method_decl
                    }
                    impl #impl_generics #trait_name #thiserror_ty_generics for Result<__RETURN, #source_ty> #where_clause {
                        fn #toss_method (self, #args) -> Result<__RETURN, #ty #ty_generics> #where_clause {
                            self.map_err(|e| {
                                #new_struct
                            })
                        }
                        #with_method_impl
                    }
                })
            } else {
                None
            }
        }).collect();

    let thiserror_export = {
        #[cfg(feature = "thiserror")]
        let mod_name = format_ident!("__import_thiserror_by_{}", snake_case_trimmed(ty));
        #[cfg(feature = "thiserror")]
        quote! {
            #[doc(hidden)]
            mod #mod_name {
                pub use tosserror::thiserror;
            }
            #[allow(unused_imports)]
            use #mod_name::*;
        }

        #[cfg(not(feature = "thiserror"))]
        quote! {}
    };

    quote! {
        #(#impls)*
        #thiserror_export
    }
}

fn snake_case_trimmed(ident: &Ident) -> String {
    let mut snake = snake_case(ident);
    snake = snake.trim_end_matches("_error").to_owned();
    snake
}

fn snake_case(ident: &Ident) -> String {
    let mut snake = String::new();
    for (i, ch) in ident.to_string().char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

fn source_field<'a, 'b>(fields: &'a [Field<'b>]) -> Option<&'a Field<'b>> {
    for field in fields {
        if field.attrs.from.is_some() || field.attrs.source.is_some() {
            return Some(field);
        }
    }
    for field in fields {
        match &field.member {
            Member::Named(ident) if ident == "source" => return Some(field),
            _ => {}
        }
    }
    None
}

fn backtrace_field<'a, 'b>(fields: &'a [Field<'b>]) -> Option<&'a Field<'b>> {
    for field in fields {
        if field.attrs.backtrace.is_some()
            && Some(&field.member) != source_field(fields).map(|f| &f.member)
        {
            return Some(field);
        }
    }
    for field in fields {
        if type_is_backtrace(field.ty) {
            return Some(field);
        }
    }
    None
}

fn type_is_backtrace(ty: &Type) -> bool {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };

    let last = path.segments.last().unwrap();
    last.ident == "Backtrace" && last.arguments.is_empty()
}

fn type_is_option(ty: &Type) -> bool {
    type_parameter_of_option(ty).is_some()
}

fn type_parameter_of_option(ty: &Type) -> Option<&Type> {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return None,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Option" {
        return None;
    }

    let bracketed = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed,
        _ => return None,
    };

    if bracketed.args.len() != 1 {
        return None;
    }

    match &bracketed.args[0] {
        GenericArgument::Type(arg) => Some(arg),
        _ => None,
    }
}
