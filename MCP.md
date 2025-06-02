# MCP (Model Context Protocol) Support

RustyRoad now includes MCP support to enable Language Learning Models (LLMs) to interact with the framework programmatically.

## What is MCP?

The Model Context Protocol (MCP) is an open standard that enables secure connections between LLM applications and external data sources and tools. It allows LLMs to:

- Access and manipulate data sources
- Execute tools and functions
- Maintain context across interactions

## RustyRoad MCP Server

The RustyRoad MCP server exposes the framework's core functionality through a standardized interface that LLMs can use.

### Starting the MCP Server

```bash
rustyroad mcp
```

This starts the MCP server which communicates over stdio using JSON-RPC.

### Available Tools

The MCP server provides the following tools:

#### 1. `create_project`
Create a new RustyRoad project.

**Parameters:**
- `name` (required): Name of the project
- `database_type` (optional): Type of database ("postgres", "mysql", "sqlite", "mongodb", "none")
- `database_name` (optional): Name of the database
- `database_username` (optional): Database username
- `database_password` (optional): Database password
- `database_host` (optional): Database host
- `database_port` (optional): Database port

#### 2. `generate_controller`
Generate a new controller.

**Parameters:**
- `name` (required): Name of the controller/model
- `type` (optional): Type of controller ("read", "create", "update", "delete")

#### 3. `generate_model`
Generate a new model.

**Parameters:**
- `name` (required): Name of the model

#### 4. `generate_migration`
Generate a new migration.

**Parameters:**
- `name` (required): Name of the migration
- `columns` (optional): Array of column definitions in `name:type[:constraints]` format

#### 5. `add_feature`
Add a feature to the project.

**Parameters:**
- `feature` (required): Feature to add ("grapesjs")

### Available Resources

#### 1. `project://info`
Get information about the current RustyRoad project, including:
- Current directory
- Whether it's a RustyRoad project
- Project configuration

### Usage with LLMs

LLMs can connect to the RustyRoad MCP server to:

1. **Create new web applications**: Generate complete RustyRoad projects with database setup
2. **Generate code**: Create controllers, models, and migrations based on requirements
3. **Add features**: Integrate additional functionality like GrapesJS
4. **Inspect projects**: Get information about existing RustyRoad projects

### Example MCP Communication

The server communicates using JSON-RPC over stdio. Here's an example of creating a new project:

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_project",
    "arguments": {
      "name": "my_blog",
      "database_type": "postgres",
      "database_name": "my_blog_db"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"success\": true, \"message\": \"Project 'my_blog' created successfully\", \"project_path\": \"/path/to/my_blog\"}"
      }
    ]
  }
}
```

## Benefits for LLM Integration

With MCP support, LLMs can:

- **Understand RustyRoad projects**: Access project structure and configuration
- **Generate code intelligently**: Create appropriate controllers and models based on context
- **Follow best practices**: Use RustyRoad's conventions and patterns
- **Handle complex workflows**: Chain multiple operations together
- **Provide interactive assistance**: Help developers build applications step-by-step

This makes RustyRoad more accessible and enables AI-assisted development workflows.