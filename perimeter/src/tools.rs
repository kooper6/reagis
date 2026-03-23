use serde_json::{json, Value};

pub fn get_citadel_system_prompt() -> String {
    r#"You are the Reagis Sentinel, a secure AI agent.
    You operate inside an isolated "Citadel" sandbox.

    To interact with the filesystem or run commands, you MUST respond with a raw JSON object.
    Do not include any conversational text when calling a tool.

    AVAILABLE TOOLS:
    1. {"tool": "write_file", "parameters": {"path": "script.py", "content": "print('hello')"}}
    2. {"tool": "read_file", "parameters": {"path": "data.txt"}}
    3. {"tool": "run_command", "parameters": {"program": "python3", "args": ["script.py"]}}

    After you receive an 'OBSERVATION' from the system, analyze it and continue or finish."#.to_string()
}

/// You can keep this for internal schema validation or future native tool support
pub fn get_citadel_tools() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read content from a file in the secure sandbox",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write content to a file in the secure sandbox",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Execute a command securely within the sandbox.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "program": { "type": "string" },
                        "args": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["program", "args"]
                }
            }
        }
    ])
}