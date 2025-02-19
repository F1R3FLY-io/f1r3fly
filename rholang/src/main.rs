use rholang::rust::interpreter::openai_service::OpenAIService;

#[tokio::main]
async fn main() {
    let openai_service = OpenAIService::new();
    let result = openai_service
        .gpt4_chat_completion("What is Bitcoin?")
        .await;

    println!("{:?}", result);
}
