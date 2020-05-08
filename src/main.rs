use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

fn main() {
    let now = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please specify a JSON log file path as a first argument.");
    }

    let fname = &args[1];
    let path = Path::new(fname);
    let file = match File::open(&path) {
        Err(why) => panic!("Cannot open {}: {}", path.display(), why.to_string()),
        Ok(file) => file,
    };

    let mut histogram: HashMap<String, u64> = HashMap::with_capacity(128);

    let mut reader = BufReader::new(file);
    let mut line = String::with_capacity(2048);
    loop {
        match reader.read_line(&mut line) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }

                match serde_json::from_str(&line) {
                    Ok(value) => {
                        match value {
                            Value::Object(r) => {
                                let t: String = r["type"].to_string();
                                        let count = histogram.entry(t).or_insert(0);
                                *count += 1;
                            }
                            _ => {
                                let count = histogram
                                    .entry(String::from("non-json-object"))
                                    .or_insert(0);
                                *count += 1;
                            }
                        }
                    }
                    Err(_err) => {
                        let count = histogram.entry(String::from("invalid-json")).or_insert(0);
                        *count += 1;
                    }
                }

                // Clear the buffer so it doesn't grow.
                line.clear();
            }
            Err(_err) => {
                let count = histogram
                    .entry(String::from("unreadable-line"))
                    .or_insert(0);
                *count += 1;
            }
        };
    }

    println!("{:?}", histogram);
    println!("Finished in {} nanoseconds", now.elapsed().as_nanos());
}
