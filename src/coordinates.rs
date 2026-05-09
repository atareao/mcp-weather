//! Geocodificación de ciudades usando Nominatim
//!
//! <https://nominatim.org/release-docs/develop/api/Search/>

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;

/// Datos de coordenadas geográficas
#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinates {
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
}

fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("MCP-Weather/0.1")
        .build()
        .expect("Failed to create HTTP client")
}

fn url_encode(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('"', "%22")
        .replace('<', "%3C")
        .replace('>', "%3E")
}

/// Obtiene coordenadas de una ciudad usando Nominatim
pub async fn get_coordinates(city: &str) -> Result<Coordinates, String> {
    let client = http_client();
    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1&bounded=1",
        url_encode(city)
    );

    info!("Nominatim: buscando '{}'", city);

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Error conectando con Nominatim: {e}"))?;

    let status = resp.status();

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Nominatim error {}: {}", status, body));
    }

    let data: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| format!("Error parseando JSON: {e}"))?;

    if data.is_empty() {
        return Err(format!("Ciudad '{}' no encontrada", city));
    }

    let item = &data[0];
    let lat = item["lat"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or("latitud no encontrada")?;
    let lon = item["lon"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .ok_or("longitud no encontrada")?;
    let city_name = item["display_name"].as_str().unwrap_or(city).to_string();

    info!("Nominatim: {} -> ({:.4}, {:.4})", city_name, lat, lon);

    Ok(Coordinates {
        city: city_name,
        latitude: lat,
        longitude: lon,
    })
}
