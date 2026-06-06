use crate::util::debug_value;
use radiobrowser::{
    ApiStation,
    blocking::{RadioBrowserAPI, StationSearchBuilder},
};
use std::error::Error;
use util::debug;

mod mpv;
mod tui;
mod util;

pub use mpv::{check_app_native, play_url};
pub use tui::tui_main;

pub struct SearchOptions<'a> {
    pub station_name: Option<&'a str>,
    pub language: Option<&'a str>,
    pub country: Option<&'a str>,
    pub country_code: Option<&'a str>,
    pub state: Option<&'a str>,
    pub tag: Option<&'a str>,
    pub codec: Option<&'a str>,
    pub bitrate_min: Option<&'a u16>,
    pub bitrate_max: Option<&'a u16>,
}

pub struct RadioBrowserApp {
    radio_browser: RadioBrowserAPI,
    debug_mode: bool,
}

impl RadioBrowserApp {
    pub fn new(debug_mode: bool) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            radio_browser: RadioBrowserAPI::new()?,
            debug_mode,
        })
    }

    pub fn status_api(&mut self) {
        debug(self.debug_mode, "Checking status");

        let status = self.radio_browser.get_server_status().ok();

        match status {
            Some(status) => {
                println!("=== Radio Browser Status ===");
                println!("Status            : {}", status.status);
                println!("API Version       : {}", status.supported_version);
                println!(
                    "Software Version  : {}",
                    status.software_version.as_deref().unwrap_or("Unknown")
                );
                println!("Stations          : {}", status.stations);
                println!("Broken Stations   : {}", status.stations_broken);
                println!("Tags              : {}", status.tags);
                println!("Languages         : {}", status.languages);
                println!("Countries         : {}", status.countries);
                println!("Clicks (1h)       : {}", status.clicks_last_hour);
                println!("Clicks (24h)      : {}", status.clicks_last_day);
            }
            None => {
                println!("Status not found! Check your internet connection.");
            }
        }
    }

    pub fn search_builder(
        &self,
        limit: &str,
        opts: SearchOptions,
    ) -> Result<Vec<ApiStation>, Box<dyn Error>> {
        let mut builder: StationSearchBuilder = self.radio_browser.get_stations();

        debug(self.debug_mode, "Searching...");

        if let Some(name) = opts.station_name {
            builder = builder.name(name);
            // debug_value(self.debug_mode, "builder station_name ", &builder);
        }

        if let Some(lang) = opts.language {
            builder = builder.language(lang);
        }

        if let Some(country) = opts.country {
            builder = builder.country(country);
        }

        if let Some(country_code) = opts.country_code {
            builder = builder.countrycode(country_code);
        }

        if let Some(state) = opts.state {
            builder = builder.state(state);
        }

        if let Some(tag) = opts.tag {
            builder = builder.tag(tag);
        }

        if let Some(codec) = opts.codec {
            builder = builder.codec(codec);
        }

        if let Some(bitrate_min) = opts.bitrate_min {
            builder = builder.bitrate_min(*bitrate_min);
        }

        if let Some(bitrate_max) = opts.bitrate_max {
            builder = builder.bitrate_max(*bitrate_max);
        }

        let stations = builder.limit(limit).send()?;

        debug_value(self.debug_mode, "Result Station", &stations);

        Ok(stations)
    }

    /// Display logic
    pub fn print_stations(&self, stations: &[ApiStation]) {
        if stations.is_empty() {
            println!("Station Not Found");
            return;
        }

        for (station_number, station) in stations.iter().enumerate() {
            println!("{}. {}", station_number + 1, station.name);
        }
    }

    /// Kirim click ke API untuk stasiun dengan uuid tertentu.
    /// Dipanggil otomatis setiap kali stasiun diputar.
    pub fn click_station(&mut self, uuid: &str) -> Result<String, Box<dyn Error>> {
        let result = self.radio_browser.station_click(uuid)?;
        Ok(result.message)
    }

    /// Tambah satu vote untuk stasiun dengan uuid tertentu.
    /// Setiap IP hanya bisa vote sekali per stasiun.
    pub fn vote_station(&mut self, uuid: &str) -> Result<String, Box<dyn Error>> {
        let result = self.radio_browser.station_vote(uuid)?;
        if result.ok {
            Ok(result.message)
        } else {
            Err(result.message.into())
        }
    }
}

/// Wrapper untuk station_click — fire-and-forget dari thread TUI.
pub fn station_click_blocking(uuid: &str) {
    if let Ok(mut app) = RadioBrowserApp::new(false) {
        let _ = app.click_station(uuid);
    }
}

/// Wrapper untuk station_vote — dikembalikan Result agar TUI bisa tampilkan pesan.
pub fn station_vote_blocking(uuid: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut app = RadioBrowserApp::new(false)?;
    app.vote_station(uuid)
}
