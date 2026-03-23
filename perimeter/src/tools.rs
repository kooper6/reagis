use serde_json::{json, Value};

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
                        "path": { "type": "string", "description": "The relative path to the file" }
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
                        "path": { "type": "string", "description": "The relative path to create" },
                        "content": { "type": "string", "description": "The text to write into the file" }
                    },
                    "required": ["path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Execute a command securely within the sandbox. The command will be subject to resource limits (time, memory, output size).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "program": { "type": "string", "description": "The executable to run (e.g., 'python3', 'ls', 'cat')." },
                        "args": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of arguments to pass to the program."
                        }
                    },
                    "required": ["program", "args"]
                }
            }
        }
    ])
}