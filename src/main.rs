use std::{env, fs};

use r5d::search_reminders;

/* Program configuration */
struct Config {
    files: Vec<String>, // files that should be processed
}

impl Config {
    fn new() -> Config {
        Config { files: Vec::new() }
    }
}

fn main() {
    let mut config = Config::new();
    parse_arguments(&mut config);

    let mut total_matches = 0;
    let filenames = config.files;

    for filename in filenames.iter() {
        let matches = process(&filename);

        eprintln!("{filename} - {matches} match(es)");

        total_matches += matches;
    }

    if filenames.len() > 1 {
        eprintln!("{total_matches} match(es) in {} files", filenames.len())
    }

    // Return with exit code 5 if we found reminders
    if total_matches > 0 {
        std::process::exit(5);
    }
}

fn process(filename: &str) -> i32 {
    let mut rings = 0;
    let contents = fs::read_to_string(filename).expect("error reading file");
    let reminders = match search_reminders(&contents) {
        Ok(reminders) => reminders,
        Err(err) => {
            eprintln!("(!!) error in {filename}: {:#?}", err);
            std::process::exit(1);
        }
    };

    for reminder in reminders {
        if reminder.is_due() {
            let line = reminder.line;
            let reminder = if reminder.description == "" {
                "<empty reminder>"
            } else {
                &reminder.description
            };
            println!("{filename}:{line} {reminder}");
            rings += 1;
        }
    }

    return rings;
}

fn usage() {
    let progname = env::args_os().next().unwrap().to_str().unwrap().to_string();

    println!("r5d - remind");
    println!("  search for '!remind' statements in files and prints due reminders.");
    println!("");
    println!("Usage: {progname} FILES...");
    println!("");
    println!("");
}

fn parse_arguments(config: &mut Config) {
    let mut args = env::args_os();
    args.next(); // Ignore program name

    for arg in args {
        let arg = arg.to_str().expect("argument parsing error").to_string();
        if arg == "" {
            continue;
        } else if arg.starts_with('-') {
            // Program argument
            match arg.as_str() {
                "--help" | "-h" => {
                    usage();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("invalid argument: {arg}");
                    std::process::exit(1);
                }
            }
        } else {
            config.files.push(arg);
        }
    }
}
