use clap::{App, Arg};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Deserialize, Debug)]
struct LogLine {
    #[serde(rename = "type")]
    typ: String,
}

fn histogram_parallel(file: File) -> HashMap<String, u64> {
    let histogram: Mutex<HashMap<String, u64>> = Mutex::new(HashMap::with_capacity(128));

    let reader = BufReader::new(file);
    reader
        .lines() // split to lines serially (single thread)
        .filter_map(|line: Result<String, _>| line.ok())
        .par_bridge() // parallelize all lines to dynamically allocated workers
        .filter_map(|line: String| serde_json::from_str(&line).ok()) // reject non-JSON lines
        .for_each(|r: LogLine| {
            let mut map = histogram.lock().unwrap();
            let count = map.entry(r.typ).or_insert(0);
            *count += 1;
        });

    histogram.into_inner().unwrap()
}

fn main() {
    let cli = App::new("jlh")
        .version("0.1")
        .author("Dimitar P. <mitko.p@gmail.com>")
        .about(
            "Reads a JSON log file with one record per line and produces a histogram
on the type field of each record.",
        )
        .arg(
            Arg::with_name("INPUT")
                .short("f")
                .long("file")
                .takes_value(true)
                .required(true)
                .help("The log file to analyze, stdin if omitted"),
        )
        .get_matches();

    let fname = cli.value_of("INPUT").unwrap();
    let path = Path::new(fname);
    let file = match File::open(&path) {
        Err(why) => panic!("Cannot open {}: {}", path.display(), why.to_string()),
        Ok(file) => file,
    };

    let now = Instant::now();
    let histogram = histogram_parallel(file);

    let eta = now.elapsed();
    println!("{:#?}", histogram);
    println!(
        "Finished in {}.{:0>8} seconds",
        eta.as_secs(),
        eta.subsec_nanos()
    );
}
