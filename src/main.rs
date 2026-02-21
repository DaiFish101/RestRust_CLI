use clap::Parser;
use colored::*;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::fs;
use std::str::FromStr;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'X', long)]
    method: String,

    #[arg(short = 'u', long)]
    url: String,

    #[arg(short = 'H', long)]
    headers: Option<Vec<String>>,

    #[arg(short = 'd', long)]
    body: Option<String>,

    #[arg(long)]
    body_file: Option<String>,

    #[arg(long)]
    show_headers: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let client = Client::new();
    let mut header_map = HeaderMap::new();

    if let Some(headers) = args.headers {
        for header in headers {
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = HeaderName::from_str(parts[0].trim())?;
                let value = HeaderValue::from_str(parts[1].trim())?;
                header_map.insert(key, value);
            }
        }
    }

    let method = args.method.to_uppercase();
    let mut request = match method.as_str() {
        "GET" => client.get(&args.url),
        "POST" => client.post(&args.url),
        "PUT" => client.put(&args.url),
        "DELETE" => client.delete(&args.url),
        _ => {
            eprintln!("{}", "Invalid HTTP method!".red());
            return Ok(());
        }
    };

    request = request.headers(header_map);

    if let Some(file_path) = args.body_file {
        let file_content = fs::read_to_string(file_path)?;
        let json_body: Value = serde_json::from_str(&file_content)?;
        request = request.json(&json_body);
    }

    if let Some(body) = args.body {
        let json_body: Value = serde_json::from_str(&body)?;
        request = request.json(&json_body);
    }

    let start = Instant::now();
    let response = request.send()?;
    let duration = start.elapsed();

    let status = response.status();
    let status_str = status.to_string();

    let colored_status = if status.is_success() {
        status_str.green()
    } else if status.is_client_error() {
        status_str.yellow()
    } else if status.is_server_error() {
        status_str.red()
    } else {
        status_str.normal()
    };

    println!("{} {}", "Status:".bold(), colored_status);
    println!("{} {:.2?}", "Response Time:".bold(), duration);

    if args.show_headers {
        println!("{}", "\nResponse Headers:".bold());
        for (key, value) in response.headers() {
            println!("{}:{:?}", key, value);
        }
    }

    let text = response.text()?;

    println!("\n{}", "Response Body:".bold());

    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("{}", text);
    }
    Ok(())
}