use clap::{App, Arg};
#[macro_use]
extern crate prettytable;
use prettytable::Table;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
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

fn histogram_parallel(file: File) -> HashMap<String, (usize, usize)> {
    let histogram: Mutex<HashMap<String, (usize, usize)>> =
        Mutex::new(HashMap::with_capacity(128));

    let reader = BufReader::new(file);
    reader
        .lines() // split to lines serially (single thread)
        .filter_map(|line: Result<String, _>| line.ok())
        .par_bridge() // parallelize all lines to dynamically allocated workers
        .filter_map(|line: String| {
            serde_json::from_str(&line)
                .ok()
                .map(|record| (record, line.len()))
        })
        .for_each(|tuple: (LogLine, usize)| {
            let (record, len) = tuple;
            let mut locked_histogram = histogram.lock().unwrap();
            let (count, bytes) = locked_histogram.entry(record.typ).or_insert((0, 0));
            *count += 1;
            *bytes += len;
        });

    histogram.into_inner().unwrap()
}

fn main() {
    // Setup a CLI application's metadata and options.
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
                .help("The JSON log file to analyze"),
        )
        .get_matches();

    // Open the file specified as a mandatory CLI option or exit if that fails.
    let fname = cli.value_of("INPUT").unwrap();
    let path = Path::new(fname);
    let file = match File::open(&path) {
        Err(why) => panic!("Cannot open {}: {}", path.display(), why.to_string()),
        Ok(file) => file,
    };

    // Produce the histogram and measure how much time it took.
    let now = Instant::now();
    let histogram = histogram_parallel(file);
    let eta = now.elapsed();

    // Copy the histogram in a map with sorted keys.
    let mut sorted_map: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    sorted_map.extend(histogram.into_iter());

    // Prepare and print a console table with the histogram results.
    let mut t = Table::new();
    t.add_row(row![bFg->"Record type", bFg->"Count", bFg->"Bytes"]);
    for (key, (count, bytes)) in sorted_map.iter() {
        t.add_row(row![key, count, bytes]);
    }
    t.printstd();

    // Print the time it took to calculate the histogram.
    println!(
        "Finished in {}.{:0>8} seconds",
        eta.as_secs(),
        eta.subsec_nanos()
    );
}
