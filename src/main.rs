use std::{env, fs};

use r5d::search_reminders;

fn main() {
    let mut args = env::args_os();
    args.next(); // Ignore program name

    let mut rings = 0;

    for file in args {
        rings += process(file.to_str().expect("argument parsing error"));
    }

    // Return with exit code 5 if we found reminders
    if rings > 0 {
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
            let reminder = if reminder.reminder == "" {
                "<empty reminder>"
            } else {
                &reminder.reminder
            };
            println!("{filename}:{line} {reminder}");
            rings += 1;
        }
    }

    return rings;
}
