//! Información meteorológica usando Open-Meteo
//!
//! <https://open-meteo.com/en/docs>

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;

/// Datos meteorológicos
#[derive(Debug, Serialize, Deserialize)]
pub struct Weather {
    pub city: String,
    pub temperature: f64,
    pub wind_speed: f64,
    pub humidity: u32,
    pub rain: u32,
    pub weather_description: String,
}

fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("MCP-Weather/0.1")
        .build()
        .expect("Failed to create HTTP client")
}

/// Convierte código WMO en descripción legible
fn decode_weather_code(code: u32) -> &'static str {
    match code {
        0 => "Cielo despejado",
        1..=3 => "Parcialmente nublado",
        45 | 48 => "Niebla",
        51 | 53 | 55 => "Llovizna",
        56 | 57 => "Llovizna engelante",
        61 | 63 | 65 => "Lluvia",
        66 | 67 => "Lluvia engelante",
        71 | 73 | 75 => "Nevada",
        77 => "Nieve granulada",
        80 | 81 | 82 => "Chubascos",
        85 | 86 => "Chubascos de nieve",
        95 => "Tormenta",
        96 | 99 => "Tormenta con granizo",
        _ => "Desconocido",
    }
}

/// Previsión diaria
#[derive(Debug, Serialize, Deserialize)]
pub struct ForecastDay {
    pub date: String,
    pub temp_max: f64,
    pub temp_min: f64,
    pub weather_description: String,
    pub precipitation: f64,
    pub precipitation_probability: u32,
    pub wind_speed_max: f64,
}

/// Previsión meteorológica para varios días
#[derive(Debug, Serialize, Deserialize)]
pub struct Forecast {
    pub city: String,
    pub days: Vec<ForecastDay>,
}

/// Obtiene datos meteorológicos de Open-Meteo para unas coordenadas dadas
pub async fn fetch_weather(lat: f64, lon: f64, city: &str) -> Result<Weather, String> {
    let client = http_client();
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,wind_speed_10m,rain,weather_code&timezone=auto&format=json",
        lat, lon
    );

    info!("Open-Meteo: consultando ({}, {})", lat, lon);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Error conectando con Open-Meteo: {e}"))?;

    let status = resp.status();

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Open-Meteo error {}: {}", status, body));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error parseando JSON Open-Meteo: {e}"))?;

    let current = data
        .get("current")
        .ok_or("Respuesta sin campo 'current'".to_string())?;

    let temperature = current["temperature_2m"].as_f64().unwrap_or(0.0);
    let wind_speed = current["wind_speed_10m"].as_f64().unwrap_or(0.0);
    let humidity = current["relative_humidity_2m"].as_u64().unwrap_or(0) as u32;
    let rain = current["rain"].as_u64().unwrap_or(0) as u32;
    let weather_code = current["weather_code"].as_u64().unwrap_or(0) as u32;
    let description = decode_weather_code(weather_code);

    info!(
        "Open-Meteo: {} -> {:.1}°C, viento {:.1}m/s, hum {}%, lluvia {}mm, {}",
        city, temperature, wind_speed, humidity, rain, description
    );

    Ok(Weather {
        city: city.to_string(),
        temperature,
        wind_speed,
        humidity,
        rain,
        weather_description: description.to_string(),
    })
}

/// Parsea la sección `daily` de la respuesta JSON de Open-Meteo
fn parse_forecast_daily(daily: &serde_json::Value, city: &str) -> Result<Forecast, String> {
    let times: Vec<&str> = daily["time"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    if times.is_empty() {
        return Err("No hay datos de previsión disponibles".to_string());
    }

    let mut forecast_days = Vec::with_capacity(times.len());

    for (i, date_str) in times.iter().enumerate() {
        let temp_max = daily["temperature_2m_max"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let temp_min = daily["temperature_2m_min"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let weather_code = daily["weather_code"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let precipitation = daily["precipitation_sum"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let precipitation_probability = daily["precipitation_probability_max"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let wind_speed_max = daily["wind_speed_10m_max"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        forecast_days.push(ForecastDay {
            date: date_str.to_string(),
            temp_max,
            temp_min,
            weather_description: decode_weather_code(weather_code).to_string(),
            precipitation,
            precipitation_probability,
            wind_speed_max,
        });
    }

    info!(
        "Open-Meteo previsión: {} -> {} días recuperados",
        city,
        forecast_days.len()
    );

    Ok(Forecast {
        city: city.to_string(),
        days: forecast_days,
    })
}

/// Obtiene la previsión meteorológica de los próximos días
pub async fn fetch_forecast(lat: f64, lon: f64, city: &str, days: u32) -> Result<Forecast, String> {
    let client = http_client();
    let days = days.clamp(1, 16);
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,weather_code,precipitation_sum,precipitation_probability_max,wind_speed_10m_max&forecast_days={}&timezone=auto&format=json",
        lat, lon, days
    );

    info!(
        "Open-Meteo: previsión para ({}, {}), {} días",
        lat, lon, days
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Error conectando con Open-Meteo: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Open-Meteo error {}: {}", status, body));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error parseando JSON Open-Meteo: {e}"))?;

    let daily = data
        .get("daily")
        .ok_or("Respuesta sin campo 'daily'".to_string())?;

    parse_forecast_daily(daily, city)
}

/// Previsión horaria — un slot por hora
#[derive(Debug, Serialize, Deserialize)]
pub struct HourlySlot {
    pub time: String,
    pub temperature: f64,
    pub humidity: u32,
    pub wind_speed: f64,
    pub weather_description: String,
    pub precipitation: f64,
    pub precipitation_probability: u32,
}

/// Previsión horaria para las próximas horas
#[derive(Debug, Serialize, Deserialize)]
pub struct HourlyForecast {
    pub city: String,
    pub slots: Vec<HourlySlot>,
}

/// Parsea la sección `hourly` de la respuesta JSON de Open-Meteo
fn parse_hourly(hourly: &serde_json::Value, city: &str) -> Result<HourlyForecast, String> {
    let times: Vec<&str> = hourly["time"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    if times.is_empty() {
        return Err("No hay datos de previsión horaria disponibles".to_string());
    }

    let mut slots = Vec::with_capacity(times.len());

    for (i, time_str) in times.iter().enumerate() {
        let temperature = hourly["temperature_2m"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let humidity = hourly["relative_humidity_2m"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let wind_speed = hourly["wind_speed_10m"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let weather_code = hourly["weather_code"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let precipitation = hourly["precipitation"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let precipitation_probability = hourly["precipitation_probability"]
            .as_array()
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        slots.push(HourlySlot {
            time: time_str.to_string(),
            temperature,
            humidity,
            wind_speed,
            weather_description: decode_weather_code(weather_code).to_string(),
            precipitation,
            precipitation_probability,
        });
    }

    info!(
        "Open-Meteo previsión horaria: {} -> {} slots recuperados",
        city,
        slots.len()
    );

    Ok(HourlyForecast {
        city: city.to_string(),
        slots,
    })
}

/// Obtiene la previsión meteorológica horaria de las próximas horas
pub async fn fetch_hourly_forecast(
    lat: f64,
    lon: f64,
    city: &str,
    hours: u32,
) -> Result<HourlyForecast, String> {
    let client = http_client();
    let hours = hours.clamp(1, 48);
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code,precipitation,precipitation_probability&forecast_hours={}&timezone=auto&format=json",
        lat, lon, hours
    );

    info!(
        "Open-Meteo: previsión horaria para ({}, {}), {} horas",
        lat, lon, hours
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Error conectando con Open-Meteo: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Open-Meteo error {}: {}", status, body));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error parseando JSON Open-Meteo: {e}"))?;

    let hourly = data
        .get("hourly")
        .ok_or("Respuesta sin campo 'hourly'".to_string())?;

    parse_hourly(hourly, city)
}

#[cfg(test)]
mod tests {
    use super::decode_weather_code;

    #[test]
    fn test_weather_codes_known() {
        assert_eq!(decode_weather_code(0), "Cielo despejado");
        assert_eq!(decode_weather_code(1), "Parcialmente nublado");
        assert_eq!(decode_weather_code(2), "Parcialmente nublado");
        assert_eq!(decode_weather_code(3), "Parcialmente nublado");
        assert_eq!(decode_weather_code(45), "Niebla");
        assert_eq!(decode_weather_code(48), "Niebla");
        assert_eq!(decode_weather_code(51), "Llovizna");
        assert_eq!(decode_weather_code(53), "Llovizna");
        assert_eq!(decode_weather_code(55), "Llovizna");
        assert_eq!(decode_weather_code(56), "Llovizna engelante");
        assert_eq!(decode_weather_code(57), "Llovizna engelante");
        assert_eq!(decode_weather_code(61), "Lluvia");
        assert_eq!(decode_weather_code(63), "Lluvia");
        assert_eq!(decode_weather_code(65), "Lluvia");
        assert_eq!(decode_weather_code(66), "Lluvia engelante");
        assert_eq!(decode_weather_code(67), "Lluvia engelante");
        assert_eq!(decode_weather_code(71), "Nevada");
        assert_eq!(decode_weather_code(73), "Nevada");
        assert_eq!(decode_weather_code(75), "Nevada");
        assert_eq!(decode_weather_code(77), "Nieve granulada");
        assert_eq!(decode_weather_code(80), "Chubascos");
        assert_eq!(decode_weather_code(81), "Chubascos");
        assert_eq!(decode_weather_code(82), "Chubascos");
        assert_eq!(decode_weather_code(85), "Chubascos de nieve");
        assert_eq!(decode_weather_code(86), "Chubascos de nieve");
        assert_eq!(decode_weather_code(95), "Tormenta");
        assert_eq!(decode_weather_code(96), "Tormenta con granizo");
        assert_eq!(decode_weather_code(99), "Tormenta con granizo");
    }

    #[test]
    fn test_weather_codes_unknown() {
        assert_eq!(decode_weather_code(4), "Desconocido");
        assert_eq!(decode_weather_code(100), "Desconocido");
        assert_eq!(decode_weather_code(255), "Desconocido");
        assert_eq!(decode_weather_code(999), "Desconocido");
    }

    #[test]
    fn test_weather_codes_discontinued() {
        // Old codes that should now be "Desconocido"
        assert_eq!(decode_weather_code(103), "Desconocido");
        assert_eq!(decode_weather_code(111), "Desconocido");
        assert_eq!(decode_weather_code(119), "Desconocido");
        assert_eq!(decode_weather_code(144), "Desconocido");
        assert_eq!(decode_weather_code(176), "Desconocido");
    }

    mod forecast {
        use super::super::parse_forecast_daily;

        fn sample_daily_json() -> serde_json::Value {
            serde_json::json!({
                "time": ["2026-05-10", "2026-05-11", "2026-05-12"],
                "temperature_2m_max": [22.5, 24.1, 20.3],
                "temperature_2m_min": [12.0, 13.5, 11.2],
                "weather_code": [0, 3, 61],
                "precipitation_sum": [0.0, 0.2, 5.4],
                "precipitation_probability_max": [0, 10, 85],
                "wind_speed_10m_max": [15.0, 12.3, 18.7]
            })
        }

        #[test]
        fn test_parse_forecast_basic() {
            let daily = sample_daily_json();
            let forecast = parse_forecast_daily(&daily, "Madrid").unwrap();
            assert_eq!(forecast.city, "Madrid");
            assert_eq!(forecast.days.len(), 3);
        }

        #[test]
        fn test_parse_forecast_day_fields() {
            let daily = sample_daily_json();
            let forecast = parse_forecast_daily(&daily, "Barcelona").unwrap();

            let day0 = &forecast.days[0];
            assert_eq!(day0.date, "2026-05-10");
            assert_eq!(day0.temp_max, 22.5);
            assert_eq!(day0.temp_min, 12.0);
            assert_eq!(day0.weather_description, "Cielo despejado");
            assert_eq!(day0.precipitation, 0.0);
            assert_eq!(day0.precipitation_probability, 0);
            assert_eq!(day0.wind_speed_max, 15.0);
        }

        #[test]
        fn test_parse_forecast_weather_codes() {
            let daily = sample_daily_json();
            let forecast = parse_forecast_daily(&daily, "Test").unwrap();
            assert_eq!(forecast.days[0].weather_description, "Cielo despejado"); // code 0
            assert_eq!(forecast.days[1].weather_description, "Parcialmente nublado"); // code 3
            assert_eq!(forecast.days[2].weather_description, "Lluvia"); // code 61
        }

        #[test]
        fn test_parse_forecast_last_day_values() {
            let daily = sample_daily_json();
            let forecast = parse_forecast_daily(&daily, "Test").unwrap();
            let last = &forecast.days[2];
            assert_eq!(last.temp_max, 20.3);
            assert_eq!(last.temp_min, 11.2);
            assert_eq!(last.precipitation, 5.4);
            assert_eq!(last.precipitation_probability, 85);
            assert_eq!(last.wind_speed_max, 18.7);
        }

        #[test]
        fn test_parse_forecast_empty_times() {
            let daily = serde_json::json!({ "time": [] });
            let result = parse_forecast_daily(&daily, "Test");
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_forecast_missing_times() {
            let daily = serde_json::json!({});
            let result = parse_forecast_daily(&daily, "Test");
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_forecast_handles_missing_fields() {
            let daily = serde_json::json!({
                "time": ["2026-05-10"],
                "temperature_2m_max": null
            });
            let forecast = parse_forecast_daily(&daily, "Test").unwrap();
            assert_eq!(forecast.days[0].temp_max, 0.0);
            assert_eq!(forecast.days[0].temp_min, 0.0);
            assert_eq!(forecast.days[0].wind_speed_max, 0.0);
            assert_eq!(forecast.days[0].weather_description, "Cielo despejado");
        }

        #[test]
        fn test_forecast_serde_roundtrip() {
            use crate::weather::Forecast;
            let daily = sample_daily_json();
            let forecast = parse_forecast_daily(&daily, "Sevilla").unwrap();
            let serialized = serde_json::to_string(&forecast).unwrap();
            let deserialized: Forecast = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized.city, forecast.city);
            assert_eq!(deserialized.days.len(), forecast.days.len());
            for (a, b) in forecast.days.iter().zip(deserialized.days.iter()) {
                assert_eq!(a.date, b.date);
                assert_eq!(a.temp_max, b.temp_max);
                assert_eq!(a.temp_min, b.temp_min);
                assert_eq!(a.weather_description, b.weather_description);
                assert_eq!(a.precipitation, b.precipitation);
                assert_eq!(a.precipitation_probability, b.precipitation_probability);
                assert_eq!(a.wind_speed_max, b.wind_speed_max);
            }
        }
    }

    mod hourly {
        use super::super::parse_hourly;

        fn sample_hourly_json() -> serde_json::Value {
            serde_json::json!({
                "time": ["2026-05-10T10:00", "2026-05-10T11:00", "2026-05-10T12:00"],
                "temperature_2m": [18.5, 19.2, 20.1],
                "relative_humidity_2m": [65, 60, 55],
                "wind_speed_10m": [10.0, 12.5, 14.0],
                "weather_code": [1, 3, 61],
                "precipitation": [0.0, 0.0, 2.5],
                "precipitation_probability": [0, 5, 70]
            })
        }

        #[test]
        fn test_parse_hourly_basic() {
            let hourly = sample_hourly_json();
            let forecast = parse_hourly(&hourly, "Madrid").unwrap();
            assert_eq!(forecast.city, "Madrid");
            assert_eq!(forecast.slots.len(), 3);
        }

        #[test]
        fn test_parse_hourly_slot_fields() {
            let hourly = sample_hourly_json();
            let forecast = parse_hourly(&hourly, "Barcelona").unwrap();

            let slot0 = &forecast.slots[0];
            assert_eq!(slot0.time, "2026-05-10T10:00");
            assert_eq!(slot0.temperature, 18.5);
            assert_eq!(slot0.humidity, 65);
            assert_eq!(slot0.wind_speed, 10.0);
            assert_eq!(slot0.weather_description, "Parcialmente nublado");
            assert_eq!(slot0.precipitation, 0.0);
            assert_eq!(slot0.precipitation_probability, 0);
        }

        #[test]
        fn test_parse_hourly_weather_codes() {
            let hourly = sample_hourly_json();
            let forecast = parse_hourly(&hourly, "Test").unwrap();
            assert_eq!(
                forecast.slots[0].weather_description,
                "Parcialmente nublado"
            );
            assert_eq!(
                forecast.slots[1].weather_description,
                "Parcialmente nublado"
            );
            assert_eq!(forecast.slots[2].weather_description, "Lluvia");
        }

        #[test]
        fn test_parse_hourly_last_slot_values() {
            let hourly = sample_hourly_json();
            let forecast = parse_hourly(&hourly, "Test").unwrap();
            let last = &forecast.slots[2];
            assert_eq!(last.temperature, 20.1);
            assert_eq!(last.humidity, 55);
            assert_eq!(last.wind_speed, 14.0);
            assert_eq!(last.precipitation, 2.5);
            assert_eq!(last.precipitation_probability, 70);
        }

        #[test]
        fn test_parse_hourly_empty_times() {
            let hourly = serde_json::json!({ "time": [] });
            let result = parse_hourly(&hourly, "Test");
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_hourly_missing_times() {
            let hourly = serde_json::json!({});
            let result = parse_hourly(&hourly, "Test");
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_hourly_handles_missing_fields() {
            let hourly = serde_json::json!({
                "time": ["2026-05-10T10:00"],
                "temperature_2m": null
            });
            let forecast = parse_hourly(&hourly, "Test").unwrap();
            assert_eq!(forecast.slots[0].temperature, 0.0);
            assert_eq!(forecast.slots[0].humidity, 0);
            assert_eq!(forecast.slots[0].wind_speed, 0.0);
            assert_eq!(forecast.slots[0].weather_description, "Cielo despejado");
            assert_eq!(forecast.slots[0].precipitation, 0.0);
            assert_eq!(forecast.slots[0].precipitation_probability, 0);
        }

        #[test]
        fn test_hourly_serde_roundtrip() {
            use super::super::HourlyForecast;
            let hourly = sample_hourly_json();
            let forecast = parse_hourly(&hourly, "Sevilla").unwrap();
            let serialized = serde_json::to_string(&forecast).unwrap();
            let deserialized: HourlyForecast = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized.city, forecast.city);
            assert_eq!(deserialized.slots.len(), forecast.slots.len());
            for (a, b) in forecast.slots.iter().zip(deserialized.slots.iter()) {
                assert_eq!(a.time, b.time);
                assert_eq!(a.temperature, b.temperature);
                assert_eq!(a.humidity, b.humidity);
                assert_eq!(a.wind_speed, b.wind_speed);
                assert_eq!(a.weather_description, b.weather_description);
                assert_eq!(a.precipitation, b.precipitation);
                assert_eq!(a.precipitation_probability, b.precipitation_probability);
            }
        }
    }
}
