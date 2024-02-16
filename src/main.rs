use std::fs::{self, File};
use std::io::{self, Write, copy};
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

fn main() -> io::Result<()> {
    // Initialize standard input and output
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Introduce the application
    println!("--------------------");
    println!("--------------------");
    println!("Welcome to the zipping CLI!");
    println!("");
    println!("To quit the application at any stage simply press Cmd + C");
    println!("");
    println!("You will be asked to enter the path for the target directory, see examples of what this might look like below:");
    println!("");
    println!("Windows: C:\\Users\\Name\\Documents\\data_to_be_zipped");
    println!("");
    println!("Mac: Users/Name/Documents/data_to_be_zipped");
    println!("");
    println!("");


    let path = loop {
        // Prompt for the directory path
        println!("Please enter the directory path: ");
        stdout.flush()?;
        let mut path = String::new();
        stdin.read_line(&mut path)?;
        let path = path.trim();

        // Check if the path exists and is a directory
        if !Path::new(&path).exists() {
            eprintln!("ERROR: The provided path does not exist! Please try again.");
        } else if !Path::new(&path).is_dir() {
            eprintln!("ERROR: The provided path is not a directory! Please try again.");
        } else {
            break path.to_string();
        }
    };

    let include_files = loop {
        // Prompt for including files outside of directories
        println!("Do you want to include files outside of directories? (yes/no): ");
        stdout.flush()?;
        let mut response = String::new();
        stdin.read_line(&mut response)?;
        match response.trim().eq_ignore_ascii_case("yes") {
            true => break true,
            false => match response.trim().eq_ignore_ascii_case("no") {
                true => break false,
                false => {
                    eprintln!("ERROR: Invalid response! Please answer 'yes' or 'no'.");
                }
            },
        }
    };

    // Proceed with zipping directories based on the user's input
    if let Err(e) = zip_directories(&path, include_files) {
        eprintln!("Error zipping directories: {}", e);
        return Err(e);
    }

    println!("Operation completed successfully.");
    Ok(())
}

fn zip_directories(base_path: &str, include_files: bool) -> io::Result<()> {
    let total_items = count_items_to_zip(base_path, include_files)?;
    let start = Instant::now();
    let mut items_processed = 0;

    for entry in WalkDir::new(base_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() || (include_files && path.is_file()) {
            let relative_path = path.strip_prefix(base_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
                .to_path_buf();
            let zip_file_name = format!("{}.zip", relative_path.to_str().unwrap().replace("/", "_"));
            let file = File::create(&zip_file_name)?;
            let mut zip = ZipWriter::new(file);

            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let file_path = entry.path();
                    if file_path.is_file() {
                        let mut file = File::open(&file_path)?;
                        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
                        let file_name = file_path.file_name().unwrap().to_str().unwrap();
                        zip.start_file(file_name, options)?;
                        copy(&mut file, &mut zip)?;
                    }
                }
            } else if include_files && path.is_file() {
                let mut file = File::open(path)?;
                let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
                let file_name = relative_path.file_name().unwrap().to_str().unwrap();
                zip.start_file(file_name, options)?;
                copy(&mut file, &mut zip)?;
            }

            zip.finish()?;
            items_processed += 1;
            let elapsed = start.elapsed();
            println!(
                "Zipped: {} ({} of {}), Time Elapsed: {:.2?}",
                zip_file_name,
                items_processed,
                total_items,
                elapsed
            );
        }
    }
    Ok(())
}

fn count_items_to_zip(base_path: &str, include_files: bool) -> io::Result<usize> {
    let mut count = 0;
    for entry in WalkDir::new(base_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() || (include_files && path.is_file()) {
            count += 1;
        }
    }
    Ok(count)
}