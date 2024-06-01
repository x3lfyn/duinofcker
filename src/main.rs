use tokio::task::JoinHandle;
use std::error::Error;
use tracing_subscriber::prelude::*;
use tracing_subscriber::fmt;
use tracing_subscriber::filter::EnvFilter;
use tracing::metadata::LevelFilter;
use clap::Parser;
use rand::Rng;
use std::sync::Arc;

pub mod algorithm;
pub mod util;
pub mod emulator;

use crate::emulator::emulator;

#[derive(Parser, Debug)]
#[command(name = "duinofcker")]
#[command(bin_name = "duinofcker")]
struct Cli {
    #[arg(short, long)]
    number: u64,

    #[arg(short, long, default_value_t = 295.0)]
    hashrate: f64,

    #[arg(short, long, default_value_t = 5.0)]
    range: f64,

    #[arg(short, long)]
    username: String,

    #[arg(short, long)]
    mining_key: String
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
            )
        .init();

    let cli = Cli::parse();

    let mut handles: Vec<JoinHandle<()>> = vec![];

    let username_arc = Arc::new(cli.username);
    let mining_key_arc = Arc::new(cli.mining_key);

    for i in 0..cli.number {
        let username = Arc::clone(&username_arc);
        let mining_key = Arc::clone(&mining_key_arc);

        let handle = tokio::spawn(async move {
            let half_range = cli.range / 2.0;
            let hashrate = rand::thread_rng().gen_range(-half_range..half_range) + cli.hashrate;
            let _ = emulator(&format!("arduino{}", i), hashrate, &username, &mining_key).await;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await?;
    };
//    println!("{:#?}", algo("5c50c5631c92a9814220b1ed3709f0f05f4a34b1", hex!("27c1005102ba5fd9bb84347546999d1a7377cfda"), 100000, precalc))
    //println!("{:#?}", mine("205d7c95fdc2ce3e9bc682d82936ec4c4603e0c8".to_string(), "8ce9c115f23270fca847d60a4c13d597619f4a26".to_string(), 8, 305.0).await);

    Ok(())
}
