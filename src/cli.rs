use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use redio::{
    RadioBrowserApp, SearchOptions, check_app_native, play_url, station_click_blocking,
    station_vote_blocking, tui_main,
};
use std::error::Error;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = r#"
   ___          ___        ___                               ___           
  / _ \___ ____/ (_)__    / _ )_______ _    _____ ___ ____  / _ | ___  ___ 
 / , _/ _ `/ _  / / _ \  / _  / __/ _ \ |/|/ (_-</ -_) __/ / __ |/ _ \/ _ \
/_/|_|\_,_/\_,_/_/\___/ /____/_/  \___/__,__/___/\__/_/   /_/ |_/ .__/ .__/
                                                               /_/  /_/    
Radio browser app for listening radio online from radio browser <https://www.radio-browser.info>"#
)]
pub struct Cli {
    /// Enable debug output
    #[arg(short, long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub commands: Commands,
}

/// Semua filter dan opsi untuk subcommand `search`
#[derive(Args)]
pub struct SearchArgs {
    /// Limit hasil pencarian
    #[arg(long, default_value = "10")]
    pub limit: String,

    /// Cari stasiun berdasarkan nama
    #[arg(short, long)]
    pub name: Option<String>,

    /// Cari stasiun berdasarkan bahasa
    #[arg(short, long)]
    pub language: Option<String>,

    /// Cari stasiun berdasarkan negara
    #[arg(short, long)]
    pub country: Option<String>,

    /// Cari stasiun berdasarkan kode negara
    #[arg(long)]
    pub country_code: Option<String>,

    /// Cari stasiun berdasarkan state/provinsi
    #[arg(short, long)]
    pub state: Option<String>,

    /// Cari stasiun berdasarkan tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Cari stasiun berdasarkan codec
    #[arg(long)]
    pub codec: Option<String>,

    /// Bitrate minimum
    #[arg(long)]
    pub bitrate_min: Option<u16>,

    /// Bitrate maksimum
    #[arg(long)]
    pub bitrate_max: Option<u16>,

    /// Aksi setelah pencarian
    #[command(subcommand)]
    pub action: Option<SearchActions>,
}

impl SearchArgs {
    pub fn has_any_filter(&self) -> bool {
        self.name.is_some()
            || self.language.is_some()
            || self.country.is_some()
            || self.country_code.is_some()
            || self.state.is_some()
            || self.tag.is_some()
            || self.codec.is_some()
            || self.bitrate_min.is_some()
            || self.bitrate_max.is_some()
    }

    pub fn as_search_options(&self) -> SearchOptions<'_> {
        SearchOptions {
            station_name: self.name.as_deref(),
            language: self.language.as_deref(),
            country: self.country.as_deref(),
            country_code: self.country_code.as_deref(),
            state: self.state.as_deref(),
            tag: self.tag.as_deref(),
            codec: self.codec.as_deref(),
            bitrate_min: self.bitrate_min.as_ref(),
            bitrate_max: self.bitrate_max.as_ref(),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tampilkan status API dari RadioBrowser
    Status,

    /// Cari stasiun radio berdasarkan filter
    Search(Box<SearchArgs>),

    /// Vote untuk stasiun berdasarkan UUID
    ///
    /// Contoh:
    ///   redio vote 960397f0-0c18-4afe-b66d-4e0ca0a3912c
    Vote {
        /// UUID stasiun yang ingin di-vote
        uuid: String,
    },

    /// Periksa semua dependensi yang diperlukan
    Doctor,

    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Jalankan dengan antarmuka terminal (TUI)
    Tui {},
}

#[derive(Subcommand)]
pub enum SearchActions {
    /// Putar audio setelah pencarian
    ///
    /// Contoh:
    ///   search -n "jazz" play          → putar stasiun pertama (default)
    ///   search -n "jazz" play --pick 3 → putar stasiun ke-3 dari hasil
    Play {
        /// Nomor urut stasiun yang ingin diputar (1-based, default: 1)
        #[arg(short, long, default_value = "1", value_name = "NOMOR")]
        pick: usize,
    },

    /// Vote untuk stasiun dari hasil pencarian
    ///
    /// Contoh:
    ///   search -n "jazz" vote          → vote stasiun pertama
    ///   search -n "jazz" vote --pick 3 → vote stasiun ke-3
    Vote {
        /// Nomor urut stasiun yang ingin di-vote (1-based, default: 1)
        #[arg(short, long, default_value = "1", value_name = "NOMOR")]
        pick: usize,
    },
}

pub fn cli_init(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let mut app = RadioBrowserApp::new(cli.debug)?;

    match &cli.commands {
        Commands::Search(args) => command_search(&mut app, args)?,
        Commands::Status => app.status_api(),
        Commands::Vote { uuid } => command_vote(uuid),
        Commands::Doctor => match check_app_native("mpv") {
            Ok(path) => println!("Semua dependensi tersedia: {path}"),
            Err(e) => println!("Dependensi tidak ditemukan: {e}"),
        },
        Commands::Completions { shell } => {
            generate_completions(*shell);
        }
        Commands::Tui {} => match tui_main() {
            Ok(()) => {}
            Err(e) => eprintln!("Tui Error Init: {e}"),
        },
    }

    Ok(())
}

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "redio", &mut std::io::stdout());
}

/// Vote langsung via UUID — `redio vote <uuid>`
fn command_vote(uuid: &str) {
    print!("Mengirim vote untuk {uuid}... ");
    match station_vote_blocking(uuid) {
        Ok(msg) => println!("✓ {msg}"),
        Err(e) => println!("✗ Gagal: {e}"),
    }
}

fn command_search(app: &mut RadioBrowserApp, args: &SearchArgs) -> Result<(), Box<dyn Error>> {
    if !args.has_any_filter() {
        println!("Minimal satu filter harus diisi!");
        return Ok(());
    }

    let stations = app.search_builder(&args.limit, args.as_search_options())?;
    app.print_stations(&stations);

    match &args.action {
        Some(SearchActions::Play { pick }) => {
            if stations.is_empty() {
                println!("Tidak ada stasiun yang ditemukan untuk diputar.");
                return Ok(());
            }

            let index = pick.saturating_sub(1);
            match stations.get(index) {
                Some(station) => {
                    println!(
                        "Memutar stasiun #{} dari {}: {}",
                        index + 1,
                        stations.len(),
                        station.name
                    );
                    // Kirim click sebelum mulai putar
                    station_click_blocking(&station.stationuuid);
                    play_url(&station.url)?;
                }
                None => println!(
                    "Nomor {} tidak valid. Hasil pencarian hanya {} stasiun (gunakan 1–{}).",
                    pick,
                    stations.len(),
                    stations.len(),
                ),
            }
        }

        Some(SearchActions::Vote { pick }) => {
            if stations.is_empty() {
                println!("Tidak ada stasiun ditemukan.");
                return Ok(());
            }

            let index = pick.saturating_sub(1);
            match stations.get(index) {
                Some(station) => {
                    print!(
                        "Mengirim vote untuk stasiun #{} — {}... ",
                        index + 1,
                        station.name
                    );
                    match station_vote_blocking(&station.stationuuid) {
                        Ok(msg) => println!("✓ {msg}"),
                        Err(e) => println!("✗ Gagal: {e}"),
                    }
                }
                None => println!(
                    "Nomor {} tidak valid. Hasil pencarian hanya {} stasiun (gunakan 1–{}).",
                    pick,
                    stations.len(),
                    stations.len(),
                ),
            }
        }

        None => {}
    }

    Ok(())
}
