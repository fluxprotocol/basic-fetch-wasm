use std::env;
use serde::{ Deserialize, Serialize };
use serde_json::json;
use jsonpath_rust::JsonPathFinder;
use bigdecimal::{ BigDecimal };
use std::str::{ FromStr };
use std::ops::{ Add, Div, Mul };
use std::collections::HashMap;

mod flux;

#[derive(Serialize, Deserialize, Debug)]
struct Source {
    end_point: String,
    source_path: String,
    source_parse_as: Option<String>,
    multiplier: Option<String>,
    http_method: Option<String>,
    http_headers: Option<HashMap<String, String>>,
    http_body: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let sources: Vec<Source> = serde_json::from_str(args.get(1).unwrap()).unwrap();
    let sources_type = args.get(2).unwrap();
    let mut string_result: String = "".to_string();
    let mut number_result = BigDecimal::from(0);
    let mut used_sources: u32 = 0;

    assert_ne!(sources.len(), 0, "ERR_NO_SOURCES");

    if sources_type == "string" {
        assert_eq!(sources.len(), 1, "ERR_TOO_MUCH_SOURCES");
    } else if sources_type == "number" {
        assert!(args.get(3).is_some(), "ERR_NO_MULTIPLIER");
    } else {
        panic!("ERR_UNSUPPORTED_TYPE");
    }

    for source in sources.iter() {
        let http_result = flux::http_call(&source.end_point, Some(flux::HttpCallOptions {
            method: source.http_method.clone(),
            body: source.http_body.clone(),
            headers: source.http_headers.clone(),
        }));

        if http_result.is_err() {
            println!("Could not fetch {}:{}:{:?}", &source.end_point, &source.source_path, &source.http_headers);
            eprintln!("{}", http_result.unwrap_err().error);
            continue;
        }

        let http_result_data = http_result.unwrap().result;
        
        // Finds a value in the returned json using the path given in args
        let finder = JsonPathFinder::from_str(&http_result_data, &source.source_path);
        if finder.is_err() {
            eprintln!("Invalid source path {}", &source.source_path);
            continue;
        }

        let unwrapped_finder = finder.unwrap();
        let found_values = unwrapped_finder.find();
        let result_value = match found_values.get(0) {
            Some(val) => val,
            None => {
                eprintln!("Could not find: {}, skipping api source", source.source_path);
                continue;
            }
        };

        println!("Matching values found: {}", found_values.len());

        if sources_type == "string" {
            if found_values.len() > 1 {
                string_result = json!(found_values).to_string();
            } else {
                if result_value.is_string() {
                    string_result = result_value.as_str().unwrap().to_string();
                } else {
                    string_result = result_value.to_string();
                }
            }
        } else if sources_type == "number" {
            assert_eq!(found_values.len(), 1, "ERR_TOO_MUCH_RESULTS");

            // Converting numbers to strings so we can use BigDecimal to combine them all
            let mut val: Option<BigDecimal> = None;

            if result_value.is_i64() {
                val = Some(BigDecimal::from(result_value.as_i64().unwrap()));
            } else if result_value.is_string() {
                // strings can still be valid numbers
                val = Some(BigDecimal::from_str(result_value.as_str().unwrap()).unwrap());
            } else if result_value.is_u64() {
                val = Some(BigDecimal::from(result_value.as_u64().unwrap()));
            } else if result_value.is_f64() {
                val = Some(BigDecimal::from_str(&result_value.as_f64().unwrap().to_string()).unwrap());
            }

            val = match &source.multiplier {
                Some(multiplier) => Some(BigDecimal::from_str(&multiplier).unwrap().mul(val.unwrap())),
                None => val,
            };

            println!("url: {}, source: {}, result: {:?}", &source.end_point, &source.source_path, val);

            number_result = number_result.add(val.unwrap());
        }

        used_sources += 1;
    }

    println!("used sources: {}/{}", &used_sources, sources.len());
    assert_ne!(used_sources, 0, "ERR_FAILING_SOURCES");

    if sources_type == "string" {
        flux::exit_with_outcome(&flux::Outcome::Valid(string_result));
    } else if sources_type == "number" {
        let multiplier = BigDecimal::from_str(args.get(3).unwrap()).unwrap();

        number_result = number_result.div(BigDecimal::from(used_sources));
        number_result = number_result.mul(multiplier);
        number_result = number_result.round(0);

        flux::exit_with_outcome(&flux::Outcome::Valid(number_result.to_string()));
    }
}
