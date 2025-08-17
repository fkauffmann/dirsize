use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::fmt::Write; // Import Write trait for formatting
use std::time::Instant;
use std::io::{stdout, Write as IoWrite};
use std::thread;
use std::time::Duration;

/// Recursively compute the total size of a directory (including all subdirectories and files)
fn dir_size(path: &Path) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    size += metadata.len();
                } else if metadata.is_dir() {
                    size += dir_size(&path);
                }
            }
        }
    }
    size
}

fn main() -> io::Result<()> {
    // Start timer
    let start_time = Instant::now();
    // Parse command line arguments for directory path
    let args: Vec<String> = env::args().collect();
    let dir_path = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        Path::new(".")
    };

    // Read the directory entries
    let current_dir = fs::read_dir(dir_path)?;

    println!("Directory: {}\n", dir_path.display());

    let mut dirs_and_sizes = Vec::new();
    for entry in current_dir {
        let entry = entry?;
        let path = entry.path();
        // Only consider subdirectories
        if entry.file_type()?.is_dir() {
            let size = dir_size(&path);
            dirs_and_sizes.push((path, size));
            // Animation: print a rotating bar
            static FRAMES: [&str; 4] = ["|", "/", "-", "\\"];
            let frame = FRAMES[dirs_and_sizes.len() % FRAMES.len()];
            print!("\rComputing sizes... {}", frame);
            stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(60));
        }
    }
    // Efface l'animation
    print!("\r                      \r");
    stdout().flush().unwrap();
    // Sort directories by size, descending
    dirs_and_sizes.sort_by(|a, b| b.1.cmp(&a.1));
    // Compute total size
    let total_size: u64 = dirs_and_sizes.iter().map(|x| x.1).sum();
    let total_mb = (total_size as f64) / (1024.0 * 1024.0);
    // Find the largest directory size for scaling the bar
    let max_size = dirs_and_sizes.first().map(|x| x.1).unwrap_or(1);
    // ANSI colors (6 colors)
    let colors = [
        "\x1b[31m", // red
        "\x1b[32m", // green
        "\x1b[33m", // yellow
        "\x1b[34m", // blue
        "\x1b[35m", // magenta
        "\x1b[36m", // cyan
    ];
    let color_end = "\x1b[0m";
    for (i, (path, size)) in dirs_and_sizes.iter().enumerate() {
        let size_mb = (*size as f64) / (1024.0 * 1024.0);
        let bar_len = if max_size > 0 {
            ((*size as f64) / (max_size as f64) * 50.0).round() as usize
        } else {
            1
        };
        let color = colors[i % colors.len()];
        let bar = format!("{}{}{}", color, "❚".repeat(bar_len.max(1)), color_end);
        // Format MB with thousands separator
        let mut size_str = String::new();
        write!(&mut size_str, "{:.2}", size_mb).unwrap();
        let size_str = add_thousands_sep(&size_str);
        println!("{} {} ({} MB)", bar, path.display(), size_str);
    }
    // Format total MB with thousands separator
    let mut total_str = String::new();
    write!(&mut total_str, "{:.2}", total_mb).unwrap();
    let total_str = add_thousands_sep(&total_str);
    println!("\nTotal size: {} MB", total_str);
    // Print execution duration
    let duration = start_time.elapsed();
    println!("\nExecution time : {} ms", duration.as_millis());
    Ok(())
}

/// Add thousands separator to a string representing a floating point number
fn add_thousands_sep(num: &str) -> String {
    let mut parts = num.split('.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next();
    let int_chars: Vec<char> = int_part.chars().rev().collect();
    let mut out = String::new();
    for (i, c) in int_chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    let int_with_sep: String = out.chars().rev().collect();
    match frac_part {
        Some(frac) => format!("{}.{}", int_with_sep, frac),
        None => int_with_sep,
    }
}
