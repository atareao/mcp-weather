//! Tests de integracion: consultas reales a Open-Meteo para Valencia, Madrid y Londres.

use mcp_weather::weather;

#[tokio::test]
async fn test_fetch_weather_valencia() {
    let lat = 39.4699;
    let lon = -0.3763;
    let result = weather::fetch_weather(lat, lon, "Valencia").await;
    assert!(result.is_ok(), "Valencia: {:?}", result.err());
    let w = result.unwrap();
    assert_eq!(w.city, "Valencia");
    assert!(
        w.temperature > -50.0 && w.temperature < 60.0,
        "Temperatura fuera de rango: {}",
        w.temperature
    );
    assert!(
        w.wind_speed >= 0.0 && w.wind_speed < 300.0,
        "Velocidad viento fuera de rango: {}",
        w.wind_speed
    );
    assert!(w.humidity <= 100, "Humedad > 100%: {}", w.humidity);
    assert!(!w.weather_description.is_empty(), "Descripcion vacia");
    println!(
        "Valencia: {:.1}°C, {}% hum, {:.1} km/h, {}mm, {}",
        w.temperature, w.humidity, w.wind_speed, w.rain, w.weather_description
    );
}

#[tokio::test]
async fn test_fetch_weather_madrid() {
    let lat = 40.4168;
    let lon = -3.7038;
    let result = weather::fetch_weather(lat, lon, "Madrid").await;
    assert!(result.is_ok(), "Madrid: {:?}", result.err());
    let w = result.unwrap();
    assert_eq!(w.city, "Madrid");
    assert!(
        w.temperature > -50.0 && w.temperature < 60.0,
        "Temperatura fuera de rango: {}",
        w.temperature
    );
    assert!(
        w.wind_speed >= 0.0 && w.wind_speed < 300.0,
        "Velocidad viento fuera de rango: {}",
        w.wind_speed
    );
    assert!(w.humidity <= 100, "Humedad > 100%: {}", w.humidity);
    assert!(!w.weather_description.is_empty(), "Descripcion vacia");
    println!(
        "Madrid: {:.1}°C, {}% hum, {:.1} km/h, {}mm, {}",
        w.temperature, w.humidity, w.wind_speed, w.rain, w.weather_description
    );
}

#[tokio::test]
async fn test_fetch_weather_londres() {
    let lat = 51.5074;
    let lon = -0.1278;
    let result = weather::fetch_weather(lat, lon, "Londres").await;
    assert!(result.is_ok(), "Londres: {:?}", result.err());
    let w = result.unwrap();
    assert_eq!(w.city, "Londres");
    assert!(
        w.temperature > -50.0 && w.temperature < 60.0,
        "Temperatura fuera de rango: {}",
        w.temperature
    );
    assert!(
        w.wind_speed >= 0.0 && w.wind_speed < 300.0,
        "Velocidad viento fuera de rango: {}",
        w.wind_speed
    );
    assert!(w.humidity <= 100, "Humedad > 100%: {}", w.humidity);
    assert!(!w.weather_description.is_empty(), "Descripcion vacia");
    println!(
        "Londres: {:.1}°C, {}% hum, {:.1} km/h, {}mm, {}",
        w.temperature, w.humidity, w.wind_speed, w.rain, w.weather_description
    );
}
