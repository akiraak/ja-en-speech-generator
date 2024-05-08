// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//use std::error::Error;
//use std::thread;
//use tauri::{Window};
//use tokio::runtime::Runtime;
 
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        CreateSpeechRequestArgs, SpeechModel, Voice,
    },
    Client,
};

use std::error::Error;
use std::thread;
use tauri::{Window};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

//use async_openai::{Client, types::*};


#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

// init a background process on the command, and emit periodic events only to the window that used the command
/* 
#[tauri::command]
fn init_process(window: Window) -> String {
    std::thread::spawn(move || {
        loop {
            //window.emit("event-name", Payload { message: "Tauri is awesome!".into() }).unwrap();
            window.emit("add_to_output", Some("init_process")).expect("failed to emit event");
            // sleep for 1 second
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    "init_process".to_string()
}*/


/* 
#[tauri::command]
fn submit_command(command: &str, window: Window) -> String {
    println!("submit_command: {}", command);
    let command_owned = command.to_string();

    // 別スレッドで非同期関数を実行
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        let system_context = "あなたは生徒に質問に答える教師です。２００文字以内で答えてください。".to_string();
        //let user_context = "コンピュータの歴史について教えてください。".to_string();
        let user_context = command_owned;

        match rt.block_on(exec_chatgpt(system_context, user_context, window.clone())) {
            Ok(japanese_text) => {
                println!("japanese_text: {}", japanese_text);
                match rt.block_on(get_mp3(japanese_text.clone(), "japanese".to_string(), window.clone())) {
                    Ok(file_path) => {
                        println!("File: {}", file_path);
                    }
                    Err(e) => println!("Error: {}", e),
                }

                let system_context = "あなたは日本語を英語に翻訳するアシスタントです。".to_string();
                match rt.block_on(exec_chatgpt(system_context, japanese_text.clone(), window.clone())) {
                    Ok(english_text) => {
                        println!("english_text: {}", english_text);
                        match rt.block_on(get_mp3(english_text, "english".to_string(), window.clone())) {
                            Ok(file_path) => {
                                println!("File: {}", file_path);
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    });

    format!("exec: {}", command)
}*/

/*
#[tauri::command]
fn submit_command(command: &str, window: Window) -> String {
    println!("submit_command: {}", command);
    let command_owned = command.to_string();

    thread::spawn(move || {
        println!("thread");
        execute_workflow(command_owned, window)
    });

    /*
    let handle = thread::spawn(move || {
        println!("thread");
        let r = execute_workflow(command_owned, window);
        // rをprintする
        match r.await {
            Ok(_) => println!("Ok"),
            Err(e) => println!("Error: {}", e),
        }
        println!("thread end");
    });
    
    // この行を追加
    let _ = handle.join();
    */

    println!("Command end");
    format!("Command execution started for: {}", command)
}
*/

#[tauri::command]
fn init_process(command: &str, window: Window) -> String {
    tokio::spawn(async move {
        window.emit("add_to_output", Some("init_process_sub")).expect("failed to emit event");
    });
    "init_process".to_string()
}

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
    //let runtime = Runtime::new()?;

    /* 
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            println!("Failed to create Tokio runtime: {}", e);
            return Ok(());
        }
    };
    //let client = Client::new();

    let japanese_text = runtime.block_on(make_japanese(&command, &window))?;
    runtime.block_on(make_english(&japanese_text, &window))?;
    */
    // make_japaneseを呼び出す
    let japanese_text = make_japanese(&command, &window).await?;
    // make_englishを呼び出す
    make_english(&japanese_text, &window).await?;

    Ok(())
}

async fn make_japanese(input: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("make_japanese");
    /*
    let system_context = match language {
        "japanese" => "あなたは生徒に質問に答える教師です。２００文字以内で答えてください。",
        "english" => "あなたは日本語を英語に翻訳するアシスタントです。",
        _ => unreachable!(),
    };*/
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

/* 
async fn exec_chatgpt(system_context: String, user_context: String, window: Window) -> Result<String, Box<dyn Error>> {
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
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_context)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_context)
                .build()?
                .into(),
        ])
        .build()?;

    println!("request: {}", serde_json::to_string(&request).unwrap());

    let response = client.chat().create(request).await?;
    //println!("\nResponse:\n");
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
    Err("No content found in the first choice".into())
}*/

async fn exec_chatgpt(system_context: &str, user_context: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("exec_chatgpt");
    /* 
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512)
        .model("gpt-4-turbo")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default().content(system_context).build()?.info(),
            ChatCompletionRequestUserMessageArgs::default().content(user_context).build()?.info(),
        ])
        .build()?;
    */

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

    //let response = client.chat().create(request).await?;
    //response.choices.first().map_or(Err("No content found".into()), |choice| Ok(choice.message.content.clone()))
    let response = client.chat().create(request).await?;
    //println!("\nResponse:\n");
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
    // window.emitでエラーを表示する
    window.emit("add_to_output", Some("ChatGPTの呼び出してエラーになりました。".to_string())).expect("failed to emit event");
    Err("ChatGPTの呼び出してエラーになりました。".into())
}

/* 
async fn get_mp3(text: String, file_prefix: String, window: Window) -> Result<String, Box<dyn Error>> {
    let client = Client::new();

    window.emit("add_to_output", Some("
# TTS")).expect("failed to emit event");

    let request = CreateSpeechRequestArgs::default()
        .input(text)
        .voice(Voice::Nova)
        //.model(SpeechModel::Tts1)
        .model(SpeechModel::Tts1Hd)
        .build()?;

    let response = client.audio().speech(request).await?;
    let file_path = format!("./{}.mp3", file_prefix);
    response.save(file_path.clone()).await?;

    window.emit("add_to_output", Some(format!("FilePath: {}", file_path))).expect("failed to emit event");

    Ok(file_path.to_string())
}*/

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

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![submit_command, init_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
