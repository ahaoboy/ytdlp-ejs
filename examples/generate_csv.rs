//! Export test cases to CSV format
//!
//! Usage: cargo run --example generate_csv > cases.csv
//!        cargo run --example generate_csv -- --output cases.csv

use ejs::test_data::{get_cache_path, ALL_VARIANTS, TEST_CASES};
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse output file argument
    let output_file = if args.len() >= 3 && args[1] == "--output" {
        Some(&args[2])
    } else {
        None
    };

    let mut output: Box<dyn Write> = match output_file {
        Some(path) => {
            let file = File::create(path).expect("Failed to create output file");
            Box::new(file)
        }
        None => Box::new(io::stdout()),
    };

    let mut count = 0;

    for test_case in TEST_CASES {
        let variants = test_case.variants.unwrap_or(ALL_VARIANTS);

        for variant in variants {
            let cache_path = get_cache_path(test_case.player, variant);
            let path = Path::new(&cache_path);

            // Check if player file exists
            if !path.exists() {
                continue;
            }

            let filename = format!("{}-{}", test_case.player, variant);

            // Export n tests
            for step in test_case.n {
                writeln!(output, "{} n {} {}", filename, step.input, step.expected)
                    .expect("Failed to write");
                count += 1;
            }

            // Export sig tests
            for step in test_case.sig {
                writeln!(output, "{} sig {} {}", filename, step.input, step.expected)
                    .expect("Failed to write");
                count += 1;
            }
        }
    }

    if output_file.is_some() {
        eprintln!("Exported {} test cases", count);
    }
}
