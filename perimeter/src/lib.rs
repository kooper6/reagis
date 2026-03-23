pub mod cli;
pub mod session;
pub mod tools;

use guardian::ReagisGuard;
use reagis_core::CommandRunner;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use serde_json::Value;
use std::io::{self, Write};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn run_agent(session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let sandbox_path = "./citadel_workspace";
    let guard = ReagisGuard::new(sandbox_path)?;
    let ollama = Ollama::default();

    // Command Runner: 10s timeout, 512MB RAM limit, 1MB max output
    let runner = CommandRunner::new(10, Some(512), 1024 * 1024);

    let mut session = session::Session::load(session_id).unwrap_or_else(|| session::Session::new(session_id.to_string()));

    if session.history.is_empty() {
        print!("{}", "Enter your mission objective: ".blue().bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
             println!("Mission aborted.");
             return Ok(());
        }
        session.history.push(ChatMessage::user(input.to_string()));
        session.save()?;
    } else {
        println!("{}", "Resuming session...".yellow());
        if let Some(last) = session.history.last() {
             println!("Last message ({:?}): {:.50}...", last.role, last.content);
        }
    }

    loop {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::default_spinner()
            .tick_chars("/|\\- ")
            .template("{spinner:.green} {msg}")
            .unwrap());
        spinner.set_message("Thinking...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let res = ollama
            .chat_with_tools("llama3", session.history.clone(), Some(tools::get_citadel_tools))
            .await?;

        spinner.finish_and_clear();

        let response_message = res.message;

        if let Some(tool_calls) = &response_message.tool_calls {
            session.history.push(response_message.clone());

            for call in tool_calls {
                let func_name = &call.function.name;
                let args = &call.function.arguments;

                println!("{} {}", "Guardian Intercept: Agent wants to call".red().bold(), func_name.cyan());

                let result = match func_name.as_str() {
                    "write_file" => {
                        let path = args["path"].as_str().unwrap_or_default();
                        let content = args["content"].as_str().unwrap_or_default();
                        guard.write_file(path, content)
                    },
                    "read_file" => {
                        let path = args["path"].as_str().unwrap_or_default();
                        guard.read_file(path)
                    },
                    "run_command" => {
                        let program = args["program"].as_str().unwrap_or_default();
                        // Extract args correctly
                        let args_vec = if let Some(arr) = args["args"].as_array() {
                             arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect::<Vec<String>>()
                        } else {
                            vec![]
                        };

                        let run_res = runner.run_command(program, &args_vec).await;
                        match run_res {
                            Ok(res) => Ok(format!("Stdout: {}\nStderr: {}\nExit Code: {:?}", res.stdout, res.stderr, res.exit_code)),
                            Err(e) => Err(format!("Execution failed: {}", e))
                        }
                    },
                    _ => Err("Unknown tool".to_string())
                };

                let status = result.unwrap_or_else(|e| e);
                session.history.push(ChatMessage::tool(status, call.id.clone()));
            }
            session.save()?;
            continue;
        } else {
            let content = response_message.content.clone();
            println!("{} {}", "Citadel:".green().bold(), content);
            session.history.push(ChatMessage::assistant(content));
            session.save()?;

            print!("{}", "> ".blue().bold());
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                println!("Session saved. Exiting.");
                break;
            }

            session.history.push(ChatMessage::user(input.to_string()));
            session.save()?;
        }
    }

    Ok(())
}