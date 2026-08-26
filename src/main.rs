use std::{
    env, fs,
    io::{self},
};

use r5d::{get_files, is_directory, search_reminders};

const ANSI_RED: &str = "\u{001b}[31m";
const ANSI_YELLOW: &str = "\u{001b}[33m";
const ANSI_GREEN: &str = "\u{001b}[32m";
const ANSI_RESET: &str = "\u{001b}[0m";
/* Program configuration */
struct Config {
    paths: Vec<String>,    // files and directories that should be processed
    recursive: bool,       // search directories recursively
    ignore_binary: bool,   // Ignore UTF-8 encoding errors (i.e. binary files)
    ignore_noremind: bool, // Ignore the !noremind flag in file
    show_all: bool,        // Show all reminders
}

impl Config {
    fn new() -> Config {
        Config {
            paths: Vec::new(),
            recursive: false,
            ignore_binary: false,
            ignore_noremind: false,
            show_all: false,
        }
    }
}

fn main() {
    let mut config = Config::new();
    parse_arguments(&mut config);

    let mut total_dues = 0;
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
        let dues = process(
            &filename,
            config.ignore_binary,
            config.ignore_noremind,
            config.show_all,
        );

        if dues < 0 {
            // Ignore
            continue;
        } else if dues == 0 {
            eprintln!("{filename} - no due reminders");
        } else if dues == 1 {
            eprintln!("{filename} - one due reminder");
        } else {
            eprintln!("{filename} - {dues} due reminders");
        }

        total_dues += dues;
    }

    if filenames.len() > 1 {
        if total_dues == 1 {
            eprintln!("One due reminder in {} files", filenames.len())
        } else {
            eprintln!("{total_dues} due reminders in {} files", filenames.len())
        }
    }

    // Return with exit code 5 if we found reminders
    if total_dues > 0 {
        std::process::exit(5);
    }
}

fn process(filename: &str, ignore_binary: bool, ignore_noremind: bool, show_all: bool) -> i32 {
    let mut rings = 0;
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => {
            if ignore_binary && err.kind() == io::ErrorKind::InvalidData {
                eprintln!("{filename} is not valid UTF-8");
                return -1;
            } else {
                eprintln!("error reading file {filename}");
                std::process::exit(1);
            }
        }
    };
    let reminders = match search_reminders(&contents, ignore_noremind) {
        Ok(reminders) => reminders,
        Err(err) => {
            eprintln!("(!!) error in {filename}: {:#?}", err);
            std::process::exit(1);
        }
    };

    for reminder in reminders {
        let due = reminder.is_due();
        if show_all || due {
            if due {
                print!("{}", ANSI_RED);
                println!("Due@{}:{}", filename, reminder.line);
                print!("{}", ANSI_YELLOW);
                println!("  Due:         {}", reminder.due_fmt());
                rings += 1;
            } else {
                print!("{}", ANSI_GREEN);
                println!("Pending@{}:{}", filename, reminder.line);
                println!("  Due:         {}", reminder.due_fmt());
                print!("{}", ANSI_RESET);
            }

            print!("{}", ANSI_RESET);
            println!("  Description: {}", reminder.description);
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
    println!("  -a, --all                          Show all reminders");
    println!("  -b, --ignore-binary                Ignore UTF-8 reading error, i.e. binary files");
    println!(
        "  -n, --ignore-noremind              Ignore the !noremind flag, i.e. always show all reminders"
    );
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
                "all" | "-a" => config.show_all = true,
                "--ignore-noremind" | "-n" => config.ignore_noremind = true,
                "--ignore-binary" | "-b" => config.ignore_binary = true,
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
