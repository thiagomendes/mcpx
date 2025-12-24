//! MCP Client Service
//! 
//! Provides MCP SDK client functionality for communicating with upstream servers.
//! Used by gateway proxy to aggregate tools from multiple servers.

use rmcp::{
    ServiceExt,
    model::{ClientCapabilities, ClientInfo, Implementation, Tool},
    transport::streamable_http_client::{StreamableHttpClientTransport, StreamableHttpClientTransportConfig},
    service::RunningService,
};

#[derive(Debug, Clone)]
pub struct McpToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl From<Tool> for McpToolInfo {
    fn from(tool: Tool) -> Self {
        Self {
            name: tool.name.to_string(),
            description: tool.description.map(|d| d.to_string()),
            input_schema: serde_json::to_value(&tool.input_schema).unwrap_or_default(),
        }
    }
}

pub async fn list_tools_from_server(server_url: &str, auth_header_name: Option<&str>, auth_header_value: Option<&str>) -> Result<Vec<McpToolInfo>, String> {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    
    // Build custom headers if auth is provided
    let mut custom_headers = HeaderMap::new();
    if let (Some(name), Some(value)) = (auth_header_name, auth_header_value) {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|e| format!("Invalid header name: {:?}", e))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|e| format!("Invalid header value: {:?}", e))?;
        custom_headers.insert(header_name, header_value);
    }

    // Build client with custom headers
    let client = if custom_headers.is_empty() {
        reqwest::Client::new()
    } else {
        reqwest::Client::builder()
            .default_headers(custom_headers)
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {:?}", e))?
    };

    let config = StreamableHttpClientTransportConfig::with_uri(server_url);
    let transport = StreamableHttpClientTransport::with_client(client, config);

    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "mcpx-gateway".to_string(),
            title: None,
            version: "1.0.0".to_string(),
            website_url: None,
            icons: None,
        },
    };

    let mcp_client: RunningService<rmcp::RoleClient, _> = client_info.serve(transport).await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    let tools_result = mcp_client.list_tools(None).await
        .map_err(|e| format!("Failed to list tools: {:?}", e))?;

    let _ = mcp_client.cancel().await;

    Ok(tools_result.tools.into_iter().map(McpToolInfo::from).collect())
}
