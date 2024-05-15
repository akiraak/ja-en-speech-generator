use std::error::Error;
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateSpeechRequestArgs, SpeechModel, Voice,
    },
    Client,
};
use tauri::Window;
use crate::utils::audio;

pub async fn make_japanese(input: &str, file_prefix: &str, window: &Window) -> Result<(String, String), Box<dyn Error>> {
    println!("make_japanese");

    let system_context = "あなたは生徒に質問に答える教師です。300文字以内で答えてください。";
    let response_text = exec_chatgpt(system_context, input, window).await?;

    let org_mp3_file_path = format!("{}{}-org-ja.mp3", crate::utils::file::OUTPUT_FILE_BASE_PATH, file_prefix);
    let mp3_file_path = format!("{}{}-ja.mp3", crate::utils::file::OUTPUT_FILE_BASE_PATH, file_prefix);
    get_mp3(response_text.clone(), org_mp3_file_path.clone(), window).await?;

    audio::marge_mp3_files(&[
            "../silence_1sec.mp3",
            &org_mp3_file_path
        ], &mp3_file_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    Ok((response_text, mp3_file_path))
}

pub async fn make_english(japanese_text: &str, file_prefix: &str, window: &Window) -> Result<(String, String), Box<dyn Error>> {
    println!("make_english");
    let system_context = "あなたは日本語を英語に翻訳するアシスタントです。";
    let response_text = exec_chatgpt(system_context, japanese_text, window).await?;

    let org_mp3_file_path = format!("{}{}-org-en.mp3", crate::utils::file::OUTPUT_FILE_BASE_PATH, file_prefix);
    let mp3_file_path = format!("{}{}-en.mp3", crate::utils::file::OUTPUT_FILE_BASE_PATH, file_prefix);
    get_mp3(response_text.clone(), org_mp3_file_path.clone(), window).await?;

    audio::marge_mp3_files(&[
            "../silence_1sec.mp3",
            &org_mp3_file_path
        ], &mp3_file_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    Ok((response_text, mp3_file_path))
}

async fn exec_chatgpt(system_context: &str, user_context: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("exec_chatgpt");

    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-4o")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default().content(system_context).build()?.into(),
            ChatCompletionRequestUserMessageArgs::default().content(user_context).build()?.into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;
    for choice in response.choices {
        if let Some(content) = choice.message.content {
            window.emit("add_to_output", Some(format!("
# ChatGPT
Response: {}", content))).expect("failed to emit event");
            return Ok(content);
        }
    }
    window.emit("add_to_output", Some("ChatGPTの呼び出してエラーになりました。".to_string())).expect("failed to emit event");
    Err("ChatGPTの呼び出してエラーになりました。".into())
}

async fn get_mp3(text: String, file_path: String, window: &Window) -> Result<(), Box<dyn Error>> {
    println!("get_mp3");

    window.emit("add_to_output", Some("
# TTS")).expect("failed to emit event");

    let client = Client::new();
    let request = CreateSpeechRequestArgs::default()
        .input(text)
        .voice(Voice::Nova)
        .model(SpeechModel::Tts1Hd)
        .build()?;

    let response = client.audio().speech(request).await?;
    response.save(file_path.clone()).await?;

    window.emit("add_to_output", Some(format!("FilePath: {}", file_path))).expect("failed to emit event");

    Ok(())
}
