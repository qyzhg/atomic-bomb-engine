mod models;
mod core;

use tokio;
use clap::Parser;
use models::args::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match core::execute::run(
        &args.url,
        args.duration_secs,
        args.concurrent_requests,
        args.timeout,
        args.verbose,
        &args.method,
        &args.json,
        &args.form,
        args.headers,
        args.cookie,
    ).await {
        Ok(result) => {
            core::show_result_with_table::show_result_with_table(result)
        },
        Err(e) => println!("Error: {}", e),
    }
}

