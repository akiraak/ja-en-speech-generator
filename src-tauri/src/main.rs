// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateSpeechRequestArgs, SpeechModel, Voice,
    },
    Client,
};
use tauri::{Window};

#[tauri::command]
fn submit_command(command: &str, window: Window) -> String {
    println!("submit_command: {}", command);
    let command_owned = command.to_string();

    tokio::spawn(async move {
        println!("thread");
        //execute_workflow(command_owned, window)
        if let Err(e) = execute_workflow(command_owned, window).await {
            eprintln!("Failed to execute workflow: {}", e);
        }
    });

    println!("Command end");
    format!("Command execution started for: {}", command)
}

async fn execute_workflow(command: String, window: Window) -> Result<(), Box<dyn Error>> {
    println!("execute_workflow");

    let japanese_text = make_japanese(&command, &window).await?;
    let english_text = make_english(&japanese_text, &window).await?;
    save_text(&command, &japanese_text, &english_text, &window);

    Ok(())
}

async fn make_japanese(input: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("make_japanese");

    let system_context = "あなたは生徒に質問に答える教師です。２００文字以内で答えてください。";
    let response_text = exec_chatgpt(system_context, input, window).await?;
    get_mp3(response_text.clone(), format!("../{}.mp3", "japanese"), window).await?;

    Ok(response_text)
}

async fn make_english(japanese_text: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("make_english");
    let system_context = "あなたは日本語を英語に翻訳するアシスタントです。";
    let response_text = exec_chatgpt(system_context, japanese_text, window).await?;
    get_mp3(response_text.clone(), format!("../{}.mp3", "english"), window).await?;

    Ok(response_text)
}

async fn exec_chatgpt(system_context: &str, user_context: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("exec_chatgpt");

    window.emit("add_to_output", Some(format!("
# ChatGPT
SystemContent: {}
UserContent: {}", system_context, user_context))).expect("failed to emit event");

    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-4-turbo")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default().content(system_context).build()?.into(),
            ChatCompletionRequestUserMessageArgs::default().content(user_context).build()?.into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;
    for choice in response.choices {
        println!(
            "{}: Role: {}  Content: {:?}",
            choice.index, choice.message.role, choice.message.content
        );
        if let Some(content) = choice.message.content {
            window.emit("add_to_output", Some(format!("Response: {}", content))).expect("failed to emit event");
            return Ok(content);
        }
    }
    window.emit("add_to_output", Some("ChatGPTの呼び出してエラーになりました。".to_string())).expect("failed to emit event");
    Err("ChatGPTの呼び出してエラーになりました。".into())
}

async fn get_mp3(text: String, file_name: String, window: &Window) -> Result<(), Box<dyn Error>> {
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
    response.save(file_name.clone()).await?;

    window.emit("add_to_output", Some(format!("FilePath: {}", file_name))).expect("failed to emit event");

    Ok(())
}

fn save_text(title: &str, japanese: &str, english: &str, window: &Window) -> String {
    println!("save_text");

    window.emit("add_to_output", Some("
# SaveText")).expect("failed to emit event");

    let file_name = "../output.txt";
    let text = format!("# Title
{}

# Japanese
{}

# English
{}", title, japanese, english);
    std::fs::write(file_name, text).expect("failed to write file");

    window.emit("add_to_output", Some(format!("FilePath: {}", file_name))).expect("failed to emit event");

    file_name.to_string()
}




#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![submit_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
