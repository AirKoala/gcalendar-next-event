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
    Authenticate {
        /// The client ID for the Google Cloud project.
        #[arg(long)]
        client_id: String,

        /// The client secret for the Google Cloud project.
        #[arg(long)]
        client_secret: String,

        /// Don't save the credentials to the config file.
        #[arg(long, short = 'S')]
        nosave: bool,
    },

    /// Get the next event from the user's calendar.
    GetNextEvent {
        /// Don't use cached data.
        #[arg(long, short = 'C')]
        nocache: bool,
    },
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
            let creds = authenticate::Creds::authenticate(&client_id, &client_secret)
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

            println!(
                "{}",
                match calendar.get_next_event().await {
                    Ok(Some(event)) => {
                        event.format_status_line()
                    }
                    _ => {
                        "".to_string()
                    }
                }
            );
        }
    }
}
