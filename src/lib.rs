use chrono::{DateTime, NaiveDateTime, Utc};

// Date formats that will be tried to be parsed in this order
const DATE_FORMATS: [&str; 2] = ["%Y-%m-%dT%H:%M:%S%z", "%Y-%m-%dT%H:%M:%S"];

#[derive(Debug)]
pub enum ResultError {
    SyntaxError,
    DateformatError,
}

#[derive(Debug)]
pub struct Reminder {
    pub datetime: DateTime<Utc>, // Time the reminder is set to, if present
    pub reminder: String,        // Reminder string if present
    pub line: i32,               // Line number that matched
}

impl Reminder {
    pub fn new() -> Reminder {
        Reminder {
            datetime: Utc::now(),
            reminder: "".to_string(),
            line: 0,
        }
    }

    // Checks if the given reminder is due
    pub fn is_due(&self) -> bool {
        let now = Utc::now().timestamp();
        return self.datetime.timestamp() <= now;
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

    // Try to parse to the given date formats
    for fmt in DATE_FORMATS {
        if let Ok(date) = NaiveDateTime::parse_from_str(str, fmt) {
            return Some(date.and_utc());
        }
    }

    return None;
}

/* Searches for reminders in the given string */
pub fn search_reminders(content: &str) -> Result<Vec<Reminder>, ResultError> {
    let mut reminders: Vec<Reminder> = Vec::new();
    let mut line_counter = 0;
    for line in content.lines() {
        line_counter += 1;
        if let Some(remind) = line.find("!remind") {
            let remind = line[remind + 7..].trim();

            let mut reminder = Reminder::new();
            reminder.line = line_counter;

            // Allow empty reminders
            if remind == "" {
                reminders.push(reminder);
            } else {
                // Parse date and note
                let (date, note) = match remind.split_once(" ") {
                    None => (remind, ""),
                    Some((date, remind)) => (date, remind),
                };
                let datetime = match parse_datetime(date) {
                    Some(datetime) => datetime,
                    None => return Err(ResultError::DateformatError),
                };

                reminder.datetime = datetime;
                reminder.reminder = note.to_string();
                reminders.push(reminder);
            }
        }
    }
    Ok(reminders)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_datetime() {
        // Note: For the tests we accept a difference of 10 seconds
        let now = Utc::now();
        let timestamp = now.timestamp();

        // Parse special dates
        for str in ["now", "", "-", "_"] {
            assert!(
                parse_datetime(str)
                    .expect(&format!("parsing of {str} failed"))
                    .timestamp()
                    - timestamp
                    < 10
            );
        }

        // Parse RFC3339
        assert_eq!(
            parse_datetime(&now.to_rfc3339()).expect("RFC3339 parsing failed"),
            now
        );
        // Parse custom formats
        for fmt in DATE_FORMATS {
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
            let diff = parsed.timestamp() - now.timestamp();
            assert!(diff.abs() < 1);
        }
    }

    #[test]
    fn test_search_reminders() {
        let now = Utc::now().fixed_offset().timestamp();

        // Should not find any reminders
        assert!(search_reminders("123").unwrap().len() == 0);
        // Should find one empty reminder
        assert!(search_reminders("123\n!remind\n456").unwrap().len() == 1);
        // Should find one non-empty reminder
        let reminders = search_reminders("123\n# !remind now Hello World!\n").unwrap();
        assert!(reminders.len() == 1);
        assert!(reminders[0].datetime.timestamp() - now < 10);
        assert!(reminders[0].reminder == "Hello World!");
        assert!(reminders[0].is_due());
    }
}
