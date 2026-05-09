//! Definición de herramientas MCP

use crate::coordinates;
use crate::weather;
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};

// ── get_coordinates ───────────────────────────────────────────────────────────

#[mcp_tool(
    name = "get_coordinates",
    description = "Obtiene las coordenadas geográficas (latitud y longitud) de una ciudad usando Nominatim",
    title = "Obtener coordenadas",
    read_only_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetCoordinatesTool {
    /// Nombre de la ciudad a buscar (ej: \"Madrid\", \"Buenos Aires\", \"Tokyo\")
    pub city: String,
}

impl GetCoordinatesTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let coords = coordinates::get_coordinates(&self.city)
            .await
            .map_err(|e| CallToolError::from_message(e))?;

        let text = format!(
            "Coordenadas de {}: latitud={:.6}, longitud={:.6}",
            coords.city, coords.latitude, coords.longitude
        );
        Ok(CallToolResult::text_content(vec![TextContent::from(text)]))
    }
}

// ── get_weather ───────────────────────────────────────────────────────────────

#[mcp_tool(
    name = "get_weather",
    description = "Obtiene la información meteorológica actual de una ciudad usando Open-Meteo. Incluye temperatura, viento, humedad, lluvia y descripción del clima.",
    title = "Obtener clima",
    read_only_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetWeatherTool {
    /// Nombre de la ciudad (ej: \"Barcelona\", \"Londres\", \"New York\")
    pub city: String,
}

impl GetWeatherTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let coords = coordinates::get_coordinates(&self.city)
            .await
            .map_err(|e| {
                CallToolError::from_message(format!("Error obteniendo coordenadas: {e}"))
            })?;

        let w = weather::fetch_weather(coords.latitude, coords.longitude, &coords.city)
            .await
            .map_err(|e| CallToolError::from_message(format!("Error obteniendo clima: {e}")))?;

        let text = format!(
            "Clima en {}:\n  Temperatura: {:.1}°C\n  Viento: {:.1} m/s\n  Humedad: {}%\n  Lluvia: {}mm\n  Condición: {}",
            w.city, w.temperature, w.wind_speed, w.humidity, w.rain, w.weather_description,
        );
        Ok(CallToolResult::text_content(vec![TextContent::from(text)]))
    }
}

// ── get_forecast ──────────────────────────────────────────────────────────────

#[mcp_tool(
    name = "get_forecast",
    description = "Obtiene la previsión meteorológica para los próximos días de una ciudad usando Open-Meteo. Incluye temperatura máxima/mínima, viento, precipitación y descripción del clima.",
    title = "Obtener previsión",
    read_only_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetForecastTool {
    /// Nombre de la ciudad (ej: \"Madrid\", \"Paris\", \"Berlin\")
    pub city: String,
    /// Número de días de previsión (entre 1 y 16). Por defecto 7 si no se especifica.
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 {
    7
}

impl GetForecastTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let coords = coordinates::get_coordinates(&self.city)
            .await
            .map_err(|e| {
                CallToolError::from_message(format!("Error obteniendo coordenadas: {e}"))
            })?;

        let forecast =
            weather::fetch_forecast(coords.latitude, coords.longitude, &coords.city, self.days)
                .await
                .map_err(|e| {
                    CallToolError::from_message(format!("Error obteniendo previsión: {e}"))
                })?;

        let mut lines = vec![format!(
            "Previsión meteorológica para {}{} ({} días):",
            forecast.city,
            if forecast.city != self.city {
                format!(" ({})", self.city)
            } else {
                String::new()
            },
            forecast.days.len(),
        )];
        for day in &forecast.days {
            let emoji = match day.weather_description.as_str() {
                "Cielo despejado" => "☀️",
                "Parcialmente nublado" => "⛅",
                "Niebla" => "🌫️",
                "Llovizna" | "Lluvia" | "Lluvia engelante" | "Llovizna engelante" => "🌧️",
                "Nevada" | "Nieve granulada" | "Chubascos de nieve" => "🌨️",
                "Chubascos" => "🌦️",
                "Tormenta" | "Tormenta con granizo" => "⛈️",
                _ => "❓",
            };
            lines.push(format!(
                "  {} {}: {:.0}°C / {:.0}°C | {:.1}mm | {}m/s | prob. {}%%",
                day.date,
                emoji,
                day.temp_max,
                day.temp_min,
                day.precipitation,
                day.wind_speed_max,
                day.precipitation_probability
            ));
        }
        Ok(CallToolResult::text_content(vec![TextContent::from(
            lines.join("\n"),
        )]))
    }
}

// ── get_hourly_forecast ─────────────────────────────────────────────────────

#[mcp_tool(
    name = "get_hourly_forecast",
    description = "Obtiene la previsión meteorológica para las próximas horas de una ciudad usando Open-Meteo. Incluye temperatura, humedad, viento, precipitación y descripción del clima hora a hora.",
    title = "Obtener previsión horaria",
    read_only_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetHourlyForecastTool {
    /// Nombre de la ciudad (ej: \"Madrid\", \"Paris\", \"Berlin\")
    pub city: String,
    /// Número de horas de previsión (entre 1 y 48). Por defecto 24 si no se especifica.
    #[serde(default = "default_hours")]
    pub hours: u32,
}

fn default_hours() -> u32 {
    24
}

impl GetHourlyForecastTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let coords = coordinates::get_coordinates(&self.city)
            .await
            .map_err(|e| {
                CallToolError::from_message(format!("Error obteniendo coordenadas: {e}"))
            })?;

        let forecast = weather::fetch_hourly_forecast(
            coords.latitude,
            coords.longitude,
            &coords.city,
            self.hours,
        )
        .await
        .map_err(|e| {
            CallToolError::from_message(format!("Error obteniendo previsión horaria: {e}"))
        })?;

        let mut lines = vec![format!(
            "Previsión horaria para {}{} ({} horas):",
            forecast.city,
            if forecast.city != self.city {
                format!(" ({})", self.city)
            } else {
                String::new()
            },
            forecast.slots.len(),
        )];
        for slot in &forecast.slots {
            let emoji = match slot.weather_description.as_str() {
                "Cielo despejado" => "☀️",
                "Parcialmente nublado" => "⛅",
                "Niebla" => "🌫️",
                "Llovizna" | "Lluvia" | "Lluvia engelante" | "Llovizna engelante" => "🌧️",
                "Nevada" | "Nieve granulada" | "Chubascos de nieve" => "🌨️",
                "Chubascos" => "🌦️",
                "Tormenta" | "Tormenta con granizo" => "⛈️",
                _ => "❓",
            };
            lines.push(format!(
                "  {} {}: {:.1}°C | {}% hum | {:.1} m/s | {:.1}mm | prob. {}%%",
                slot.time,
                emoji,
                slot.temperature,
                slot.humidity,
                slot.wind_speed,
                slot.precipitation,
                slot.precipitation_probability
            ));
        }
        Ok(CallToolResult::text_content(vec![TextContent::from(
            lines.join("\n"),
        )]))
    }
}

// ── Toolbox ───────────────────────────────────────────────────────────────────

rust_mcp_sdk::tool_box!(
    WeatherTools,
    [
        GetCoordinatesTool,
        GetWeatherTool,
        GetForecastTool,
        GetHourlyForecastTool
    ]
);

#[cfg(test)]
mod tests {
    use super::{default_days, default_hours};

    #[test]
    fn test_default_days_is_7() {
        assert_eq!(default_days(), 7);
    }

    #[test]
    fn test_default_hours_is_24() {
        assert_eq!(default_hours(), 24);
    }
}
