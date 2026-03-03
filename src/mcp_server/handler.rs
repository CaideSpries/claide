//! JSON-RPC 2.0 request handler for MCP server.
//!
//! Dispatches `initialize`, `tools/list`, and `tools/call` methods.

use serde_json::{json, Value};
use tracing::debug;

use crate::kernel::ZeptoKernel;
use crate::tools::mcp::protocol::{
    CallToolResult, ContentBlock, ListToolsResult, McpError, McpResponse, McpTool,
};
use crate::tools::ToolContext;

/// Server info returned during initialization.
const SERVER_NAME: &str = "zeptoclaw";
const PROTOCOL_VERSION: &str = "2024-11-05";

/// Handle a parsed JSON-RPC 2.0 request and produce a response.
///
/// Dispatches to the appropriate handler based on the `method` field:
/// - `initialize` — return server info and capabilities
/// - `notifications/initialized` — acknowledge (no response needed, but we return success)
/// - `tools/list` — list all registered tools from the kernel
/// - `tools/call` — execute a tool and return the result
/// - anything else — return a -32601 "Method not found" error
pub async fn handle_request(
    kernel: &ZeptoKernel,
    id: Option<u64>,
    method: &str,
    params: Option<Value>,
) -> McpResponse {
    debug!(method = method, id = ?id, "MCP server handling request");

    match method {
        "initialize" => handle_initialize(id),
        "notifications/initialized" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        },
        "tools/list" => handle_tools_list(kernel, id),
        "tools/call" => handle_tools_call(kernel, id, params).await,
        _ => error_response(id, -32601, &format!("Method not found: {}", method)),
    }
}

/// Handle `initialize` — return server info and capabilities (tools only).
fn handle_initialize(id: Option<u64>) -> McpResponse {
    let result = json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION")
        }
    });

    McpResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

/// Handle `tools/list` — map kernel tool definitions to MCP tool list.
fn handle_tools_list(kernel: &ZeptoKernel, id: Option<u64>) -> McpResponse {
    let defs = kernel.tool_definitions();
    let mcp_tools: Vec<McpTool> = defs
        .into_iter()
        .map(|def| McpTool {
            name: def.name,
            description: Some(def.description),
            input_schema: def.parameters,
        })
        .collect();

    let result = ListToolsResult { tools: mcp_tools };

    McpResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::to_value(result).unwrap_or(json!({}))),
        error: None,
    }
}

/// Handle `tools/call` — execute a tool via the kernel and return the result.
async fn handle_tools_call(
    kernel: &ZeptoKernel,
    id: Option<u64>,
    params: Option<Value>,
) -> McpResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return error_response(id, -32602, "Missing params for tools/call");
        }
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(name) => name.to_string(),
        None => {
            return error_response(id, -32602, "Missing 'name' in tools/call params");
        }
    };

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let ctx = ToolContext::default();
    let safety = kernel.safety.as_ref();

    let output = crate::kernel::execute_tool(
        &kernel.tools,
        &tool_name,
        arguments,
        &ctx,
        safety,
        &kernel.metrics,
        kernel.taint.as_ref(),
    )
    .await;

    match output {
        Ok(tool_output) => {
            let call_result = CallToolResult {
                content: vec![ContentBlock::Text {
                    text: tool_output.for_llm,
                }],
                is_error: tool_output.is_error,
            };
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::to_value(call_result).unwrap_or(json!({}))),
                error: None,
            }
        }
        Err(e) => {
            let call_result = CallToolResult {
                content: vec![ContentBlock::Text {
                    text: format!("Tool execution error: {}", e),
                }],
                is_error: true,
            };
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::to_value(call_result).unwrap_or(json!({}))),
                error: None,
            }
        }
    }
}

/// Build a JSON-RPC 2.0 error response.
fn error_response(id: Option<u64>, code: i64, message: &str) -> McpResponse {
    McpResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(McpError {
            code,
            message: message.to_string(),
            data: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::HookEngine;
    use crate::kernel::ZeptoKernel;
    use crate::safety::SafetyLayer;
    use crate::tools::{EchoTool, ToolRegistry};
    use crate::utils::metrics::MetricsCollector;
    use std::sync::Arc;

    /// Build a minimal kernel with just an EchoTool for testing.
    fn test_kernel() -> ZeptoKernel {
        let config = crate::config::Config::default();
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(EchoTool));

        ZeptoKernel {
            config: Arc::new(config.clone()),
            provider: None,
            tools,
            safety: if config.safety.enabled {
                Some(SafetyLayer::new(config.safety.clone()))
            } else {
                None
            },
            metrics: Arc::new(MetricsCollector::new()),
            hooks: Arc::new(HookEngine::new(config.hooks.clone())),
            mcp_clients: vec![],
            ltm: None,
            taint: None,
        }
    }

    #[tokio::test]
    async fn test_handle_initialize_returns_server_info() {
        let kernel = test_kernel();
        let resp = handle_request(&kernel, Some(1), "initialize", None).await;

        assert!(resp.error.is_none());
        let result = resp.result.expect("should have result");
        assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(result["serverInfo"]["name"], SERVER_NAME);
        assert!(result["capabilities"]["tools"].is_object());
    }

    #[tokio::test]
    async fn test_handle_tools_list_returns_kernel_tools() {
        let kernel = test_kernel();
        let resp = handle_request(&kernel, Some(2), "tools/list", None).await;

        assert!(resp.error.is_none());
        let result = resp.result.expect("should have result");
        let tools_result: ListToolsResult = serde_json::from_value(result).expect("should parse");
        assert_eq!(tools_result.tools.len(), 1);
        assert_eq!(tools_result.tools[0].name, "echo");
        assert_eq!(
            tools_result.tools[0].description,
            Some("Echoes back the provided message".to_string())
        );
        assert!(tools_result.tools[0].input_schema.is_object());
    }

    #[tokio::test]
    async fn test_handle_tools_call_executes_echo() {
        let kernel = test_kernel();
        let params = json!({
            "name": "echo",
            "arguments": { "message": "hello from MCP" }
        });
        let resp = handle_request(&kernel, Some(3), "tools/call", Some(params)).await;

        assert!(resp.error.is_none());
        let result = resp.result.expect("should have result");
        let call_result: CallToolResult = serde_json::from_value(result).expect("should parse");
        assert!(!call_result.is_error);
        assert_eq!(call_result.content.len(), 1);
        assert_eq!(call_result.content[0].as_text(), Some("hello from MCP"));
    }

    #[tokio::test]
    async fn test_handle_unknown_method_returns_error() {
        let kernel = test_kernel();
        let resp = handle_request(&kernel, Some(4), "unknown/method", None).await;

        assert!(resp.result.is_none());
        let err = resp.error.expect("should have error");
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_unknown_tool() {
        let kernel = test_kernel();
        let params = json!({
            "name": "nonexistent_tool",
            "arguments": {}
        });
        let resp = handle_request(&kernel, Some(5), "tools/call", Some(params)).await;

        // Tool not found is returned as a successful response with is_error=true
        assert!(resp.error.is_none());
        let result = resp.result.expect("should have result");
        let call_result: CallToolResult = serde_json::from_value(result).expect("should parse");
        assert!(call_result.is_error);
        assert!(call_result.content[0]
            .as_text()
            .unwrap()
            .contains("Tool not found"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_params() {
        let kernel = test_kernel();
        let resp = handle_request(&kernel, Some(6), "tools/call", None).await;

        assert!(resp.result.is_none());
        let err = resp.error.expect("should have error");
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("Missing params"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_name() {
        let kernel = test_kernel();
        let params = json!({ "arguments": {} });
        let resp = handle_request(&kernel, Some(7), "tools/call", Some(params)).await;

        assert!(resp.result.is_none());
        let err = resp.error.expect("should have error");
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("Missing 'name'"));
    }

    #[tokio::test]
    async fn test_mcp_tool_mapping_from_tool_definition() {
        let kernel = test_kernel();
        let defs = kernel.tool_definitions();

        let mcp_tools: Vec<McpTool> = defs
            .into_iter()
            .map(|def| McpTool {
                name: def.name,
                description: Some(def.description),
                input_schema: def.parameters,
            })
            .collect();

        assert_eq!(mcp_tools.len(), 1);
        let tool = &mcp_tools[0];
        assert_eq!(tool.name, "echo");
        assert_eq!(
            tool.description,
            Some("Echoes back the provided message".to_string())
        );
        // Verify the schema structure
        assert_eq!(tool.input_schema["type"], "object");
        assert!(tool.input_schema["properties"]["message"].is_object());
    }

    #[tokio::test]
    async fn test_call_tool_result_content_structure() {
        let kernel = test_kernel();
        let params = json!({
            "name": "echo",
            "arguments": { "message": "structured test" }
        });
        let resp = handle_request(&kernel, Some(8), "tools/call", Some(params)).await;

        let result = resp.result.expect("should have result");
        let call_result: CallToolResult = serde_json::from_value(result).expect("should parse");

        // Verify the content block structure
        assert_eq!(call_result.content.len(), 1);
        match &call_result.content[0] {
            ContentBlock::Text { text } => {
                assert_eq!(text, "structured test");
            }
            _ => panic!("Expected Text content block"),
        }
        assert!(!call_result.is_error);
    }

    #[tokio::test]
    async fn test_handle_initialize_id_preserved() {
        let kernel = test_kernel();
        let resp = handle_request(&kernel, Some(42), "initialize", None).await;
        assert_eq!(resp.id, Some(42));
    }

    #[tokio::test]
    async fn test_handle_notifications_initialized() {
        let kernel = test_kernel();
        let resp = handle_request(&kernel, Some(2), "notifications/initialized", None).await;
        assert!(resp.error.is_none());
        assert!(resp.result.is_some());
    }
}
