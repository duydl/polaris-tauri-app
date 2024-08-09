
use tauri::Manager;
use tauri::http::{Response, Request};
use reqwest::Client;
use std::sync::Arc;

use tauri::command;

use serde_json::json;

#[command]
async fn fetch_audio_file(path: String) -> Result<Vec<u8>, String> {
    let client = Client::new();
    let url = format!("http://localhost:5050/api{}", path);
    println!("URL Audio: {}",url);
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes().await {
                    Ok(bytes) => Ok(bytes.to_vec()),
                    Err(err) => Err(format!("Error reading bytes: {}", err)),
                }
            } else {
                Err(format!("Failed to fetch audio file, status: {}", response.status()))
            }
        }
        Err(err) => Err(format!("Request error: {}", err)),
    }
}

#[command]
async fn proxy_api_request(path: String, options: serde_json::Value) -> Result<String, String> {
    let client = Client::new();
    let url = format!("http://localhost:5050/api{}", path);
    println!("URL: {}",url);
    let mut request_builder = client.request(
        options["method"].as_str().unwrap_or("GET").parse().unwrap(),
        &url
    );

    if let Some(headers) = options["headers"].as_object() {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value.as_str().unwrap_or(""));
        }
    }

    if let Some(body) = options["body"].as_str() {
        request_builder = request_builder.body(body.to_string());
    }

    match request_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let text = response.text().await.map_err(|e| e.to_string())?;
            println!("Response: {} {}", status, text);
            // Create the ApiResponse structure and convert it to a JSON string
            let api_response = json!({
                "status": status,
                "body": text,
            });

            Ok(api_response.to_string())
        }
        Err(err) => {
            println!("Request Error: {}", err);
            Err(err.to_string())
        },
    }
}

fn main() {
    tauri::Builder::default()
      .setup(|app| {
        // #[cfg(debug_assertions)]
        {
            let window = app.get_window("main").unwrap();
            // window.open_devtools();
            // window.close_devtools();
        }
        Ok(())
      })
      .invoke_handler(tauri::generate_handler![proxy_api_request, fetch_audio_file])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
 }
