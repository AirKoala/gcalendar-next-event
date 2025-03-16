use chrono::Utc;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod authenticate;
mod calendar;
mod config;

// const DEFAULT_CREDS_PATH_STRING: &str = "~/.config/gcalendar-next-event/creds.json";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Commands,
    #[arg(long, short = 'c')]
    config_path: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Authenticate with google cloud.
    #[command(visible_alias("auth"))]
    Authenticate {
        /// The client ID for the Google Cloud project.
        #[arg(long)]
        client_id: Option<String>,

        /// The client secret for the Google Cloud project.
        #[arg(long)]
        client_secret: Option<String>,

        /// Don't save the credentials to the config file.
        #[arg(long, short = 'S')]
        nosave: bool,
    },

    /// Get the next event from the user's calendar.
    #[command(visible_alias("event"))]
    GetNextEvent {
        /// Don't use cached data.
        #[arg(long, short = 'C')]
        nocache: bool,
    },

    /// List the user's calendars.
    #[command(visible_alias("cal"))]
    ListCalendars,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut config = config::Config::load_from(args.config_path.as_deref()).unwrap_or_else(|_| {
        eprintln!("Failed to load config file. Creating a new one at the default location.");
        config::Config::new_default()
    });

    match args.subcommand {
        Commands::Authenticate {
            client_id,
            client_secret,
            nosave,
        } => {
            let creds = authenticate::Creds::authenticate(
                &client_id.unwrap_or(config.creds.client_id),
                &client_secret.unwrap_or(config.creds.client_secret),
            )
            .await
            .unwrap();

            config.creds = creds;

            if !nosave {
                config.save_to(args.config_path.as_deref()).unwrap();
            }
        }
        Commands::GetNextEvent { nocache } => {
            config.nocache = nocache;

            let calendar = calendar::Calendar::new(&config).await.unwrap();

            if let Ok(Some(event)) = calendar.get_next_event().await {
                if config.max_time_until_event_seconds.is_none()
                    || Utc::now()
                        + chrono::TimeDelta::seconds(config.max_time_until_event_seconds.unwrap())
                        > event.start_time
                {
                    print!("{}", event.format_status_line());
                }
            }
            println!();
        }
        Commands::ListCalendars => {
            calendar::Calendar::new(&config)
                .await
                .expect("Failed to get calendars.")
                .get_calendars_table()
                .await
                .expect("Failed to generate table.")
                .printstd();
        }
    }
}
