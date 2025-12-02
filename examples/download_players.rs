//! Download YouTube player files for testing
//!
//! Usage: cargo run --example download_players

use ejs::test_data::{get_cache_path, get_player_paths, ALL_VARIANTS, TEST_CASES};
use std::fs;
use std::path::Path;

fn main() {
    let player_paths = get_player_paths();
    let mut downloaded = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for test_case in TEST_CASES {
        let variants = test_case.variants.unwrap_or(ALL_VARIANTS);

        for variant in variants {
            let cache_path = get_cache_path(test_case.player, variant);
            let path = Path::new(&cache_path);

            if path.exists() {
                skipped += 1;
                continue;
            }

            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("Failed to create directory {:?}: {}", parent, e);
                        failed += 1;
                        continue;
                    }
                }
            }

            let Some(player_path) = player_paths.get(variant) else {
                eprintln!("Unknown variant: {}", variant);
                failed += 1;
                continue;
            };

            let url = format!(
                "https://www.youtube.com/s/player/{}/{}",
                test_case.player, player_path
            );

            println!("Downloading: {}", url);

            match download_file(&url, &cache_path) {
                Ok(_) => {
                    println!("  -> Saved to {}", cache_path);
                    downloaded += 1;
                }
                Err(e) => {
                    eprintln!("  -> Failed: {}", e);
                    failed += 1;
                }
            }
        }
    }

    println!();
    println!("Summary:");
    println!("  Downloaded: {}", downloaded);
    println!("  Skipped (already exists): {}", skipped);
    println!("  Failed: {}", failed);
}

fn download_file(url: &str, path: &str) -> Result<(), String> {
    // Use curl with built-in retry mechanism
    let output = std::process::Command::new("curl")
        .args([
            "-sL",
            "--connect-timeout",
            "10",
            "--max-time",
            "120",
            "--retry",
            "3",
            "--retry-delay",
            "1",
            "--retry-all-errors",
            "-o",
            path,
            url,
        ])
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !output.status.success() {
        // Clean up partial download
        fs::remove_file(path).ok();
        return Err(format!(
            "curl failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Check if file was created and has content
    let metadata = fs::metadata(path).map_err(|e| {
        fs::remove_file(path).ok();
        format!("Failed to read file: {}", e)
    })?;

    if metadata.len() == 0 {
        fs::remove_file(path).ok();
        return Err("Downloaded file is empty".to_string());
    }

    Ok(())
}
