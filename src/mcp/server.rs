use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{Project, Database, DatabaseType, CRUDType};
use crate::features::add_feature;
use crate::writers::{create_new_controller, create_base_model};
use crate::database::migrations::create_migration;

#[derive(Serialize, Deserialize, Debug)]
struct McpRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct McpResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Serialize, Deserialize, Debug)]
struct McpError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ToolInfo {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResourceInfo {
    uri: String,
    name: String,
    description: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

pub struct RustyRoadMcpServer {
    current_directory: String,
}

impl RustyRoadMcpServer {
    pub fn new() -> Self {
        Self {
            current_directory: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }

    pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = Self::new();
        
        eprintln!("RustyRoad MCP Server starting...");
        
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        let mut stdout = io::stdout();
        
        while let Some(line) = reader.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<McpRequest>(&line) {
                Ok(request) => {
                    let response = server.handle_request(request).await;
                    let response_json = serde_json::to_string(&response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    eprintln!("Failed to parse request: {}", e);
                    let error_response = McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(McpError {
                            code: -32700,
                            message: "Parse error".to_string(),
                            data: Some(json!(e.to_string())),
                        }),
                    };
                    let response_json = serde_json::to_string(&error_response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
        }
        
        Ok(())
    }

    async fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            "resources/list" => self.handle_resources_list(request.id),
            "resources/read" => self.handle_resources_read(request.id, request.params).await,
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {}
                },
                "serverInfo": {
                    "name": "RustyRoad MCP Server",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        }
    }

    fn handle_tools_list(&self, id: Option<Value>) -> McpResponse {
        let tools = vec![
            ToolInfo {
                name: "create_project".to_string(),
                description: "Create a new RustyRoad project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the project"
                        },
                        "database_type": {
                            "type": "string",
                            "enum": ["postgres", "mysql", "sqlite", "mongodb", "none"],
                            "description": "Type of database to use"
                        },
                        "database_name": {
                            "type": "string",
                            "description": "Name of the database"
                        },
                        "database_username": {
                            "type": "string",
                            "description": "Database username"
                        },
                        "database_password": {
                            "type": "string",
                            "description": "Database password"
                        },
                        "database_host": {
                            "type": "string",
                            "description": "Database host"
                        },
                        "database_port": {
                            "type": "number",
                            "description": "Database port"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolInfo {
                name: "generate_controller".to_string(),
                description: "Generate a new controller".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the controller/model"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["read", "create", "update", "delete"],
                            "description": "Type of controller to generate"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolInfo {
                name: "generate_model".to_string(),
                description: "Generate a new model".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the model"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolInfo {
                name: "generate_migration".to_string(),
                description: "Generate a new migration".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the migration"
                        },
                        "columns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Column definitions in name:type[:constraints] format"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolInfo {
                name: "add_feature".to_string(),
                description: "Add a feature to the project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "feature": {
                            "type": "string",
                            "enum": ["grapesjs"],
                            "description": "Feature to add"
                        }
                    },
                    "required": ["feature"]
                }),
            },
        ];

        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": tools
            })),
            error: None,
        }
    }

    async fn handle_tools_call(&mut self, id: Option<Value>, params: Option<Value>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Tool name is required".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let tool_params = params.get("arguments").unwrap_or(&json!({}));

        let result = match tool_name {
            "create_project" => self.create_project_tool(tool_params).await,
            "generate_controller" => self.generate_controller_tool(tool_params).await,
            "generate_model" => self.generate_model_tool(tool_params).await,
            "generate_migration" => self.generate_migration_tool(tool_params).await,
            "add_feature" => self.add_feature_tool(tool_params).await,
            _ => Err("Unknown tool".to_string()),
        };

        match result {
            Ok(content) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&content).unwrap_or_else(|_| content.to_string())
                        }
                    ]
                })),
                error: None,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!(e)),
                }),
            },
        }
    }

    fn handle_resources_list(&self, id: Option<Value>) -> McpResponse {
        let resources = vec![
            ResourceInfo {
                uri: "project://info".to_string(),
                name: "Project Information".to_string(),
                description: "Information about the current RustyRoad project".to_string(),
                mime_type: "application/json".to_string(),
            },
        ];

        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "resources": resources
            })),
            error: None,
        }
    }

    async fn handle_resources_read(&self, id: Option<Value>, params: Option<Value>) -> McpResponse {
        let uri = match params.and_then(|p| p.get("uri")).and_then(|v| v.as_str()) {
            Some(uri) => uri,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "URI is required".to_string(),
                        data: None,
                    }),
                };
            }
        };

        match uri {
            "project://info" => {
                let project_info = self.get_project_info().await;
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "contents": [
                            {
                                "uri": "project://info",
                                "mimeType": "application/json",
                                "text": serde_json::to_string_pretty(&project_info).unwrap_or_default()
                            }
                        ]
                    })),
                    error: None,
                }
            }
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32602,
                    message: "Unknown resource URI".to_string(),
                    data: None,
                }),
            },
        }
    }

    async fn create_project_tool(&mut self, params: &Value) -> Result<Value, String> {
        let name = params["name"].as_str()
            .ok_or("Project name is required")?;
        let db_type = params.get("database_type").and_then(|v| v.as_str()).unwrap_or("none");
        let db_name = params.get("database_name").and_then(|v| v.as_str()).unwrap_or(name);
        let db_username = params.get("database_username").and_then(|v| v.as_str()).unwrap_or("postgres");
        let db_password = params.get("database_password").and_then(|v| v.as_str()).unwrap_or("");
        let db_host = params.get("database_host").and_then(|v| v.as_str()).unwrap_or("localhost");
        let db_port = params.get("database_port").and_then(|v| v.as_u64()).unwrap_or(5432) as u16;

        let database_type = match db_type {
            "postgres" | "postgresql" => DatabaseType::Postgres,
            "mysql" => DatabaseType::Mysql,
            "sqlite" => DatabaseType::Sqlite,
            "mongodb" | "mongo" => DatabaseType::Mongo,
            _ => DatabaseType::None,
        };

        let database = Database {
            database_type,
            name: db_name.to_string(),
            username: db_username.to_string(),
            password: db_password.to_string(),
            host: db_host.to_string(),
            port: db_port,
        };

        match Project::create_new_project(name.to_string(), database).await {
            Ok(project) => {
                self.current_directory = project.directory.clone();
                Ok(json!({
                    "success": true,
                    "message": format!("Project '{}' created successfully", name),
                    "project_path": project.directory
                }))
            }
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }

    async fn generate_controller_tool(&self, params: &Value) -> Result<Value, String> {
        let name = params["name"].as_str()
            .ok_or("Controller name is required")?;
        let controller_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("read");
        
        let crud_type = match controller_type.to_lowercase().as_str() {
            "get" | "read" => CRUDType::Read,
            "post" | "create" => CRUDType::Create,
            "put" | "update" => CRUDType::Update,
            "delete" => CRUDType::Delete,
            _ => CRUDType::Read,
        };
        
        match create_new_controller(name.to_string(), crud_type).await {
            Ok(_) => Ok(json!({
                "success": true,
                "message": format!("Controller '{}' generated successfully", name)
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }

    async fn generate_model_tool(&self, params: &Value) -> Result<Value, String> {
        let name = params["name"].as_str()
            .ok_or("Model name is required")?;
        
        match create_base_model(name).await {
            Ok(_) => Ok(json!({
                "success": true,
                "message": format!("Model '{}' generated successfully", name)
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }

    async fn generate_migration_tool(&self, params: &Value) -> Result<Value, String> {
        let name = params["name"].as_str()
            .ok_or("Migration name is required")?;
        let columns = params.get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();
        
        match create_migration(name, columns).await {
            Ok(_) => Ok(json!({
                "success": true,
                "message": format!("Migration '{}' generated successfully", name)
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }

    async fn add_feature_tool(&self, params: &Value) -> Result<Value, String> {
        let feature = params["feature"].as_str()
            .ok_or("Feature name is required")?;
        
        match add_feature(feature.to_string()).await {
            Ok(_) => Ok(json!({
                "success": true,
                "message": format!("Feature '{}' added successfully", feature)
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }

    async fn get_project_info(&self) -> Value {
        let project_path = Path::new(&self.current_directory);
        
        let mut info = HashMap::new();
        info.insert("current_directory".to_string(), self.current_directory.clone());
        
        // Check if it's a RustyRoad project by looking for common files
        let cargo_toml = project_path.join("Cargo.toml");
        let rustyroad_toml = project_path.join("rustyroad.toml");
        
        info.insert("is_rustyroad_project".to_string(), 
            rustyroad_toml.exists().to_string());
        info.insert("has_cargo_toml".to_string(), 
            cargo_toml.exists().to_string());
        
        // Try to read rustyroad.toml if it exists
        if rustyroad_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&rustyroad_toml) {
                info.insert("rustyroad_config".to_string(), content);
            }
        }
        
        json!(info)
    }
}