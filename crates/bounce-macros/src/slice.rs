use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, DeriveInput, Ident, Meta, NestedMeta, Path};

pub(crate) struct WithNotionAttr {
    pub path: Path,
}

impl WithNotionAttr {
    fn parse(meta: &Meta) -> syn::Result<Option<Vec<Self>>> {
        match meta {
            Meta::Path(m) => {
                if !m.is_ident("with_notion") {
                    return Ok(None);
                }

                Err(syn::Error::new_spanned(m, "expected list of types"))
            }
            Meta::List(m) => {
                if !m.path.is_ident("with_notion") {
                    return Ok(None);
                }

                let mut attrs = Vec::new();

                for attr in m.nested.iter() {
                    match attr {
                        NestedMeta::Meta(Meta::Path(m)) => {
                            attrs.push(Self { path: m.clone() });
                        }
                        _ => return Err(syn::Error::new_spanned(attr, "expected type")),
                    }
                }

                Ok(Some(attrs))
            }
            Meta::NameValue(m) => {
                if !m.path.is_ident("with_notion") {
                    return Ok(None);
                }

                Err(syn::Error::new_spanned(m, "expected list of types"))
            }
        }
    }
}

pub(crate) struct ObservedAttr {
    ident: Ident,
}

impl ObservedAttr {
    fn parse(meta: &Meta) -> syn::Result<Option<Self>> {
        match meta {
            Meta::Path(m) => match m.get_ident() {
                Some(m) => {
                    if m == "observed" {
                        return Ok(Some(Self {
                            ident: m.to_owned(),
                        }));
                    }

                    Ok(None)
                }
                None => Ok(None),
            },
            Meta::List(m) => {
                if !m.path.is_ident("observed") {
                    return Ok(None);
                }

                Err(syn::Error::new_spanned(
                    m,
                    "observed attribute accepts no argument",
                ))
            }
            Meta::NameValue(m) => {
                if !m.path.is_ident("observed") {
                    return Ok(None);
                }

                Err(syn::Error::new_spanned(
                    m,
                    "observed attribute accepts no argument",
                ))
            }
        }
    }
}

pub(crate) enum BounceAttr {
    WithNotion(WithNotionAttr),
    Observed(ObservedAttr),
}

impl Parse for BounceAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        todo!()
    }
}

#[derive(Default)]
pub(crate) struct BounceAttrs {
    pub notions: Vec<WithNotionAttr>,
    pub observed: Option<ObservedAttr>,
}

impl Parse for BounceAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = Punctuated::<BounceAttr, Comma>::parse_terminated(input)?;

        let mut this = Self::default();

        for attr in attrs {
            match attr {
                BounceAttr::WithNotion(m) => {
                    this.notions.push(m);
                }
                BounceAttr::Observed(m) => {
                    if this.observed.is_some() {
                        return Err(syn::Error::new_spanned(
                            m.ident,
                            "you can only have 1 observed attribute",
                        ));
                    }

                    this.observed = Some(m);
                }
            }
        }

        Ok(this)
    }
}

impl BounceAttrs {
    pub fn parse_one(&mut self, attr: &Attribute) -> syn::Result<()> {
        if !attr.path.is_ident("bounce") {
            return Ok(());
        }

        let meta = attr.parse_meta()?;

        match meta {
            Meta::Path(m) => {
                return Err(syn::Error::new_spanned(
                    m,
                    "expected additional attribute content, found #[bounce]",
                ));
            }
            Meta::List(m) => {
                for attr in m.nested.iter() {
                    match attr {
                        NestedMeta::Meta(ref m) => {
                            if let Some(m) = WithNotionAttr::parse(m)? {
                                self.notions.extend(m);

                                continue;
                            }

                            if ObservedAttr::parse(m)?.is_some() {
                                if self.observed {
                                    return Err(syn::Error::new_spanned(
                                        m,
                                        "you can only have 1 observed attribute",
                                    ));
                                }

                                self.observed = true;

                                continue;
                            }
                            return Err(syn::Error::new_spanned(attr, "unknown attribute"));
                        }
                        NestedMeta::Lit(ref l) => {
                            return Err(syn::Error::new_spanned(
                                l,
                                "expected additional attribute content, found literal",
                            ));
                        }
                    }
                }
            }
            Meta::NameValue(m) => {
                return Err(syn::Error::new_spanned(
                    m,
                    "expected bracketed list, found name value pair",
                ));
            }
        }

        Ok(())
    }

    pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = Self::default();

        for attr in attrs {
            this.parse_one(attr)?;
        }

        Ok(this)
    }

    pub fn notion_idents(&self) -> Vec<Path> {
        self.notions.iter().map(|m| m.path.clone()).collect()
    }

    pub fn create_notion_apply_impls(&self, notion_ident: &Ident) -> Vec<TokenStream> {
        let idents = self.notion_idents();
        let mut notion_apply_impls = Vec::new();

        for ident in idents {
            let notion_apply_impl = quote! {
                let #notion_ident = match <::std::rc::Rc::<dyn std::any::Any>>::downcast::<#ident>(#notion_ident) {
                    ::std::result::Result::Ok(m) => return ::bounce::WithNotion::<#ident>::apply(::std::clone::Clone::clone(&self), m),
                    ::std::result::Result::Err(e) => e,
                };
            };

            notion_apply_impls.push(notion_apply_impl);
        }

        notion_apply_impls
    }

    pub fn create_notion_id_impls(&self) -> Vec<TokenStream> {
        self.notions
            .iter()
            .map(|m| {
                let path = &m.path;

                quote! { ::std::any::TypeId::of::<#path>() }
            })
            .collect()
    }
}

pub(crate) fn macro_fn(input: DeriveInput) -> TokenStream {
    let bounce_attrs = match BounceAttrs::parse(&input.attrs) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };

    let notion_ident = Ident::new("notion", Span::mixed_site());
    let notion_apply_impls = bounce_attrs.create_notion_apply_impls(&notion_ident);
    let notion_ids_impls = bounce_attrs.create_notion_id_impls();

    let type_ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let impl_observed = bounce_attrs.observed.then(|| {
        quote! {
            fn changed(self: ::std::rc::Rc<Self>) {
                ::bounce::Observed::changed(self);
            }
        }
    });

    quote! {
        #[automatically_derived]
        impl #impl_generics ::bounce::Slice for #type_ident #ty_generics #where_clause {
            type Action = <Self as ::bounce::__vendored::yew::functional::Reducible>::Action;

            fn reduce(self: ::std::rc::Rc<Self>, action: Self::Action) -> ::std::rc::Rc<Self> {
                ::bounce::__vendored::yew::functional::Reducible::reduce(self, action)
            }

            fn apply(self: ::std::rc::Rc<Self>, #notion_ident: ::std::rc::Rc<dyn ::std::any::Any>) -> ::std::rc::Rc<Self> {
                #(#notion_apply_impls)*

                self
            }

            fn notion_ids(&self) -> ::std::vec::Vec<::std::any::TypeId> {
                ::std::vec![#(#notion_ids_impls,)*]
            }

            #impl_observed
        }
    }
}
