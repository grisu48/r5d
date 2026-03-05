use std::{env, fs, io};

use r5d::{get_files, is_directory, search_reminders};

/* Program configuration */
struct Config {
    paths: Vec<String>, // files and directories that should be processed
    recursive: bool,
}

impl Config {
    fn new() -> Config {
        Config {
            paths: Vec::new(),
            recursive: false,
        }
    }
}

fn main() {
    let mut config = Config::new();
    parse_arguments(&mut config);

    let mut total_matches = 0;
    let pathnames = config.paths;
    let mut filenames: Vec<String> = Vec::new();

    for pathname in pathnames {
        let mut files = match expand_path(&pathname, config.recursive) {
            Ok(files) => files,
            Err(err) => {
                eprintln!("path expansion error for {pathname}: {err}");
                std::process::exit(1);
            }
        };
        filenames.append(&mut files);
    }

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
    let contents =
        fs::read_to_string(filename).expect(format!("error reading file {filename}").as_str());
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
    println!("Usage: {progname} [OPTIONS] FILES...");
    println!("");
    println!("OPTIONS");
    println!("  -h, --help                         Show this help message");
    println!("  -r, --recursive                    Recursive search in directories");
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
                "--recursive" | "-r" => config.recursive = true,
                _ => {
                    eprintln!("invalid argument: {arg}");
                    std::process::exit(1);
                }
            }
        } else {
            config.paths.push(arg);
        }
    }

    if config.paths.is_empty() {
        usage();
        std::process::exit(1);
    }
}

/* Get filename if this is a file or the contents of the directory if it is a directory */
fn expand_path(pathname: &str, recursive: bool) -> Result<Vec<String>, io::Error> {
    if is_directory(pathname) {
        return get_files(pathname, recursive);
    } else {
        let mut ret: Vec<String> = Vec::new();
        ret.push(pathname.to_string());
        return Ok(ret);
    }
}
