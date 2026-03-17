use serde_json::{json, Value};

pub fn get_fs_tools() -> Value {
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
        }
    ])
}