use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Ident, Path, Result, Token, Type,
};

pub enum TableAttr {
    // table = <string>
    Table(String),
    // id = <ident>
    Id(Ident),
    // insertable [= [<attribute>]* <ident>]?
    Insertable(Option<Insertable>),
    // deletable
    Deletable(()),
    // order_by = <string>
    OrderBy(String),
}

pub struct Insertable {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
}

pub enum TableFieldAttr {
    // column = <string>
    Column(String),
    // custom_type
    CustomType(()),
    // default
    Default(()),
    // get_one [= <ident>]? [(<type>)]?
    GetOne(Getter),
    // get_optional [= <ident>]? [(<type>)]?
    GetOptional(Getter),
    // get_many [= <ident>]? [(<type>)]?
    GetMany(Getter),
    // set [= <ident>]?
    Set(Option<Ident>),
    // by_ref
    ByRef(()),
    // insert_attribute = <attribute>
    InsertAttr(AnyAttribute),
}

#[derive(Clone)]
pub struct Getter {
    pub func: Option<Ident>,
    pub arg_ty: Option<Type>,
}

pub enum PatchAttr {
    // table = <string>
    TableName(String),
    Table(Path),
    Id(String),
}

pub enum PatchFieldAttr {
    // column = <string>
    Column(String),
    CustomType(()),
    ByRef(()),
}

impl Parse for Getter {
    fn parse(input: ParseStream) -> Result<Self> {
        let func = if input.peek(syn::token::Eq) {
            input.parse::<syn::token::Eq>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };
        let arg_ty = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Some(content.parse::<Type>()?)
        } else {
            None
        };
        Ok(Getter { func, arg_ty })
    }
}

impl Parse for Insertable {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ident: input.parse()?,
        })
    }
}

pub fn parse_attrs<A: Parse>(attrs: &[Attribute]) -> Result<Vec<A>> {
    let attrs = attrs
        .iter()
        .filter(|a| a.path().is_ident("ormx"))
        .map(|a| a.parse_args_with(Punctuated::<A, Token![,]>::parse_terminated))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(attrs)
}

/// implements `syn::parse::Parse` for the given type
macro_rules! impl_parse {
    // entry point
    ($i:ident {
        $( $s:literal => $v:ident( $($t:tt)* ) ),*
    }) => {
        impl syn::parse::Parse for $i {
            #[allow(clippy::redundant_closure_call)]
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let ident = input.parse::<syn::Ident>()?;
                match &*ident.to_string() {
                    $( $s => (impl_parse!($($t)*))(input).map(Self::$v), )*
                    _ => Err(input.error("unknown attribute"))
                }
            }
        }
    };
    () => ( |_: ParseStream| Ok(()) );
    // parse either "= {value}" or return None
    ((= $t:tt)?) => {
        |i: ParseStream| if i.peek(syn::Token![=]) {
            i.parse::<syn::Token![=]>()?;
            #[allow(clippy::redundant_closure_call)]
            (impl_parse!($t))(i).map(Some)
        } else {
            Ok(None)
        }
    };
    // parse "= {value}"
    (= $x:tt) => ( |i: ParseStream| {
        i.parse::<syn::Token![=]>()?;
        #[allow(clippy::redundant_closure_call)]
        (impl_parse!($x))(i)
    } );
    (String) => ( |i: ParseStream| i.parse().map(|s: syn::LitStr| s.value()) );
    (bool) => ( |i: ParseStream| i.parse().map(|s: syn::LitBool| s.value()) );
    ($t:ty) => ( |i: ParseStream| i.parse::<$t>() );
}

impl_parse!(TableAttr {
    "table" => Table(= String),
    "id" => Id(= Ident),
    "insertable" => Insertable((= Insertable)?),
    "deletable" => Deletable(),
    "order_by" => OrderBy(= String)
});

impl_parse!(TableFieldAttr {
    "column" => Column(= String),
    "get_one" => GetOne(Getter),
    "get_optional" => GetOptional(Getter),
    "get_many" => GetMany(Getter),
    "set" => Set((= Ident)?),
    "custom_type" => CustomType(),
    "default" => Default(),
    "by_ref" => ByRef(),
    "insert_attribute" => InsertAttr(= AnyAttribute)
});

impl_parse!(PatchAttr {
    "table" => Table(= Path),
    "table_name" => TableName(= String),
    "id" => Id(= String)
});

impl_parse!(PatchFieldAttr {
    "column" => Column(= String),
    "custom_type" => CustomType(),
    "by_ref" => ByRef()
});

pub struct AnyAttribute(pub Vec<Attribute>);
impl Parse for AnyAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        input.call(Attribute::parse_outer).map(Self)
    }
}
