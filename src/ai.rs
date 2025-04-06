use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    Auth, Elevenlabs,
};
use kalosm::language::{ChatModelExt, OpenAICompatibleChatModel, OpenAICompatibleClient};
use std::env;

use crate::PullRequest;

fn get_gemini_api_key() -> String {
    env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set")
}

async fn get_response_from_gemini_api(prompt: &str) -> String {
    let llm = OpenAICompatibleChatModel::builder()
        .with_model("gemini-2.0-flash")
        .with_client(
            OpenAICompatibleClient::new()
                .with_base_url("https://generativelanguage.googleapis.com/v1beta/openai")
                .with_api_key(get_gemini_api_key()),
        )
        .build();
    println!("API KEY: {}", get_gemini_api_key());
    const SYS_PROMPT: &str = "This is a PR for a new feature, generate a response fit for short form content like tiktok that could go over a minecraft parkour video.";
    let mut generate_character = llm.chat(); //.with_system_prompt(SYS_PROMPT);
    let res = generate_character(prompt).await.unwrap();

    res
}

/// Converts text to speech and saves it to the specified path.
/// The server responds with "audio/mpeg" so we can save as mp3.
async fn text_to_speech(text: &str, store_path: &str) {
    let auth = Auth::from_env().unwrap();
    let elevenlabs = Elevenlabs::new(auth, "https://api.elevenlabs.io/v1/");

    // Create the tts body.
    let tts_body = TtsBody {
        model_id: None,
        text: text.to_string(),
        voice_settings: None,
    };

    // Generate the speech for the text by using the voice with id yoZ06aMxZJJ28mfd3POQ.
    let tts_result = elevenlabs.tts(&tts_body, "NOpBlnGInO9m6vDvFkFC");
    let bytes = tts_result.unwrap();

    // Do what you need with the bytes.
    // The server responds with "audio/mpeg" so we can save as mp3.
    std::fs::write(store_path, bytes).unwrap();
}

pub async fn generate_pull_request_audio(pull_request: PullRequest) {
    println!("Generating text for PR: {}", pull_request.diff_url);
    let text_response = get_response_from_gemini_api(&pull_request.diff_url).await;
    println!("Generating audio for PR: {}", pull_request.diff_url);
    let file_path = format!("data/{}.mp3", pull_request.diff_url);
    println!("Saving audio to: {}", file_path);
    text_to_speech(&text_response, &file_path).await;
}
