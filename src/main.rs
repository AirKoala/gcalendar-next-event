use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod authenticate;
mod config;

const DEFAULT_CREDS_PATH_STRING: &str = "~/.config/gcalendar-next-event/creds.json";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Authenticate with google cloud.
    Authenticate {
        #[arg(long)]
        client_id: String,
        #[arg(long)]
        client_secret: String,
        #[arg(long, short = 'S')]
        nosave: bool,
        #[arg(long, short = 'c')]
        config_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.subcommand {
        Commands::Authenticate {
            client_id,
            client_secret,
            nosave,
            config_path,
        } => {
            let creds = authenticate::Creds::authenticate(&client_id, &client_secret)
                .await
                .unwrap();
            if !nosave {
                config::Config { creds }
                    .save_to(config_path.as_deref())
                    .unwrap();
            }
        }
    }
}
