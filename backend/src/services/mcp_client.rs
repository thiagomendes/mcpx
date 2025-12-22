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

pub async fn list_tools_from_server(server_url: &str, access_token: Option<&str>) -> Result<Vec<McpToolInfo>, String> {
    let config = if let Some(token) = access_token {
        StreamableHttpClientTransportConfig::with_uri(server_url)
            .auth_header(token)
    } else {
        StreamableHttpClientTransportConfig::with_uri(server_url)
    };

    let transport = StreamableHttpClientTransport::with_client(reqwest::Client::new(), config);

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

    let client: RunningService<rmcp::RoleClient, _> = client_info.serve(transport).await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    let tools_result = client.list_tools(None).await
        .map_err(|e| format!("Failed to list tools: {:?}", e))?;

    let _ = client.cancel().await;

    Ok(tools_result.tools.into_iter().map(McpToolInfo::from).collect())
}
