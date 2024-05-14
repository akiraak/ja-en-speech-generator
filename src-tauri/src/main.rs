// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::io::BufReader;
use std::{fs::File, io::Write};
use std::process::Command;
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateSpeechRequestArgs, SpeechModel, Voice,
    },
    Client,
};
use rodio::{Decoder, OutputStream, Sink, Source};
use tauri::{Window};

const OUTPUT_FILE_BASE_PATH: &str = "../data/";
const MARGE_MP3_LIST_FILE_PATH: &str = "../data/marge_mp3_list.txt";

#[tauri::command]
fn submit_command(command: &str, window: Window) -> String {
    println!("submit_command: {}", command);
    let command_owned = command.to_string();

    tokio::spawn(async move {
        //println!("thread");
        if let Err(e) = execute_workflow(command_owned, window).await {
            eprintln!("Failed to execute workflow: {}", e);
        }
    });

    println!("Command end");
    format!("Command execution started for: {}", command)
}

async fn execute_workflow(command: String, window: Window) -> Result<(), Box<dyn Error>> {
    println!("execute_workflow");

    let file_prefix = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();

    let (ja_text, ja_mp3_path) = make_japanese(&command, &file_prefix, &window).await?;
    let (en_text, en_mp3_path) = make_english(&ja_text, &file_prefix, &window).await?;
    save_text(&command, &ja_text, &en_text, &file_prefix, &window).await?;

    let merged_mp3_output_path = format!("{}{}.mp3", OUTPUT_FILE_BASE_PATH, file_prefix);
    marge_mp3_files(&[
            &ja_mp3_path,
            "../silence_1sec.mp3",
            &en_mp3_path
        ], &merged_mp3_output_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    window.emit("add_to_output", Some(format!("
# MargeMP3
FilePath: {}", merged_mp3_output_path))).expect("failed to emit event");

    play_mp3(&merged_mp3_output_path)?;

    window.emit("add_to_output", Some(format!("
Done!"))).expect("failed to emit event");

    Ok(())
}

async fn make_japanese(input: &str, file_prefix: &str, window: &Window) -> Result<(String, String), Box<dyn Error>> {
    println!("make_japanese");

    let system_context = "あなたは生徒に質問に答える教師です。300文字以内で答えてください。";
    let response_text = exec_chatgpt(system_context, input, window).await?;

    let org_mp3_file_path = format!("{}{}-org-ja.mp3", OUTPUT_FILE_BASE_PATH, file_prefix);
    let mp3_file_path = format!("{}{}-ja.mp3", OUTPUT_FILE_BASE_PATH, file_prefix);
    get_mp3(response_text.clone(), org_mp3_file_path.clone(), window).await?;

    marge_mp3_files(&[
            "../silence_1sec.mp3",
            &org_mp3_file_path
        ], &mp3_file_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    Ok((response_text, mp3_file_path))
}

async fn make_english(japanese_text: &str, file_prefix: &str ,window: &Window) -> Result<(String, String), Box<dyn Error>> {
    println!("make_english");
    let system_context = "あなたは日本語を英語に翻訳するアシスタントです。";
    let response_text = exec_chatgpt(system_context, japanese_text, window).await?;

    let org_mp3_file_path = format!("{}{}-org-en.mp3", OUTPUT_FILE_BASE_PATH, file_prefix);
    let mp3_file_path = format!("{}{}-en.mp3", OUTPUT_FILE_BASE_PATH, file_prefix);
    get_mp3(response_text.clone(), org_mp3_file_path.clone(), window).await?;

    marge_mp3_files(&[
            "../silence_1sec.mp3",
            &org_mp3_file_path
        ], &mp3_file_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    Ok((response_text, mp3_file_path))
}

async fn exec_chatgpt(system_context: &str, user_context: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("exec_chatgpt");

    /* 
    window.emit("add_to_output", Some(format!("
# ChatGPT
SystemContent: {}
UserContent: {}", system_context, user_context))).expect("failed to emit event");
    */
    println!("# ChatGPT
SystemContent: {}
UserContent: {}", system_context, user_context);

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
        println!(
            "{}: Role: {}  Content: {:?}",
            choice.index, choice.message.role, choice.message.content
        );
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

async fn save_text(title: &str, japanese: &str, english: &str, file_prefix: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("save_text");

    window.emit("add_to_output", Some("
# SaveText")).expect("failed to emit event");

    let file_name = format!("{}{}.txt", OUTPUT_FILE_BASE_PATH, file_prefix);
    let text = format!("# Title
{}

# Japanese
{}

# English
{}", title, japanese, english);
    std::fs::write(file_name.clone(), text).expect("failed to write file");

    window.emit("add_to_output", Some(format!("FilePath: {}", file_name))).expect("failed to emit event");

    Ok(file_name.to_string())
}

fn marge_mp3_files(files: &[&str], output_file: &str) -> std::io::Result<()> {
    // リストファイルを作成
    let mut list_file = File::create(MARGE_MP3_LIST_FILE_PATH)?;
    for &mp3_file in files {
        writeln!(list_file, "file '{}'", mp3_file)?;
    }

    // FFmpeg コマンドを実行してファイルを結合
    let status = Command::new("ffmpeg")
        .args(&["-y", "-f", "concat", "-safe", "0", "-i", MARGE_MP3_LIST_FILE_PATH, "-c", "copy", output_file])
        .status()?;

    if status.success() {
        println!("MP3 files were concatenated successfully.");
    } else {
        eprintln!("Failed to concatenate MP3 files.");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to concatenate MP3 files"));
    }

    Ok(())
}

fn play_mp3(file_path: &str) -> Result<(), Box<dyn Error>> {
    println!("play_mp3");

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let file = BufReader::new(File::open(file_path)?);
    let source = Decoder::new(file)?.convert_samples::<f32>();

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

fn ensure_directory_exists() {
    std::fs::create_dir_all(OUTPUT_FILE_BASE_PATH).expect("failed to create directory");
}

#[tokio::main]
async fn main() {
    ensure_directory_exists();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![submit_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
