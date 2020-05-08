use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
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
        .lines() // split to lines serially
        .filter_map(|line: Result<String, _>| line.ok())
        .par_bridge() // parallelize
        .filter_map(|line: String| serde_json::from_str(&line).ok()) // filter out bad lines
        .for_each(|r: LogLine| {
            let mut map = histogram.lock().unwrap();
            let count = map.entry(r.typ).or_insert(0);
            *count += 1;
        });

    histogram.into_inner().unwrap()
}

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

    let histogram = histogram_parallel(file);

    let eta = now.elapsed();
    println!("{:#?}", histogram);
    println!("Finished in {}.{:0>8} seconds", eta.as_secs(), eta.subsec_nanos());
}
