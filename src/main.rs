mod models;
mod core;

use reqwest;
use tokio;
use clap::Parser;
use anyhow::Context;
use models::arges::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match core::run::run(&args.url, args.duration_secs, args.concurrent_requests, args.timeout).await {
        Ok(result) => {
            core::show_result_with_table::show_result_with_table(result)
        },
        Err(e) => println!("Error: {}", e),
    }
}

