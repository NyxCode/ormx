use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Result};

pub fn box_future() -> TokenStream {
    quote!(ormx::exports::futures::future::BoxFuture)
}

pub fn box_stream() -> TokenStream {
    quote!(ormx::exports::futures::stream::BoxStream)
}

pub fn set_once<T>(opt: &mut Option<T>, v: T) -> Result<()> {
    match opt.replace(v) {
        None => Ok(()),
        Some(_) => Err(Error::new(Span::call_site(), "duplicate attribute")),
    }
}

pub fn missing_attr(attr: &str) -> Error {
    Error::new(
        Span::call_site(),
        format!(r#"missing #[ormx({})] attribute"#, attr),
    )
}
