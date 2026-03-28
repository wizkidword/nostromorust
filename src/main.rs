use std::time::{Duration, Instant};

use chrono::Local;
use eframe::{egui, NativeOptions};
use serde::Deserialize;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([520.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Nostromo Dashboard",
        options,
        Box::new(|_cc| Ok(Box::new(DashboardApp::default()))),
    )
}

struct DashboardApp {
    weather: WeatherPod,
}

impl Default for DashboardApp {
    fn default() -> Self {
        Self {
            weather: WeatherPod::new("Philadelphia"),
        }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Continuously repaint so the clock stays current.
        ctx.request_repaint_after(Duration::from_secs(1));

        // Refresh weather every 10 minutes.
        if self.weather.last_refreshed.elapsed() >= Duration::from_secs(600) {
            self.weather.refresh();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nostromo Dashboard");
            ui.separator();

            ui.columns(2, |columns| {
                columns[0].group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Date & Time").strong());

                        let now = Local::now();
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(now.format("%A, %B %d, %Y").to_string()).size(20.0),
                        );
                        ui.label(
                            egui::RichText::new(now.format("%I:%M:%S %p").to_string()).size(28.0),
                        );
                    });
                });

                columns[1].group(|ui| {
                    self.weather.ui(ui);
                });
            });
        });
    }
}

struct WeatherPod {
    city: String,
    status: String,
    last_refreshed: Instant,
}

impl WeatherPod {
    fn new(city: &str) -> Self {
        let mut pod = Self {
            city: city.to_string(),
            status: "Loading...".to_string(),
            // Force immediate first refresh.
            last_refreshed: Instant::now() - Duration::from_secs(600),
        };
        pod.refresh();
        pod
    }

    fn refresh(&mut self) {
        self.last_refreshed = Instant::now();
        self.status = match fetch_weather_for_city(&self.city) {
            Ok(weather) => format!(
                "{}°F • {}\nWind: {:.1} mph\nUpdated: {}",
                weather.temperature_f,
                weather.description,
                weather.wind_speed_mph,
                Local::now().format("%I:%M %p")
            ),
            Err(error) => format!("Unable to load weather: {error}"),
        };
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Weather").strong());
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("City:");
                ui.text_edit_singleline(&mut self.city);
                if ui.button("Refresh").clicked() {
                    self.refresh();
                }
            });

            ui.add_space(8.0);
            ui.label(&self.status);
        });
    }
}

#[derive(Deserialize)]
struct GeocodeResponse {
    results: Option<Vec<GeocodeResult>>,
}

#[derive(Deserialize)]
struct GeocodeResult {
    latitude: f64,
    longitude: f64,
    name: String,
}

#[derive(Deserialize)]
struct ForecastResponse {
    current: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    weather_code: i32,
    wind_speed_10m: f64,
}

struct WeatherData {
    temperature_f: i32,
    wind_speed_mph: f64,
    description: String,
}

fn fetch_weather_for_city(city: &str) -> Result<WeatherData, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| format!("client error: {e}"))?;

    let geocode = client
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query(&[("name", city), ("count", "1")])
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("geocode request failed: {e}"))?
        .json::<GeocodeResponse>()
        .map_err(|e| format!("geocode parse failed: {e}"))?;

    let location = geocode
        .results
        .and_then(|mut results| results.pop())
        .ok_or_else(|| format!("no location found for '{city}'"))?;

    let forecast = client
        .get("https://api.open-meteo.com/v1/forecast")
        .query(&[
            ("latitude", location.latitude.to_string()),
            ("longitude", location.longitude.to_string()),
            (
                "current",
                "temperature_2m,weather_code,wind_speed_10m".to_string(),
            ),
            ("temperature_unit", "fahrenheit".to_string()),
            ("wind_speed_unit", "mph".to_string()),
        ])
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("forecast request failed: {e}"))?
        .json::<ForecastResponse>()
        .map_err(|e| format!("forecast parse failed: {e}"))?;

    Ok(WeatherData {
        temperature_f: forecast.current.temperature_2m.round() as i32,
        wind_speed_mph: forecast.current.wind_speed_10m,
        description: format!(
            "{} ({})",
            location.name,
            weather_code_to_label(forecast.current.weather_code)
        ),
    })
}

fn weather_code_to_label(code: i32) -> &'static str {
    match code {
        0 => "Clear",
        1 | 2 | 3 => "Partly cloudy",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing drizzle",
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing rain",
        71 | 73 | 75 => "Snow",
        77 => "Snow grains",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm + hail",
        _ => "Unknown",
    }
}
