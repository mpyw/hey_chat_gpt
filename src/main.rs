fn main() {
    println!("{}", fib(10));
}

hey_chat_gpt::あとは任せた!(
    model = "o1-preview";
    seed = 10;
    max_completion_tokens = 4096;
    "0, 1で始まるフィボナッチ数列(a_{n-1} + a_{n} = a_{n+1})を生成する関数を実装してください。f(1) = 0, f(2) = 1, f(3) = 1, f(4) = 2となるように十分注意してください。"
);
