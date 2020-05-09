### Introduction

This is a Rust command line tool that calculates a histogram of the separate types of JSON records in an input JSON log file (one JSON object per line).

A sample input file would be:

```json
{"type":"B","foo":"bar","items":["one","two"]}
{"type": "A","foo": 4.0  }
{"type": "B","bar": "abcd"}
```

The output histogram would report a count of 2 for type B and 1 for type A. It would also report total of 73 bytes for type B and 26 for type A.

### How to compile and use

Git clone:
```sh
git clone https://github.com/dimitarvp/json-log-histogram-rust.git
cd json-log-histogram-rust
```

Compile:
```sh
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

To test, you can use one of the files in the `inputs/` folder. They have been added to the repository through Git LFS so you have to fetch and checkout them first, and then decompress:
```sh
# Install LFS in the current repo and download the files.
git lfs install
git lfs fetch
git lfs checkout

# Decompress.
gunzip -v inputs/*.gz

# Run the tool with one of: 1MB.json, or 10MB.json, or 100MB.json, or 1000MB.json
./target/release/jlh -f inputs/1000MB.json
```

The tool prints an aligned text table and a total runtime at the bottom.

### Benchmarks

|CPU|File size|Time in seconds|
|-|-|-|
|[Xeon W-2150B @ 3.00GHz](http://www.cpu-world.com/CPUs/Xeon_W/Intel-Xeon%20W%20W-2150B.html)|1MB|0.11091947|
|[Xeon W-2150B @ 3.00GHz](http://www.cpu-world.com/CPUs/Xeon_W/Intel-Xeon%20W%20W-2150B.html)|10MB|0.62043929|
|[Xeon W-2150B @ 3.00GHz](http://www.cpu-world.com/CPUs/Xeon_W/Intel-Xeon%20W%20W-2150B.html)|100MB|0.643637170|
|[Xeon W-2150B @ 3.00GHz](http://www.cpu-world.com/CPUs/Xeon_W/Intel-Xeon%20W%20W-2150B.html)|1000MB|5.690039287|
|[i7-4870HQ @ 2.50GHz](http://www.cpu-world.com/CPUs/Core_i7/Intel-Core%20i7-4870HQ%20Mobile%20processor.html)|1MB|0.07234297|
|[i7-4870HQ @ 2.50GHz](http://www.cpu-world.com/CPUs/Core_i7/Intel-Core%20i7-4870HQ%20Mobile%20processor.html)|10MB|0.68889124|
|[i7-4870HQ @ 2.50GHz](http://www.cpu-world.com/CPUs/Core_i7/Intel-Core%20i7-4870HQ%20Mobile%20processor.html)|100MB|0.670027735|
|[i7-4870HQ @ 2.50GHz](http://www.cpu-world.com/CPUs/Core_i7/Intel-Core%20i7-4870HQ%20Mobile%20processor.html)|1000MB|6.659739416|
|[i3-3217U @ 1.80GHz](http://www.cpu-world.com/CPUs/Core_i3/Intel-Core%20i3-3217U%20Mobile%20processor.html)|1MB|0.14369994|
|[i3-3217U @ 1.80GHz](http://www.cpu-world.com/CPUs/Core_i3/Intel-Core%20i3-3217U%20Mobile%20processor.html)|10MB|0.49248859|
|[i3-3217U @ 1.80GHz](http://www.cpu-world.com/CPUs/Core_i3/Intel-Core%20i3-3217U%20Mobile%20processor.html)|100MB|0.535957719|
|[i3-3217U @ 1.80GHz](http://www.cpu-world.com/CPUs/Core_i3/Intel-Core%20i3-3217U%20Mobile%20processor.html)|1000MB|3.773678079|

### Implementation details and notes

- Using Rust `1.43.1`.
- Using the [rayon](https://crates.io/crates/rayon) crate for transparent parallelization of the histogram calculation.
- Using the [clap](https://crates.io/crates/clap) crate to parse the command line options (only one, which is the input JSON log file).
- Using the [prettytable-rs](https://crates.io/crates/prettytable-rs) crate to produce a pretty command line table with the results.
- Using `serde_json` to read each JSON record to a struct.
- Skipped the ability to pipe files to the tool so it can read from stdin. The motivation was that `rayon` does not provide its `.par_bridge` function to polymorphic `Box<dyn BufRead>` objects (which is the common denominator of `std::io::stdin().lock()` and `std::fs::File.open(path)`). I could have probably made it work but after 2 hours of attempts I realized that it might take a long time so I cut it short.
- Used the `.lines()` function on the `BufReader` even though that allocates a new `String` per line. I am aware of the better `BufReader.read_line` idiom with a single `String` buffer (which is cleared after every line is consumed) and my initial non-parallel version even used it -- see [this commit](https://github.com/dimitarvp/json-log-histogram-rust/commit/28a7190c8783ff94825159145a0abdc983add433). But I couldn't find a quick way to translate this idiom to simply having something with the `.lines()` function (`rayon` expects an `Iterator`). I could have implemented `Iterator` for a wrapping struct or enum but, same as above, I was not sure if it will not take me very long. IMO even with that caveat the tool is very fast (see performance results table below).
- The commit history got slightly botched because I had to use [bfg](https://rtyley.github.io/bfg-repo-cleaner/) to remove the 1MB / 10MB / 100MB / 1000MB JSON files that I added earlier (which I replaced with gzipped variants later).
