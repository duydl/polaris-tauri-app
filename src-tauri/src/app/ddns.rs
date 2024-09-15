use base64::prelude::*;
use log::{debug, error};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time;

const DDNS_UPDATE_URL: &str = "https://ydns.io/api/v1/update/";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("DDNS update query failed with HTTP status code `{0}`")]
    UpdateQueryFailed(u16),
    #[error("DDNS update query failed due to a transport error")]
    UpdateQueryTransport,
    #[error("Database error")]
    DatabaseError,
    #[error("Could not acquire database connection")]
    ConnectionError(#[from] rusqlite::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    pub host: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct Manager {
    db_path: String, // Path to the SQLite database
}

impl Manager {
    pub fn new(db_path: &str) -> Self {
        Self { db_path: db_path.to_string() }
    }

    fn connect(&self) -> SqlResult<Connection> {
        Connection::open(&self.db_path)
    }

    fn update_my_ip(&self) -> Result<(), Error> {
        let config = self.config()?;
        if config.host.is_empty() || config.username.is_empty() {
            debug!("Skipping DDNS update because credentials are missing");
            return Ok(());
        }

        let full_url = format!("{}?host={}", DDNS_UPDATE_URL, &config.host);
        let credentials = format!("{}:{}", &config.username, &config.password);
        let response = ureq::get(full_url.as_str())
            .set(
                "Authorization",
                &format!("Basic {}", BASE64_STANDARD_NO_PAD.encode(credentials)),
            )
            .call();

        match response {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(code, _)) => Err(Error::UpdateQueryFailed(code)),
            Err(ureq::Error::Transport(_)) => Err(Error::UpdateQueryTransport),
        }
    }

    // Fetch the DDNS config from the SQLite database
    pub fn config(&self) -> Result<Config, Error> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT host, username, password FROM ddns_config LIMIT 1")?;
        let mut config_iter = stmt.query_map([], |row| {
            Ok(Config {
                host: row.get(0)?,
                username: row.get(1)?,
                password: row.get(2)?,
            })
        })?;

        if let Some(config) = config_iter.next() {
            return config.map_err(|_| Error::DatabaseError);
        }

        Err(Error::DatabaseError)
    }

    // Update the DDNS config in the SQLite database
    pub fn set_config(&self, new_config: &Config) -> Result<(), Error> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE ddns_config SET host = ?, username = ?, password = ?",
            params![new_config.host, new_config.username, new_config.password],
        )?;
        Ok(())
    }

    pub fn begin_periodic_updates(&self) {
        let cloned = self.clone();
        std::thread::spawn(move || {
            cloned.run();
        });
    }

    fn run(&self) {
        loop {
            if let Err(e) = self.update_my_ip() {
                error!("Dynamic DNS update error: {:?}", e);
            }
            thread::sleep(time::Duration::from_secs(60 * 30));
        }
    }
}
