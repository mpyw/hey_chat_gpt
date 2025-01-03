use proc_macro2::TokenStream;
use quote::quote;
use std::fs;
use syn::spanned::Spanned;

mod query;
use query::{query, Message, Role};

mod cache;
use cache::{cache_result, get_cache_file_path, hash_content, load_cache};

mod macro_;
pub use macro_::{IntoSynRes, MacroInput};

mod util;
use util::extract_rust_codes;

const DEFAULT_MODEL: &str = "gpt-4o";

pub fn take_care_of_the_rest(
    MacroInput {
        vis,
        model,
        prompt: _,
        seed,
        max_completion_tokens,
    }: MacroInput,
    system_message: &str,
) -> syn::Result<TokenStream> {
    let span = vis.span();
    let source_file_path = span.source_file().path();
    let Ok(content) = fs::read_to_string(source_file_path) else {
        // Rust Analyzer対策
        return Ok(TokenStream::new());
    };

    let cache_path = get_cache_file_path(&content);
    let cache = load_cache(&content);

    if let Some(cache) = cache {
        return Ok(file_content2token_stream(&cache));
    }

    let api_key = std::env::var("OPENAI_API_KEY").into_syn(span)?;
    if api_key == "DEBUG" {
        return Ok(TokenStream::new());
    }

    let system_message = Message {
        role: Role::User, // 本当はSystemとしたいがo1-previewで撤廃されたらしい
        content: system_message.to_string(),
    };
    let user_message = Message {
        role: Role::User,
        content: content.clone(),
    };
    let messages = vec![system_message, user_message];

    let model = model.unwrap_or(DEFAULT_MODEL.to_string());
    let seed = match seed {
        Some(seed) => seed,
        None => hash_content(&content),
    };
    let Message {
        content: res_code, ..
    } = query(
        &api_key,
        model,
        &messages,
        seed,
        max_completion_tokens,
        &cache_path,
    )
    .into_syn(span)?;

    cache_result(&content, &res_code);

    Ok(file_content2token_stream(&res_code))
}

fn file_content2token_stream(res_code: &str) -> TokenStream {
    let codes = extract_rust_codes(res_code);

    let res_code = match codes.len() {
        0 => res_code.to_string(),
        _ => codes.join("\n"),
    };

    match res_code.parse() {
        Ok(ok) => ok,
        Err(_) => quote! { compile_error!(#res_code); },
    }
}
