use std::io; // Module for command line input/ouput
use std::path::{Path, PathBuf}; // Module for directory/file paths
use std::fs; // Module for file system manipulation
use std::time::Instant; // Modules used for timing things
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter, CompressionMethod};
use std::fs::File;
use std::ffi::OsStr;



// This script aims to be as readable/interpretable as possible 
// so there are times where it is deliberatly less concise or 
// efficient than it could be. 

fn main() -> io::Result<()> {
    
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

        // Should files be compressed as well as directories?
        let zip_files = answer_zip_files();

        // Get the path of the target directory
        let dir_path = get_target_dir();

        // Zipping
        if let Err(e) = zip(&dir_path, zip_files) {
            eprintln!("Error zipping files and directories: {}", e);
        }
        Ok(println!("Operation completed successfully."))
}

// Function to read in answer to zipping files 
fn answer_zip_files() -> bool {
    // Ask the user whether to zip files as well as directories
    println!("Do you also want to zip files outside of directories? (yes/no): ");
    // Initiatlise string variable to hold user response
    let mut input = String::new();
    // initialise user input
    io::stdin()
        .read_line(&mut input) // Read user input and store in the input variable
        .expect("ERROR: User input cannot be interpreted."); // Print error message if can't read input

    let input = input.trim(); // Remove newline characters

    let output_bool = match input {
        "yes" => true, // If the user inputs yes then input is set to true
        "no" => false, // If the user inputs no then input is set to false
        _ => { // If the user input is anything other than yes or no
            println!("ERROR: Invalid response! Please answer 'yes' or 'no'.");
            answer_zip_files() // Recursively call the function again until a valid response is given
        }
    };
    return output_bool
}

// Function to get the path to the target directory and verify the directory exists
fn get_target_dir() -> String {
    // Ask the user for the directory path
    println!("Please enter the directory path: ");
    // Initialise a variable to hold the user input
    let mut directory_path = String::new();
    io::stdin()
        .read_line(&mut directory_path)
        .expect("ERROR: User input cannot be interpreted."); // Print error message if can't read input

    let directory_path = directory_path.trim().to_string(); // Remove newline characters
    let path = Path::new(&directory_path); // Create a Path type variable 
    // Check if the path exists and is a directory
    if path.exists() && path.is_dir() { // Check that the path exists and that is leads to a directory
        // No action needed
    } else { // If directory does not exist or is not a directory
        println!("ERROR: The directory at the path provided does not exist or it is not a directory!");
        return get_target_dir() // Recursively call the function again until a valid path is given
    }
    return directory_path // Return a the directory path as a string rather than a path
}

fn zip(target_dir_path: &String, zip_files: bool) -> io::Result<()> {

    let start = Instant::now(); // Start a timer

    let target_path = PathBuf::from(target_dir_path); // Create PathBuf from the path String (PathBuf is mutable unlike Path)
    let zip_path = target_path.join("zipped"); // Create a new PathBuf for zipped directory (slashes are handled automatically for Paths/PathBufs)

    // Correctly calling count_items_in_directory and using its results
    let (file_count, dir_count) = count_items_in_directory(target_dir_path, zip_files)?;
    println!("Files to zip: {}, Directories to zip: {}", file_count, dir_count);
    let total_to_zip = file_count + dir_count;

    if !zip_path.exists() { // Check whether zipping directory has been created
        // Try to create the directory
        match fs::create_dir_all(&zip_path) { // Match executes print statement if error returned
            Ok(_) => {}, // No action needed for Ok, so it's an empty block
            Err(e) => println!("ERROR: Failed to create directory: {}", e), // Inform the user of the error
        }
    }

    let path = Path::new(target_dir_path);

    let mut counter = 0;
    // Iterate over the entries in the target_directory
    for entry in fs::read_dir(path)? { 
        let entry = entry?; // Question marks used to propagate errors upwards
        let path = entry.path();
        let file_name = entry.file_name().into_string().unwrap();

        // Check if the entry is the "zipped" directory and skip it
        if entry.file_name() == OsStr::new("zipped") {
            continue;
        }

        // Determine the zip file name
        let zip_file_name = format!("{}.zip", file_name.clone());
        let zip_file_path = zip_path.join(&zip_file_name);
        
        if path.is_dir() {

        // Create a new zip writer for each file/directory
        let file = File::create(zip_file_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o755)
            .large_file(true); // Enable ZIP64 extensions

            // Add directory contents to the ZIP file
            for dir_entry in WalkDir::new(&path).min_depth(1).into_iter().filter_map(|e| e.ok()) {
                let dir_path = dir_entry.path();
                let name_in_zip = dir_path.strip_prefix(&path).unwrap().to_str().unwrap();
                if dir_path.is_file() {
                    let mut f = File::open(dir_path)?;
                    zip.start_file(name_in_zip, options)?;
                    io::copy(&mut f, &mut zip)?;
                } else if dir_path.is_dir() {
                    zip.add_directory(name_in_zip, options)?;
                }
            }
            counter += 1;
            let time_elapsed = start.elapsed();
            println!("Zipped: {} ({} of {}). Time elapsed: {:.2?}", file_name, counter, total_to_zip, time_elapsed);
            zip.finish()?;
        } else if zip_files && path.is_file() {

        // Create a new zip writer for each file/directory
        let file = File::create(zip_file_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o755)
            .large_file(true); // Enable ZIP64 extensions

            // Add file to the ZIP file
            let mut f = File::open(&path)?;
            zip.start_file(file_name.clone(), options)?;
            io::copy(&mut f, &mut zip)?;
            counter += 1;
            let time_elapsed = start.elapsed();
            println!("Zipped: {} ({} of {}). Time elapsed: {:.2?}", file_name, counter, total_to_zip, time_elapsed);
            zip.finish()?;
        }      
    }

    Ok(())
}

fn count_items_in_directory(target_directory: &str, zip_files: bool) -> std::io::Result<(usize, usize)> {
    // Convert the target_directory string to a Path
    let path = Path::new(target_directory);
    let mut file_count = 0;
    let mut dir_count = 0;

    let entries = fs::read_dir(path)?;
    // Iterate over the entries in the target_directory
    for entry_result in entries { 
        let entry = entry_result?; // Question marks used to propagate errors upwards
        let entry_path = entry.path();
        
        // Check if the entry is the "zipped" directory and skip it
        if entry.file_name() == OsStr::new("zipped") {
            continue;
        }
        
        // Check if the entry is a file or directory
        if entry_path.is_dir() {
            dir_count += 1;
        } else if zip_files && entry_path.is_file() {
            // If zip_files is true, count files as well
            file_count += 1;
        }
    }

    Ok((file_count, dir_count))
}