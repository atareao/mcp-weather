<div align="center">

<img src="https://raw.githubusercontent.com/rust-mcp-stack/rust-mcp-sdk/main/assets/rust-mcp-icon.png" alt="mcp-weather icon" align="center" height="96" />

# mcp-weather

*MCP server for weather data — no API keys needed*

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP SDK](https://img.shields.io/badge/MCP_SDK-0.9-blue?style=flat-square)](https://crates.io/crates/rust-mcp-sdk)
[![Docker](https://img.shields.io/badge/Docker-ready-2496ED?style=flat-square&logo=docker)](https://hub.docker.com)
[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)

[Overview](#overview) · [Features](#features) · [Getting Started](#getting-started) · [Tools](#mcp-tools) · [Docker](#docker) · [Configuration](#configuration)

</div>

## Overview

**mcp-weather** is an [MCP (Model Context Protocol)](https://modelcontextprotocol.io) server that provides weather and geocoding tools to AI assistants. Built in Rust, it exposes four read-only tools over a streamable HTTP endpoint.

All data comes from free public APIs — no API key or account required:

- **[Open-Meteo](https://open-meteo.com)** for current weather, daily forecasts, and hourly forecasts
- **[Nominatim](https://nominatim.org)** (OpenStreetMap) for city geocoding

> [!TIP]
> Use this server with any MCP-compatible client (Claude Desktop, VS Code Copilot Chat, etc.) to give your assistant real-time weather capabilities.

## Features

- **Current weather** — temperature, wind speed, humidity, rainfall, and weather description
- **Daily forecast** — up to 16 days with min/max temperature, precipitation, wind, and probability
- **Hourly forecast** — up to 48 hours of temperature, humidity, wind, and precipitation data
- **City geocoding** — resolve any city name to latitude/longitude coordinates
- **Streamable HTTP** — MCP transport over HTTP/1.1 for easy integration
- **Structured logging** — `tracing`-based logs to stderr, configurable via `RUST_LOG`

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs) 1.70 or later
- _or_ [Docker](https://docs.docker.com/engine/install/) / [Podman](https://podman.io) for containerized deployment

### Build from source

```bash
git clone https://github.com/rust-mcp-stack/rust-mcp-sdk.git
cd mcp-weather
cargo build --release
```

The binary will be at `target/release/mcp-weather`.

### Run the server

```bash
RUST_LOG=mcp_weather=debug ./target/release/mcp-weather
```

The server starts on `http://0.0.0.0:3001`.

### Run tests

```bash
cargo test
```

Integration tests make live API calls to Open-Meteo for Valencia, Madrid, and London. They validate response structure and value ranges without specific assertions on weather values.

## MCP Tools

| Tool | Description |
|---|---|
| `get_coordinates` | Resolve a city name to latitude/longitude via Nominatim |
| `get_weather` | Current weather: temperature, wind, humidity, rain, and description |
| `get_forecast` | Daily forecast for 1–16 days (default: 7) |
| `get_hourly_forecast` | Hourly forecast for 1–48 hours (default: 24) |

### Usage examples

**`get_coordinates`**
```json
{ "city": "Tokyo" }
```
→ `Coordenadas de Tokyo: latitud=35.682839, longitud=139.759455`

**`get_weather`**
```json
{ "city": "Barcelona" }
```
→ `Clima en Barcelona: Temperatura: 22.5°C | Viento: 3.2 m/s | Humedad: 65% | Lluvia: 0mm | Condición: Cielo despejado`

**`get_forecast`**
```json
{ "city": "Berlin", "days": 5 }
```
→ 5-day forecast with daily min/max, precipitation probability, wind speed, and weather condition.

**`get_hourly_forecast`**
```json
{ "city": "Paris", "hours": 12 }
```
→ 12-hour forecast with per-hour temperature, humidity, wind, precipitation, and probability.

## Docker

A multi-stage Dockerfile is included for containerized deployment.

```bash
# Build and run with Docker
docker build -t mcp-weather .
docker run -p 3001:3001 mcp-weather

# Or use the justfile with Podman
just build
podman run -p 3001:3001 atareao/mcp-weather
```

A [Podman Quadlet](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) unit file is provided at `mcp-weather.container`.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `RUST_LOG` | `mcp_weather=debug` | Log level filter. Set to `mcp_weather=info,warn` for quieter output |

The server listens on `0.0.0.0:3001` by default. To change this, modify the `HyperServerOptions` in `src/main.rs:119` or set the port mapping when using containers.

## Architecture

```
┌──────────────────┐     ┌─────────────────┐     ┌──────────────┐
│  MCP Client       │────▶│  mcp-weather     │────▶│  Open-Meteo   │
│  (Claude, etc.)   │◀────│  (hyper-server)  │◀────│  (weather)    │
└──────────────────┘     └────────┬────────┘     └──────────────┘
                                  │
                                  │     ┌──────────────┐
                                  └────▶│  Nominatim   │
                                        │  (geocoding) │
                                        └──────────────┘
```

- `src/main.rs` — server entry point, MCP handler, and initialization
- `src/tools.rs` — MCP tool definitions using `#[mcp_tool]` macro
- `src/weather.rs` — Open-Meteo API client (current, daily, hourly)
- `src/coordinates.rs` — Nominatim geocoding client
- `src/lib.rs` — library entry for integration tests

## Resources

- [MCP specification](https://modelcontextprotocol.io)
- [rust-mcp-sdk](https://crates.io/crates/rust-mcp-sdk)
- [Open-Meteo API docs](https://open-meteo.com/en/docs)
- [Nominatim API docs](https://nominatim.org/release-docs/develop/api/Search/)
