use proc_macro2::Span;
use syn::{Error, Result};

macro_rules! stream {
    ($($t:tt)*) => {
        quote!(impl ormx::exports::Stream<Item = $($t)*> + Send + Unpin)
    };
}
pub(crate) use stream;

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
