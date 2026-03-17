mod tools;

use guardian::ReagisGuard;
use std::path::Path;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};
use serde_json::Value;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let sandbox_path = "./citadel_workspace";
    let guard = ReagisGuard::new(sandbox_path)?;
    let ollama = Ollama::default();

    let user_input = "Create a file named mission.txt with 'Secure the Citadel' as content.";
    let mut message = vec![
        ChatMessage::user(user_input.to_string()),
    ];

    loop {
        let res = ollama
            .chat_with_tools("llama3", message.clone(), Some(tools::get_fs_tools))
            .await()?;

        let response_message = res.message;

        if let Some(tool_calls) = &response_message.tools_calls {
            for call in tool_calls {
                let func_name = &call.function.name;
                let args = &call.function.arguments;

                println!("Guardian Intercept: Agent wants to call {}", func_name);

                let result = match func_name.as_str() {
                    "write_file" => {
                        let path = args["path"].as_str().unwrap_or_default();
                        let content = args["content"].as_str().unwrap_or_default();
                        guard.write_file(path, content)
                    },
                    "read_file" => {
                        let path = args["path"].as_str().unwrap_or_default();
                        guard.read_file(path)
                    }
                    _ => Err("Unknown tool".to_string())
                };

                let status = result.unwrap_or_else(|e| e);
                message.push(ChatMessage::tool(status))
            }
            continue;
        } else {
            println!("Citadel Response: {}", response_message.content);
            break;
        }

    }

    Ok(())
}
