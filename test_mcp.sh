#!/bin/bash

# Simple test script to demonstrate MCP functionality
# This script shows how to interact with the RustyRoad MCP server

echo "=== RustyRoad MCP Server Test ==="
echo "This script demonstrates basic MCP protocol interaction"
echo ""

# Test 1: Initialize the MCP server
echo "Test 1: Initialize MCP server"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | timeout 5 rustyroad mcp 2>/dev/null | head -1
echo ""

# Test 2: List available tools
echo "Test 2: List available tools"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | timeout 5 rustyroad mcp 2>/dev/null | head -1
echo ""

# Test 3: List available resources  
echo "Test 3: List available resources"
echo '{"jsonrpc":"2.0","id":3,"method":"resources/list"}' | timeout 5 rustyroad mcp 2>/dev/null | head -1
echo ""

# Test 4: Read project info resource
echo "Test 4: Read project info"
echo '{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"project://info"}}' | timeout 5 rustyroad mcp 2>/dev/null | head -1
echo ""

echo "=== MCP Test Complete ==="
echo "Note: Tests may timeout if RustyRoad is not installed or if there are compilation issues"
echo "To test manually, run: rustyroad mcp"
echo "Then send JSON-RPC commands via stdin"