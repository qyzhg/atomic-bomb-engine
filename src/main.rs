mod models;
mod core;

use tokio;
use clap::Parser;
use serde::de::Unexpected::Option;
use serde_json::Value;
use models::args::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut json: std::option::Option<Value> = None;
    if let Some(json_str) = args.json{
        match serde_json::from_str(&json_str){
            Ok(val) => json = val,
            Err(e) => panic!("{}", e)
        }
    }
    match core::execute::run(
        &args.url,
        args.duration_secs,
        args.concurrent_requests,
        args.timeout,
        args.verbose,
        &args.method,
        json,
        args.form,
        args.headers,
        args.cookie,
    ).await {
        Ok(result) => {
            core::show_result_with_table::show_result_with_table(result)
        },
        Err(e) => println!("Error: {}", e),
    }
}

