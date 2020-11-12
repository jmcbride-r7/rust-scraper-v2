extern crate reqwest;
extern crate scraper;
extern crate regex;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{io::{self, Read, Write}, fs::{File, OpenOptions}, path::Path};

#[derive(Serialize, Deserialize)]
struct Payload {
    payload_type: String,
    payload_text: String,
    expected_fail: bool,
    valid: bool,
}

#[derive(Serialize, Deserialize)]
struct Filters {
    ignore_text: String,
}

fn payload_scraper<'a>(url: &String) -> Vec<String> {

    // creating reqwest crate client
    let client = reqwest::blocking::Client::new();
    // assigning method url
    let input_url = url;

    let mut res = client.get(input_url).send().unwrap();

    println!("RUST WEB SCRAPER - Status for {}: {}", input_url, res.status());

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let mut payload_vector = Vec::new();
    let fragment = Html::parse_document(body.as_str());
    let code_selector = Selector::parse("code").unwrap();

    for code_reference in fragment.select(&code_selector) {
        let code_txt = code_reference.text().collect::<Vec<&str>>();
        payload_vector.push(code_txt[0].to_string());
    }
    payload_vector
}

fn parse_json() {

    let json_file_path = Path::new("payload.json");
    let file = File::open(json_file_path).expect("file not found");

    let payloads:Vec<Payload> = serde_json::from_reader(file).expect("Error while reading!");


}


fn parse_filtering() -> Vec<String> {

    let mut filtered_payloads = Vec::new();

    let json_file_path = Path::new("ignore.json");
    let file = File::open(json_file_path).expect("file not found");

    let filters:Vec<Filters> = serde_json::from_reader(file).expect("Error while reading!");

    for filter in filters {
        filtered_payloads.push(filter.ignore_text);
    }

    filtered_payloads
}

fn main() -> std::io::Result<()> {

    let mut input_url = String::new();
    let ignore_payloads = parse_filtering();

    println!("Enter URL for Scraper: ");

    io::stdin()
        .read_line(&mut input_url)
        .expect("Failed to read line");

    let mut file = OpenOptions::new().write(true).create(true).open("payload.json").unwrap();

    let scraped_payloads = payload_scraper(&input_url);
    let mut payload_holder: Vec<Payload> = Vec::new();

    for ignored in ignore_payloads.iter() {
        for code in scraped_payloads.iter() {
            if !code.contains(ignored) {
                let payload = Payload {
                    payload_type: "1".to_string(),
                    payload_text: code.to_string(),
                    expected_fail: false,
                    valid: true,
                };
                payload_holder.push(payload);
            }
        }
    }

    let payloads_json: String = serde_json::to_string_pretty(&payload_holder)?;

    file.write((&payloads_json).as_ref()).expect("Unable to write file");

    parse_json();

    Ok(())
}