use proc_macro2::{Span, TokenStream};
use quote::quote;
use reqwest::blocking::{Client, RequestBuilder};
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::Ident;
use syn::LitInt;
use syn::LitStr;
use syn::Token;
use syn::Visibility;
use syn::{parse::Parse, parse::ParseStream};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
    seed: u64,
    max_completion_tokens: Option<u64>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Choice {
    index: u64,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ResponseBody {
    id: String,
    object: String,
    created: u64,
    choices: Vec<Choice>,
    usage: Usage,
}

fn common_header(api_key: &str) -> RequestBuilder {
    let api_key_field = format!("Bearer {}", api_key);

    Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", api_key_field.as_str())
}

fn query(
    api_key: &str,
    input_messages: &[Message],
    seed: u64,
    max_completion_tokens: Option<u64>,
    cache_path: &std::path::Path,
) -> anyhow::Result<Message> {
    let response_body = common_header(api_key)
        .json(&RequestBody {
            model: "gpt-4o".to_string(),
            messages: Vec::from(input_messages),
            seed: seed % 9223372036854775807,
            max_completion_tokens,
        })
        .send()?;

    let body = response_body.text()?;

    let mut response_body = match serde_json::from_str::<ResponseBody>(&body) {
        Ok(parsed) => parsed,
        Err(e) => {
            let res = format!("---\n{}\n---\n{}", e, body);
            fs::write(cache_path, res).unwrap_or(());
            return Err(e.into());
        }
    };

    let res = response_body.choices.remove(0).message;
    Ok(res)
}

pub fn take_care_of_the_rest(
    MacroInput {
        vis,
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
        return cache.parse().into_syn(span);
    }

    let api_key = std::env::var("OPENAI_API_KEY").into_syn(span)?;

    let system_message = Message {
        role: Role::System,
        content: system_message.to_string(),
    };
    let user_message = Message {
        role: Role::User,
        content: content.clone(),
    };
    let messages = vec![system_message, user_message];

    let seed = match seed {
        Some(seed) => seed,
        None => hash_content(&content),
    };
    let Message {
        content: res_code, ..
    } = query(
        &api_key,
        &messages,
        seed,
        max_completion_tokens,
        &cache_path,
    )
    .into_syn(span)?;
    let res_code = remove_markdown(&res_code);

    cache_result(&content, res_code);

    match res_code.parse() {
        Ok(ok) => Ok(ok),
        Err(_) => Ok(quote! { compile_error!(#res_code); }),
    }
}

fn get_cache_file_path(content: &str) -> PathBuf {
    // CargoのOUT_DIRを取得
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not set");
    let cache_file =
        PathBuf::from(out_dir).join(format!("gpt_responses/cache_{}.txt", hash_content(content)));
    cache_file
}

fn load_cache(content: &str) -> Option<String> {
    let cache_file = get_cache_file_path(content);

    // キャッシュが存在するか確認
    if cache_file.exists() {
        // キャッシュが存在する場合は読み込む
        let response = fs::read_to_string(cache_file).expect("Failed to read cache file");
        Some(response)
    } else {
        None
    }
}

fn cache_result(content: &str, response: &str) {
    let cache_file = get_cache_file_path(content);

    // 結果を保存
    fs::write(cache_file, response).expect("Failed to write cache file");
}

fn hash_content(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

pub struct MacroInput {
    vis: Visibility,
    #[allow(unused)]
    prompt: Option<LitStr>,
    seed: Option<u64>,
    max_completion_tokens: Option<u64>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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
                let value = input.parse::<LitInt>()?;
                match ident {
                    i if i == "max_completion_tokens" => {
                        max_completion_tokens = Some(value.base10_parse()?);
                    }
                    i if i == "seed" => {
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
            prompt,
            seed,
            max_completion_tokens,
        })
    }
}

trait IntoSynRes<T> {
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

fn remove_markdown(res_code: &str) -> &str {
    res_code
        .trim()
        .trim_start_matches("```rust")
        .trim_end_matches("```")
}
