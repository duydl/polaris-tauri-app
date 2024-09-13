#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use tauri::Manager;
use tauri::http::{Response, Request};
use reqwest::Client;
use std::sync::Mutex;

use tauri::command;

use serde_json::json;

use tauri::State;


use log::info;
use simplelog::{
    ColorChoice, CombinedLogger, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs;
use std::path::{Path, PathBuf};

mod app;
mod db;
mod options;
mod paths;
mod service;
#[cfg(test)]
mod test;
// mod ui;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error(transparent)]
	App(#[from] app::Error),
	#[error("Could not parse command line arguments:\n\n{0}")]
	CliArgsParsing(getopts::Fail),
	#[error("Could not create log directory `{0}`:\n\n{1}")]
	LogDirectoryCreationError(PathBuf, std::io::Error),
	#[error("Could not create log file `{0}`:\n\n{1}")]
	LogFileCreationError(PathBuf, std::io::Error),
	#[error("Could not initialize log system:\n\n{0}")]
	LogInitialization(log::SetLoggerError),
}

fn init_logging<T: AsRef<Path>>(
	log_level: LevelFilter,
	log_file_path: &Option<T>,
) -> Result<(), Error> {
	let log_config = simplelog::ConfigBuilder::new()
		.set_location_level(LevelFilter::Error)
		.build();

	let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
		log_level,
		log_config.clone(),
		TerminalMode::Mixed,
		ColorChoice::Auto,
	)];

	if let Some(path) = log_file_path {
		if let Some(parent) = path.as_ref().parent() {
			fs::create_dir_all(parent)
				.map_err(|e| Error::LogDirectoryCreationError(parent.to_owned(), e))?;
		}
		loggers.push(WriteLogger::new(
			log_level,
			log_config,
			fs::File::create(path)
				.map_err(|e| Error::LogFileCreationError(path.as_ref().to_owned(), e))?,
		));
	}

	CombinedLogger::init(loggers).map_err(Error::LogInitialization)?;

	Ok(())
}

struct AppState{
    server_url: Mutex<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            server_url: Mutex::new(String::from("http://localhost:5050")),
        }
    }
}

#[command]
fn set_server_url(url: String, state: State<'_, AppState>) -> Result<(), String> {
    *state.server_url.lock().unwrap() = url;
    Ok(())
}

#[command]
async fn fetch_audio_file(path: String, state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let client = Client::new();
    let url = {
        let server_url = state.server_url.lock().unwrap();
        format!("{}/api{}", *server_url, path)
    };
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
async fn proxy_api_request(path: String, options: serde_json::Value, state: State<'_, AppState>) -> Result<String, String> {
    let client = Client::new();
    let url = {
        let server_url = state.server_url.lock().unwrap();
        format!("{}/api{}", *server_url, path)
    };
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

fn main() -> Result<(), Error> {

    let args: Vec<String> = std::env::args().collect();
    let options_manager = options::Manager::new();
    let cli_options = options_manager
        .parse(&args[1..])
        .map_err(Error::CliArgsParsing)?;

    if cli_options.show_help {
        let program = args[0].clone();
        let brief = format!("Usage: {} [options]", program);
        print!("{}", options_manager.usage(&brief));
        return Ok(());
    }

    let paths = paths::Paths::new(&cli_options);

    // Logging
    let log_level = cli_options.log_level.unwrap_or(LevelFilter::Info);
    init_logging(log_level, &paths.log_file_path)?;


    info!("Cache files location is {:#?}", paths.cache_dir_path);
    info!("Config files location is {:#?}", paths.config_file_path);
    info!("Database file location is {:#?}", paths.db_file_path);
    info!("Log file location is {:#?}", paths.log_file_path);

    info!("Swagger files location is {:#?}", paths.swagger_dir_path);
    info!("Web client files location is {:#?}", paths.web_dir_path);

    // Create and run app
    let app = app::App::new(cli_options.port.unwrap_or(5050), paths)?;
    app.index.begin_periodic_updates();
    app.ddns_manager.begin_periodic_updates();

    // Start server
    info!("Starting up server");
    std::thread::spawn(move || {
        let _ = service::run(app);
    });

    // Run UI
    // ui::run();


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
      .manage(AppState::default())
      .invoke_handler(tauri::generate_handler![proxy_api_request, fetch_audio_file, set_server_url])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");

      info!("Shutting down server");
      Ok(())
 }
