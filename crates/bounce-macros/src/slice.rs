use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parenthesized, Attribute, DeriveInput, Ident, Meta, Type};

pub(crate) struct WithNotionAttr {
    notion_idents: Vec<Type>,
}

impl WithNotionAttr {
    fn parse_parens_content(input: ParseStream<'_>) -> syn::Result<ParseBuffer<'_>> {
        let content;

        parenthesized!(content in input);

        Ok(content)
    }

    fn try_parse(input: ParseStream<'_>) -> syn::Result<Option<Self>> {
        let ident = input.parse::<Ident>()?;

        if ident != "with_notion" {
            return Ok(None);
        }

        let content = Self::parse_parens_content(input)?;

        let idents = Punctuated::<Type, Comma>::parse_terminated(&content)?;

        Ok(Some(Self {
            notion_idents: idents.into_iter().collect(),
        }))
    }
}

pub(crate) struct ObservedAttr {
    ident: Ident,
}

impl ObservedAttr {
    fn try_parse(input: ParseStream<'_>) -> syn::Result<Option<Self>> {
        let ident = input.parse::<Ident>()?;

        if ident != "observed" {
            return Ok(None);
        }

        Ok(Some(Self { ident }))
    }
}

pub(crate) enum BounceAttr {
    WithNotion(WithNotionAttr),
    Observed(ObservedAttr),
}

impl Parse for BounceAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let forked_input = input.fork();
        if let Some(m) = ObservedAttr::try_parse(&forked_input)? {
            input.advance_to(&forked_input);
            return Ok(Self::Observed(m));
        }

        let forked_input = input.fork();
        if let Some(m) = WithNotionAttr::try_parse(&forked_input)? {
            input.advance_to(&forked_input);
            return Ok(Self::WithNotion(m));
        }

        Err(input.error("unknown attribute: expected either with_notion or observed"))
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

        let other = attr.parse_args::<BounceAttrs>()?;

        if let Some(m) = other.observed {
            if self.observed.is_some() {
                return Err(syn::Error::new_spanned(
                    m.ident,
                    "you can only have 1 observed attribute",
                ));
            }

            self.observed = Some(m);
        }

        self.notions.extend(other.notions);

        Ok(())
    }

    pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = Self::default();

        for attr in attrs {
            this.parse_one(attr)?;
        }

        Ok(this)
    }

    pub fn notion_idents(&self) -> Vec<Type> {
        self.notions
            .iter()
            .flat_map(|m| m.notion_idents.clone())
            .collect()
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
        self.notion_idents()
            .iter()
            .map(|m| {
                quote! { ::std::any::TypeId::of::<#m>() }
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

    let impl_observed = bounce_attrs.observed.is_some().then(|| {
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
