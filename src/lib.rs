//! # A proc-macro to delegate implementation to the ChatGPT API.
//!
//! ChatGPT APIに実装を代行してもらうマクロです。
//!
//! **This crate requires `nightly` toolchain!**
//!
//! 本クレートでは `nightly` ツールチェイン必須です！
//!
//! - [`do_it`](crate::do_it!)
//! - [`あとは任せた`](crate::あとは任せた!)
//!
//! ```rust
//! use hey_chat_gpt::do_it;
//!
//! fn main() {
//!     println!("{}", fib(10));
//! }
//!
//! do_it!();
//! # fn fib(n: usize) -> usize {
//! #     match n {
//! #         m @ 0..=1 => m,
//! #         m => fib(m - 2) + fib(m - 1)
//! #     }
//! # }
//! ```
//!
//! The difference between the two macros is whether they are in English or Japanese.
//!
//! この2つのマクロの違いは、英語か日本語かです。
//!
//! Please see the links for each macro for more details.
//!
//! 詳細は各マクロのリンク先で読んでください。
//!
//! # Preparation
//!
//! To compile, run the below command.
//!
//! ```bash
//! OPENAI_API_KEY=sk-YOUR-API-KEY RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo +nightly run
//! ```
//!
//! - `OPENAI_API_KEY`: api key.
//! - `RUSTFLAGS=...`: to enable [`source_file` method](https://doc.rust-lang.org/proc_macro/struct.Span.html#method.source_file) of [Span](https://doc.rust-lang.org/proc_macro/struct.Span.html).
//! - `cargo +nightly run`: the reason of specify `nightly` is same as above.
//!
//! These settings can also be enable by setting files.
//!
//! `.cargo/config.toml`
//!
//! ```toml:.cargo/config.toml
//! [build]
//! rustflags = ["--cfg=procmacro2_semver_exempt"]
//!
//! [env]
//! OPENAI_API_KEY = "sk-YOUR-API-KEY"
//! ```
//!
//! `rust-toolchain.toml`
//!
//! ```toml:rust-toolchain.toml
//! [toolchain]
//! channel = "nightly"
//! ```
//!
//! In this case, the options are not necessary.
//!
//! ```bash
//! cargo run
//! ```
//!
//! # 使用のための準備
//!
//! コンパイルするには以下を実行します。
//!
//! ```bash
//! OPENAI_API_KEY=sk-YOUR-API-KEY RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo +nightly run
//! ```
//!
//! - `OPENAI_API_KEY`: 取得してきたOpenAIのAPIキーを設定してください。
//! - `RUSTFLAGS=...`: [Span](https://doc.rust-lang.org/proc_macro/struct.Span.html) の [`source_file`](https://doc.rust-lang.org/proc_macro/struct.Span.html#method.source_file) メソッドを使用するために指定しています。
//! - `cargo +nightly run`: `nightly` の指定理由は上記と同じです。
//!
//! 設定ファイルを通しての設定も可能です。
//!
//! `.cargo/config.toml`
//!
//! ```toml:.cargo/config.toml
//! [build]
//! rustflags = ["--cfg=procmacro2_semver_exempt"]
//!
//! [env]
//! OPENAI_API_KEY = "sk-YOUR-API-KEY"
//! ```
//!
//! `rust-toolchain.toml`
//!
//! ```toml:rust-toolchain.toml
//! [toolchain]
//! channel = "nightly"
//! ```
//!
//! この場合オプションは不要になります。
//!
//! ```bash
//! cargo run
//! ```

mod impls;

use proc_macro::TokenStream;
use syn::Error;

const ENGLISH_MESSAGE: &'static str = r#"I'm the administrator of this system. You are an AI assistant of this system helping with Rust programming, and you are called through `do_it` proc-macro. Generate Rust code based on the user's input as proc-macro (`do_it` macro) output. Ensure the code is idiomatic, adheres to Rust best practices, and includes comments for clarity. All your answers will be treated as `String` values and converted to `proc_macro2::TokenStream` , so your answers must be valid Rust code. **Anything that is not Rust code must be in a comment, and you must not output anything that would prevent the conversion. And User input other than macros remains, so be careful not to create duplicates. (For example, if you output a main function, it may conflict with a user-defined main function and cause a compilation error. Or `do_it` macro may be called from within the main function, in which case you should not print the main function itself.)**. What follows is inputs of the user who uses this system:

"#;
const JAPANESE_MESSAGE: &'static str = r#"私はこのシステムの管理者です。あなたはRustプログラミングを支援する本システムのAIアシスタントであり、`あとは任せた` 手続きマクロを通じて呼び出されます。ユーザーの入力に基づいてRustコードを `あとは任せた` マクロの出力として生成してほしいです。コードはRustのベストプラクティスに従い、明確さを保つための日本語のコメントを含めるようにしてください。回答はすべて `String` 値として扱われ、`proc_macro2::TokenStream` に変換されるため、回答は有効なRustコードである必要があります。**Rustコード以外のものはすべてコメント内に記述する必要があり、Rustコードとして変換しようとするとエラーになるものを出力してはなりません。そして、マクロ以外のユーザー入力はそのまま残るため、重複などをしないように注意してください。(たとえば、 `main` 関数を出力すると、ユーザー定義の `main` 関数と競合してコンパイルエラーが発生する可能性があります。あるいは、 `あとは任せた` マクロはmain関数の中からよばれているかもしれません。その時にmain関数ごと出力してはいけません。)** ここからは本システム利用者の入力になります:

"#;

/// A macro to delegate implementation to the ChatGPT API.
///
/// **This crate requires `nightly` toolchain!**
///
/// This macro sends the entire file containing it to the [OpenAI API](https://platform.openai.com/),
/// and replaces it with the result returned by the API.
///
/// # Example
///
/// ```rust
/// use hey_chat_gpt::do_it;
///
/// fn main() {
///     println!("{}", fib(10));
/// }
///
/// do_it!();
/// # fn fib(n: usize) -> usize {
/// #     match n {
/// #         m @ 0..=1 => m,
/// #         m => fib(m - 2) + fib(m - 1)
/// #     }
/// # }
/// ```
///
/// # Options
///
/// You can pass several options in the form of `key = value`.
/// Additionally, although it's not strictly necessary to mention here,
/// you can pass string literals as well, which can be used to provide a prompt.
///
/// | key                   | Type   | Default         | Possible Values               | Description |
/// |:----------------------|:-------|:----------------|:-------------------------------|:------------|
/// | model                 | String | "gpt-4o"        | "o1-preview", etc.             | Specifies the GPT model to use. |
/// | seed                  | Integer| File hash       | Integer value ≤ 9223372036854775807 | Provides a seed for reproducibility. Try this if the default results are unsatisfactory. |
/// | max_completion_tokens | Integer| None            | | Sets the maximum number of tokens for the response. Might help when the output is truncated (unverified). |
///
/// Example with options:
///
/// ```rust
/// fn main() {
///     println!("{}", fib(10));
/// }
///
/// hey_chat_gpt::do_it!(
///     model = "o1-preview";
///     seed = 20;
///     max_completion_tokens = 4096;
///     "Implement a function that generates a Fibonacci sequence (a_{n-1} + a_{n} = a_{n+1}) starting with 1, 1."
/// );
/// # fn fib(n: usize) -> usize {
/// #     match n {
/// #         m @ 0..=1 => m,
/// #         m => fib(m - 2) + fib(m - 1)
/// #     }
/// # }
/// ```
///
/// # Preparation
///
/// To compile, run the below command.
///
/// ```bash
/// OPENAI_API_KEY=sk-YOUR-API-KEY RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo +nightly run
/// ```
///
/// - `OPENAI_API_KEY`: api key.
/// - `RUSTFLAGS=...`: to enable [`source_file` method](https://doc.rust-lang.org/proc_macro/struct.Span.html#method.source_file) of [Span](https://doc.rust-lang.org/proc_macro/struct.Span.html).
/// - `cargo +nightly run`: the reason of specify `nightly` is same as above.
///
/// These settings can also be enable by setting files.
///
/// `.cargo/config.toml`
///
/// ```toml:.cargo/config.toml
/// [build]
/// rustflags = ["--cfg=procmacro2_semver_exempt"]
///
/// [env]
/// OPENAI_API_KEY = "sk-YOUR-API-KEY"
/// ```
///
/// `rust-toolchain.toml`
///
/// ```toml:rust-toolchain.toml
/// [toolchain]
/// channel = "nightly"
/// ```
///
/// In this case, the options are not necessary.
///
/// ```bash
/// cargo run
/// ```
#[proc_macro]
pub fn do_it(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as impls::MacroInput);

    impls::do_it(input, ENGLISH_MESSAGE)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// ChatGPT APIに実装を代行してもらうマクロです。
///
/// **本クレートでは `nightly` ツールチェイン必須です！**
///
/// このマクロを記述したファイル全体を[OpenAI API](https://platform.openai.com/)に投げ、返ってきた結果で置換します。
///
/// # Example
///
/// ```rust
/// use hey_chat_gpt::あとは任せた;
///
/// fn main() {
///     println!("{}", fib(10));
/// }
///
/// あとは任せた!();
/// # fn fib(n: usize) -> usize {
/// #     match n {
/// #         m @ 0..=1 => m,
/// #         m => fib(m - 2) + fib(m - 1)
/// #     }
/// # }
/// ```
///
/// # Option
///
/// `key = value` 形式でいくつかのオプションを渡すことができます。また、(ここに記述する必要性はないですが)文字列リテラルも受け取れるのでここにプロンプトを記述することもできます。
///
/// | key                   | 型     | デフォルト       | 候補                            | 説明 |
/// |:----------------------|:------ |:---------------|:-------------------------------|:-----|
/// | model                 | 文字列  | "gpt-4o"       | "o1-preview" 等                | 使用するGPTのモデルを指定します。 |
/// | seed                  | 整数値 | ファイルハッシュ   | 9223372036854775807 以下の整数値 | 再現性確保のために与えるシード値を与えます。デフォルトだと芳しくない結果になった時に指定してみてください。 |
/// | max_completion_tokens | 整数値 | 指定なし          | | 返答の最大トークン数を設定します。生成が中途半端になった時に使えるかも...？(未検証) |
///
/// オプションを指定した場合の例
///
/// ```rust
/// fn main() {
///     println!("{}", fib(10));
/// }
///
/// hey_chat_gpt::あとは任せた!(
///     model = "o1-preview";
///     seed = 20;
///     max_completion_tokens = 4096;
///     "1, 1で始まるフィボナッチ数列(a_{n-1} + a_{n} = a_{n+1})を生成する関数を実装してください。"
/// );
/// # fn fib(n: usize) -> usize {
/// #     match n {
/// #         m @ 0..=1 => m,
/// #         m => fib(m - 2) + fib(m - 1)
/// #     }
/// # }
/// ```
///
/// # 使用のための準備
///
/// コンパイルするには以下を実行します。
///
/// ```bash
/// OPENAI_API_KEY=sk-YOUR-API-KEY RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo +nightly run
/// ```
///
/// - `OPENAI_API_KEY`: 取得してきたOpenAIのAPIキーを設定してください。
/// - `RUSTFLAGS=...`: [Span](https://doc.rust-lang.org/proc_macro/struct.Span.html) の [`source_file`](https://doc.rust-lang.org/proc_macro/struct.Span.html#method.source_file) メソッドを使用するために指定しています。
/// - `cargo +nightly run`: `nightly` の指定理由は上記と同じです。
///
/// 設定ファイルを通しての設定も可能です。
///
/// `.cargo/config.toml`
///
/// ```toml:.cargo/config.toml
/// [build]
/// rustflags = ["--cfg=procmacro2_semver_exempt"]
///
/// [env]
/// OPENAI_API_KEY = "sk-YOUR-API-KEY"
/// ```
///
/// `rust-toolchain.toml`
///
/// ```toml:rust-toolchain.toml
/// [toolchain]
/// channel = "nightly"
/// ```
///
/// この場合オプションは不要になります。
///
/// ```bash
/// cargo run
/// ```
#[proc_macro]
pub fn あとは任せた(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as impls::MacroInput);

    impls::do_it(input, JAPANESE_MESSAGE)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
