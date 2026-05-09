//! MCP Weather Server
//!
//! Servidor MCP que proporciona herramientas para obtener información meteorológica
//! usando Open-Meteo.com (datos del clima) y Nominatim.org (geocodificación).
//!
//! Herramientas:
//! - `get_coordinates`: Obtiene latitud/longitud de una ciudad
//! - `get_weather`: Obtiene el clima actual de una ciudad
//! - `get_forecast`: Obtiene la previsión meteorológica de los próximos días
//! - `get_hourly_forecast`: Obtiene la previsión meteorológica hora a hora

mod coordinates;
mod tools;
mod weather;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

use async_trait::async_trait;
use rust_mcp_sdk::{
    event_store::InMemoryEventStore,
    mcp_icon,
    mcp_server::{hyper_server, HyperServerOptions, ServerHandler},
    schema::{
        schema_utils::CallToolError, CallToolRequestParams, CallToolResult, Implementation,
        InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, RpcError,
        ServerCapabilities, ServerCapabilitiesTools,
    },
    McpServer, ToMcpServerHandler,
};
use tools::WeatherTools;

// ── MCP Server Handler ────────────────────────────────────────────────────────

pub struct WeatherServerHandler;

#[async_trait]
impl ServerHandler for WeatherServerHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: WeatherTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool: WeatherTools = WeatherTools::try_from(params).map_err(CallToolError::new)?;

        match tool {
            WeatherTools::GetCoordinatesTool(t) => t.call_tool().await,
            WeatherTools::GetWeatherTool(t) => t.call_tool().await,
            WeatherTools::GetForecastTool(t) => t.call_tool().await,
            WeatherTools::GetHourlyForecastTool(t) => t.call_tool().await,
        }
    }
}

// ── Entry Point ───────────────────────────────────────────────────────────────

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("mcp_weather=debug".parse().unwrap()),
        )
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    eprintln!("MCP Weather Server v0.1 starting...");
    info!("Arrancando MCP Weather Server v0.1");

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "mcp-weather".into(),
            version: "0.1.0".into(),
            title: Some("MCP Weather Server".into()),
            description: Some(
                "Servidor MCP para información meteorológica usando Open-Meteo y Nominatim"
                    .into(),
            ),
            icons: vec![mcp_icon!(
                src = "https://raw.githubusercontent.com/rust-mcp-stack/rust-mcp-sdk/main/assets/rust-mcp-icon.png",
                mime_type = "image/png",
                sizes = ["128x128"],
                theme = "dark"
            )],
            website_url: Some("https://github.com/rust-mcp-stack/rust-mcp-sdk".into()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            "Usa get_coordinates para buscar coordenadas de una ciudad. Usa get_weather para obtener el clima actual. Usa get_forecast para la previsión de los próximos días. Usa get_hourly_forecast para la previsión hora a hora.".into(),
        ),
        protocol_version: ProtocolVersion::V2025_11_25.into(),
    };

    let handler = WeatherServerHandler.to_mcp_server_handler();

    let server = hyper_server::create_server(
        server_details,
        handler,
        HyperServerOptions {
            host: "0.0.0.0".to_string(),
            port: 3001,
            event_store: Some(Arc::new(InMemoryEventStore::default())),
            ..Default::default()
        },
    );

    info!("MCP Weather Server escuchando en http://0.0.0.0:3001");

    if let Err(e) = server.start().await {
        eprintln!(
            "Error al iniciar: {}",
            e.rpc_error_message().unwrap_or(&e.to_string())
        );
        return;
    }

    // Mantener el servidor vivo hasta SIGTERM/SIGINT
    tokio::signal::ctrl_c().await.ok();
    info!("Apagando MCP Weather Server");
}
