pub mod cli;
pub mod session;
pub mod tools;

use guardian::ReagisGuard;
use reagis_core::CommandRunner;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::{ChatMessage, request::ChatMessageRequest};
use std::io::{self, Write};
use colored::*;
use indicatif::ProgressBar;

pub async fn run_agent(session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sandbox_path = "./citadel_workspace";
    std::fs::create_dir_all(sandbox_path)?;

    let guard = ReagisGuard::new(sandbox_path)?;
    let ollama = Ollama::default();

    // --- Ollama Health Check ---
    if ollama.list_local_models().await.is_err() {
        println!("{}", "Ollama is not running. Starting...".yellow());
        let _ = std::process::Command::new("ollama")
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Waiting for Ollama...");
        let mut connected = false;
        for _ in 0..15 {
            if ollama.list_local_models().await.is_ok() { connected = true; break; }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        spinner.finish_and_clear();
        if !connected { return Err("Ollama connection failed.".into()); }
    }

    let runner = CommandRunner::new(10, Some(512), 1024 * 1024);
    let mut session = session::Session::load(session_id)
        .unwrap_or_else(|| session::Session::new(session_id.to_string()));

    // --- Inject the Citadel Protocol (System Prompt) ---
    if session.history.is_empty() {
        session.history.push(ChatMessage::system(tools::get_citadel_system_prompt()));

        print!("{}", "Enter mission objective: ".blue().bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        session.history.push(ChatMessage::user(input.trim().to_string()));
    }

    loop {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Thinking...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let req = ChatMessageRequest::new("llama3".to_string(), session.history.clone());
        let res = ollama.send_chat_messages(req).await?;
        spinner.finish_and_clear();

        let response_content = res.message.content.clone();

        // --- THE JSON INTERCEPTOR (The Protocol) ---
        // We look for a JSON block in the response.
        // Llama 3 is very good at outputting just the JSON when instructed.
        if let Ok(tool_req) = serde_json::from_str::<serde_json::Value>(response_content.trim()) {
            let func_name = tool_req["tool"].as_str().unwrap_or("unknown");
            let args = &tool_req["parameters"];

            println!("{} {}", "Guardian Intercept:".red().bold(), func_name.cyan());

            let result = match func_name {
                "write_file" => {
                    let p = args["path"].as_str().unwrap_or_default();
                    let c = args["content"].as_str().unwrap_or_default();
                    guard.write_file(p, c)
                },
                "read_file" => {
                    let p = args["path"].as_str().unwrap_or_default();
                    guard.read_file(p)
                },
                "run_command" => {
                    let prog = args["program"].as_str().unwrap_or_default();
                    let args_vec: Vec<String> = args["args"].as_array()
                        .map(|a| a.iter().map(|v| v.as_str().unwrap_or_default().to_string()).collect())
                        .unwrap_or_default();

                    match runner.run_command(prog, &args_vec).await {
                        Ok(r) => Ok(format!("STDOUT: {}\nCODE: {:?}", r.stdout, r.exit_code)),
                        Err(e) => Err(format!("Exec Error: {}", e))
                    }
                },
                _ => Err("Unknown tool".to_string())
            };

            let output = result.unwrap_or_else(|e| e);

            // Record the "thought" and the "observation"
            session.history.push(ChatMessage::assistant(response_content));
            session.history.push(ChatMessage::user(format!("OBSERVATION: {}", output)));
            session.save()?;
            continue;
        }

        // --- Standard Conversation ---
        if !response_content.is_empty() {
            println!("{} {}", "Citadel:".green().bold(), response_content);
            session.history.push(res.message);
            session.save()?;
        }

        print!("{}", "> ".blue().bold());
        io::stdout().flush()?;
        let mut user_input = String::new();
        if io::stdin().read_line(&mut user_input)? == 0 { break; }
        let trimmed = user_input.trim();
        if trimmed.eq_ignore_ascii_case("exit") { break; }

        session.history.push(ChatMessage::user(trimmed.to_string()));
        session.save()?;
    }
    Ok(())
}