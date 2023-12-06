use proc_macro2::TokenStream;
use syn::{Attribute, Error, Meta, Result};

pub struct Attrs<'a> {
    pub source: Option<&'a Attribute>,
    pub from: Option<&'a Attribute>,
    pub backtrace: Option<&'a Attribute>,
    pub visibility: Option<&'a TokenStream>,
}

pub fn get(input: &[Attribute]) -> Result<Attrs> {
    let mut attrs = Attrs {
        source: None,
        from: None,
        backtrace: None,
        visibility: None,
    };

    for attr in input {
        if attr.path().is_ident("source") {
            attr.meta.require_path_only()?;
            if attrs.source.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[source] attribute"));
            }
            attrs.source = Some(attr);
        } else if attr.path().is_ident("backtrace") {
            attr.meta.require_path_only()?;
            if attrs.backtrace.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[backtrace] attribute"));
            }
            attrs.backtrace = Some(attr);
        } else if attr.path().is_ident("visibility") {
            attr.meta.require_list()?;
            if attrs.visibility.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[backtrace] attribute"));
            }
            if let Meta::List(list) = &attr.meta {
                attrs.visibility = Some(&list.tokens);
            }
        } else if attr.path().is_ident("from") {
            match attr.meta {
                Meta::Path(_) => {}
                Meta::List(_) | Meta::NameValue(_) => {
                    // Assume this is meant for derive_more crate or something.
                    continue;
                }
            }
            if attrs.from.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[from] attribute"));
            }
            attrs.from = Some(attr);
        }
    }

    Ok(attrs)
}
