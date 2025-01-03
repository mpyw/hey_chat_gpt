use proc_macro2::Span;
use syn::Ident;
use syn::LitInt;
use syn::LitStr;
use syn::Token;
use syn::Visibility;
use syn::{parse::Parse, parse::ParseStream};

pub struct MacroInput {
    pub vis: Visibility,
    pub model: Option<String>,
    #[allow(unused)]
    pub prompt: Option<LitStr>,
    pub seed: Option<u64>,
    pub max_completion_tokens: Option<u64>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut model: Option<String> = None;
        let mut seed = None;
        let mut max_completion_tokens = None;
        let mut prompt = None;

        let vis = input.parse::<Visibility>()?;

        fn parse_puncts(input: ParseStream) -> syn::Result<()> {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }
            Ok(())
        }

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                let ident = input.parse::<Ident>()?;
                input.parse::<syn::Token![=]>()?;
                match ident {
                    i if i == "model" => {
                        let value = input.parse::<LitStr>()?;
                        model = Some(value.value());
                    }
                    i if i == "max_completion_tokens" => {
                        let value = input.parse::<LitInt>()?;
                        max_completion_tokens = Some(value.base10_parse()?);
                    }
                    i if i == "seed" => {
                        let value = input.parse::<LitInt>()?;
                        seed = Some(value.base10_parse()?);
                    }
                    _ => return Err(lookahead.error()),
                }
            } else if lookahead.peek(LitStr) {
                prompt = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
            parse_puncts(input)?;
        }

        Ok(Self {
            vis,
            model,
            prompt,
            seed,
            max_completion_tokens,
        })
    }
}

pub trait IntoSynRes<T> {
    fn into_syn(self, span: Span) -> syn::Result<T>;
}

impl<T, E> IntoSynRes<T> for Result<T, E>
where
    E: std::fmt::Display + std::fmt::Debug,
{
    fn into_syn(self, span: Span) -> syn::Result<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(syn::Error::new(span, err)),
        }
    }
}
