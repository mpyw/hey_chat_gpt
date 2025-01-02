mod impls;

use proc_macro::TokenStream;
use syn::Error;

const ENGLISH_MESSAGE: &'static str = r#"You are an AI assistant helping with Rust programming, and you are called through `take_care_of_the_rest` proc-macro. Generate Rust code based on the user's input as proc-macro (`take_care_of_the_rest` macro) output. Ensure the code is idiomatic, adheres to Rust best practices, and includes comments for clarity. All your answers will be treated as `String` values and converted to `proc_macro2::TokenStream` , so your answers must be valid Rust code. **Anything that is not Rust code must be in a comment, and you must not output anything that would prevent the conversion. And User input other than macros remains, so be careful not to create duplicates. (For example, if you output a main function, it may conflict with a user-defined main function and cause a compilation error.)**"#;
const JAPANESE_MESSAGE: &'static str = r#"あなたはRustプログラミングを支援するAIアシスタントであり、`後は任せた` 手続きマクロを通じて呼び出されます。ユーザーの入力に基づいてRustコードを `後は任せた` マクロの出力として生成してほしいです。コードはRustのベストプラクティスに従い、明確さを保つための日本語のコメントを含めるようにしてください。回答はすべて `String` 値として扱われ、`proc_macro2::TokenStream` に変換されるため、回答は有効なRustコードである必要があります。**Rustコード以外のものはすべてコメント内に記述する必要があり、Rustコードとして変換しようとするとエラーになるものを出力してはなりません。そして、マクロ以外のユーザー入力はそのまま残るため、重複などをしないように注意してください。(たとえば、 `main` 関数を出力すると、ユーザー定義の `main` 関数と競合してコンパイルエラーが発生する可能性があります。)**"#;

#[proc_macro]
pub fn take_care_of_the_rest(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as impls::MacroInput);

    impls::take_care_of_the_rest(input, ENGLISH_MESSAGE)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn あとは任せた(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as impls::MacroInput);

    impls::take_care_of_the_rest(input, JAPANESE_MESSAGE)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
