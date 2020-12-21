extern crate reqwest;
extern crate scraper;
extern crate regex;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{io::{Read, Write}, fs::{File, OpenOptions}, path::Path};

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
        //let payload = code_txt[0].to_string();
        //Remove " " from payloads
        //payload_vector.push(payload.replace(" ", ""));
        payload_vector.push(code_txt[0].to_string());
    }
    payload_vector
}

fn parse_filtering() -> Vec<String> {

    let mut filtered_payloads = Vec::new();

    let json_file_path = Path::new("res/ignore.json");
    let file = File::open(json_file_path).expect("file not found");

    let filters:Vec<Filters> = serde_json::from_reader(file).expect("Error while reading!");

    for filter in filters {
        filtered_payloads.push(filter.ignore_text);
    }

    filtered_payloads
}

fn write_payloads(sensor: &str, url: String, mut output_file: File) {
    let ignore_payloads = parse_filtering();

    let mut scraped_payloads = payload_scraper(&url);

    if sensor != "xss" {
        println!("Entered cmdi block");
        let mut payloads = Vec::new();
        for text in &scraped_payloads {
            for payload in text.lines() {
                payloads.push(payload.to_string());
            }
        }
        scraped_payloads = payloads;
    }

    println!("num payloads: {}", scraped_payloads.len());

    let mut payload_holder: Vec<Payload> = Vec::new();

    for payload_text in scraped_payloads {
        let payload = Payload {
            payload_type: "0".to_string(),
            payload_text: payload_text.to_string().replace(" ", ""),
            //payload_text: payload_text.to_string(),
            expected_fail: false,
            valid: true,
        };

        let mut ignore_payload = false;

        for ignore in ignore_payloads.iter() {
            if ignore.eq(&payload_text) {
                ignore_payload = true;
            }
        }
        if !ignore_payload {
            payload_holder.push(payload);
        }
    }

    let payloads_json: String = serde_json::to_string_pretty(&payload_holder).unwrap();

    output_file.write((&payloads_json).as_ref()).expect("Unable to write file");

}

fn main() {

    let xss_url = String::from("https://owasp.org/www-community/xss-filter-evasion-cheatsheet");
    let xss_file = OpenOptions::new().write(true).create(true).open("output/xss_payloads.json").unwrap();

    let cmdi_url = String::from("https://github.com/payloadbox/command-injection-payload-list");
    let cmdi_file = OpenOptions::new().write(true).create(true).open("output/cmdi_payloads.json").unwrap();

    let sqli_url = String::from("https://owasp.org/www-community/attacks/SQL_Injection_Bypassing_WAF");
    let sqli_file = OpenOptions::new().write(true).create(true).open("output/sqli_payloads.json").unwrap();

    write_payloads("xss", xss_url, xss_file);
    write_payloads("cmdi", cmdi_url, cmdi_file);
    write_payloads("sqli", sqli_url, sqli_file);

    // let test = " /?id=1+union+select+1,2,3/*";
    // println!("Before: {}", test);
    //
    // let test1 = test.replace(" ", "");
    // println!("After: {}", test1);
    //
    // assert!(test.contains(" "));



}