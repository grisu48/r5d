use std::{
    fs::{self},
    io,
};

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

// Date formats that will be tried to be parsed in this order
const DATETIME_FORMATS: [&str; 2] = ["%Y-%m-%dT%H:%M:%S%z", "%Y-%m-%dT%H:%M:%S"];
const DATE_FORMATS: [&str; 1] = ["%Y-%m-%d"];

const ANSI_RED: &str = "\u{001b}[31m";
const ANSI_YELLOW: &str = "\u{001b}[33m";
const ANSI_GREEN: &str = "\u{001b}[32m";
const ANSI_WHITE: &str = "\u{001b}[37m";
const ANSI_RESET: &str = "\u{001b}[0m";

#[derive(Debug)]
pub enum ResultError {
    SyntaxError,
    DateformatError,
    IOError(io::Error),
}

impl From<io::Error> for ResultError {
    fn from(err: io::Error) -> Self {
        ResultError::IOError(err)
    }
}

#[derive(Debug)]
pub struct Reminder {
    pub datetime: DateTime<Utc>, // Time the reminder is set to, if present
    pub description: String,     // Reminder string if present
    pub filename: String,        // File containing the reminder
    pub line: i32,               // Line number that matched
}

impl Reminder {
    pub fn new() -> Reminder {
        Reminder {
            datetime: Utc::now(),
            description: "".to_string(),
            filename: "".to_string(),
            line: 0,
        }
    }

    // Parse the given string value and return a reminder if valid
    fn create(value: &str) -> Result<Reminder, ResultError> {
        let mut reminder = Reminder::new();
        // Allow empty reminders
        if value.is_empty() {
            return Ok(reminder);
        }

        // Parse date and reminder description
        let (date, description) = match value.split_once(" ") {
            None => (value, ""),
            Some((date, description)) => (date, description),
        };
        let datetime = match parse_datetime(date) {
            Some(datetime) => datetime,
            None => return Err(ResultError::DateformatError),
        };

        reminder.datetime = datetime;
        reminder.description = description.to_string();
        Ok(reminder)
    }

    // Checks if the given reminder is due
    pub fn is_due(&self) -> bool {
        let now = Utc::now().timestamp();
        return self.datetime.timestamp() <= now;
    }

    // Create string representation of the due date
    pub fn due_fmt(&self) -> String {
        let now = Utc::now();
        let mut ret = self.datetime.format("%Y-%m-%d").to_string();
        let diff = now - self.datetime;
        let mut days = diff.num_days();
        if days == 0 {
            // Check if now
            if (self.datetime.timestamp() - now.timestamp()).abs() < 1 {
                ret = "now".to_string();
            } else {
                ret = format!("Today at {}", self.datetime.format("%H:%M:%S"));
            }
        } else if days > 0 {
            ret.push_str(format!(" ({days} days ago)").as_str());
        } else {
            days = -days;
            ret.push_str(format!(" (in {days} days)").as_str());
        }
        ret
    }

    pub fn print(&self) {
        if self.is_due() {
            print!("{}", ANSI_RED);
            print!("Due:{}:{}", self.filename, self.line);
            print!(" {}{}", ANSI_YELLOW, self.due_fmt());
        } else {
            print!("{}", ANSI_GREEN);
            print!("Ok:{}:{}", self.filename, self.line);
            print!(" {}{}", ANSI_WHITE, self.due_fmt());
        }
        println!("{}", ANSI_RESET);
        println!("  {}", self.description);
    }
}

/* Attempts to parse a given datetime string by applying various matching patterns. */
fn parse_datetime(str: &str) -> Option<DateTime<Utc>> {
    // Special handles come first
    if str == "" || str == "now" || str == "_" || str == "-" {
        return Some(Utc::now());
    }

    // RFC3339 has preference over custom date formats
    if let Ok(date) = DateTime::parse_from_rfc3339(str) {
        return Some(date.to_utc());
    }

    // Try to parse to the given datetime and only date formats
    for fmt in DATETIME_FORMATS {
        if let Ok(date) = NaiveDateTime::parse_from_str(str, fmt) {
            return Some(date.and_utc());
        }
    }
    for fmt in DATE_FORMATS {
        if let Ok(date) = NaiveDate::parse_from_str(str, fmt) {
            return Some(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
    }

    return None;
}

/* Check if a given pathname is a directory */
pub fn is_directory(pathname: &str) -> bool {
    let metadata = match fs::metadata(pathname) {
        Ok(metadata) => metadata,
        Err(_) => {
            return false;
        }
    };
    return metadata.is_dir();
}

// Get all files from a given path.
pub fn get_files(pathname: &str, recursive: bool) -> Result<Vec<String>, io::Error> {
    let mut ret: Vec<String> = Vec::new();

    let contents = fs::read_dir(pathname)?;
    for entry in contents {
        let entry = entry?;
        let path = entry.path();
        let fullpath = match path.to_str() {
            Some(path) => path,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Path encoding error",
                ));
            }
        };

        if is_directory(fullpath) {
            if recursive {
                let mut subcontents = get_files(fullpath, recursive)?;
                ret.append(&mut subcontents);
            }
        } else {
            ret.push(fullpath.to_string());
        }
    }

    return Ok(ret);
}

/* Searches for reminders in the given file */
pub fn file_search_reminders(
    filename: &str,
    ignore_noremind: bool,
) -> Result<Vec<Reminder>, ResultError> {
    let contents = fs::read_to_string(filename)?;
    let mut reminders = search_reminders(&contents, ignore_noremind)?;
    for reminder in reminders.iter_mut() {
        reminder.filename = filename.to_string();
    }
    Ok(reminders)
}

/* Searches for reminders in the given string */
pub fn search_reminders(
    content: &str,
    ignore_noremind: bool,
) -> Result<Vec<Reminder>, ResultError> {
    let mut reminders: Vec<Reminder> = Vec::new();
    let mut line_counter = 0;
    for line in content.lines() {
        line_counter += 1;

        if line.contains("!noremind") && !ignore_noremind {
            break;
        }

        if let Some(matched) = line.find("!remind") {
            let matched = line[matched + 7..].trim();

            let mut reminder = match Reminder::create(matched) {
                Ok(reminder) => reminder,
                Err(err) => {
                    return Err(err);
                }
            };
            reminder.line = line_counter;
            reminders.push(reminder);
        } else if let Some(matched) = line.find("!todo") {
            let matched = line[matched + 5..].trim();

            let mut reminder = match Reminder::create(matched) {
                Ok(reminder) => reminder,
                Err(err) => {
                    return Err(err);
                }
            };
            reminder.line = line_counter;
            reminders.push(reminder);
        }
    }
    Ok(reminders)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_datetime() {
        // Note: For the tests we accept a difference of 10 seconds.
        // Because we also parse only dates, we need to set the timestamp to midnight
        let now = Utc::now();

        // Parse special dates
        for str in ["now", "", "-", "_"] {
            assert!(
                parse_datetime(str)
                    .expect(&format!("parsing of {str} failed"))
                    .timestamp()
                    - now.timestamp()
                    < 10
            );
        }

        // Parse RFC3339
        assert_eq!(
            parse_datetime(&now.to_rfc3339()).expect("RFC3339 parsing failed"),
            now
        );

        // Parse custom formats
        let mut formats = Vec::new();
        formats.extend_from_slice(&DATE_FORMATS);
        formats.extend_from_slice(&DATETIME_FORMATS);
        for fmt in formats {
            let formatted = now.format(fmt).to_string();

            // Note: Do not use assert_eq because it's too accurate.
            let parsed = match parse_datetime(&formatted) {
                Some(t) => t,
                None => {
                    eprintln!("Parsing of '{}' failed: None returned", formatted);
                    assert!(false);
                    continue;
                }
            };
            assert_eq!(
                formatted,
                parsed.format(fmt).to_string(),
                "parsed date doesn't match input date"
            );
        }
    }

    #[test]
    fn test_search_reminders() {
        let now = Utc::now().fixed_offset().timestamp();

        // Should not find any reminders
        assert!(search_reminders("123", false).unwrap().len() == 0);
        // Should find one empty reminder
        assert!(search_reminders("123\n!remind\n456", false).unwrap().len() == 1);
        // Should find one empty reminder
        assert!(search_reminders("123\n!todo\n456", false).unwrap().len() == 1);
        // Should find one non-empty reminder
        let reminders = search_reminders("123\n# !remind now Hello World!\n", false).unwrap();
        assert!(reminders.len() == 1);
        assert!(reminders[0].datetime.timestamp() - now < 10);
        assert!(reminders[0].description == "Hello World!");
        assert!(reminders[0].is_due());
        // Check if !todo works with a date only
        let reminders =
            search_reminders("123\n# !todo 2024-01-01 Happy new year!\n", false).unwrap();
        assert!(reminders.len() == 1);
        assert!(reminders[0].datetime.timestamp() == 1704067200);
        assert!(reminders[0].description == "Happy new year!");
        assert!(reminders[0].is_due());
        // Check if !todo works with a datetime
        let reminders =
            search_reminders("123\n# !todo 2024-01-01T10:00:00Z Happy new year!\n", false).unwrap();
        assert!(reminders.len() == 1);
        assert!(reminders[0].datetime.timestamp() == 1704103200);
        assert!(reminders[0].description == "Happy new year!");
        assert!(reminders[0].is_due());
        // Check if !remind works with a datetime
        let reminders = search_reminders(
            "123\n# !remind 2150-12-31T23:59:59Z Happy new year!\n",
            false,
        )
        .unwrap();
        assert!(reminders.len() == 1);
        assert!(reminders[0].datetime.timestamp() == 5711817599);
        assert!(reminders[0].description == "Happy new year!");
        assert!(!reminders[0].is_due()); // if you see this failing in 2150 then I hope humankind is doing fine and we have reached for the stars :-)
    }

    #[test]
    fn test_no_reminders() {
        // Should find two empty reminders and ignore the rest
        assert!(
            search_reminders("!remind\n!remind\n!noremind\n!remind\n!remind", false)
                .unwrap()
                .len()
                == 2
        );
        // Same but with todo
        assert!(
            search_reminders("!todo\n!todo\n!noremind\n!todo\n!todo", false)
                .unwrap()
                .len()
                == 2
        );

        // test the ignore_noremind
        assert!(
            search_reminders("!todo\n!todo\n!noremind\n!todo\n!todo", true)
                .unwrap()
                .len()
                == 4
        );
    }
}
