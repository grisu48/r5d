use chrono::{DateTime, FixedOffset, Utc};

#[derive(Debug)]
pub enum ResultError {
    SyntaxError,
    DateformatError,
}

#[derive(Debug)]
pub struct Reminder {
    pub datetime: DateTime<FixedOffset>, // Time the reminder is set to, if present
    pub reminder: String,                // Reminder string if present
    pub line: i32,                       // Line number that matched
}

impl Reminder {
    pub fn new() -> Reminder {
        Reminder {
            datetime: Utc::now().fixed_offset(),
            reminder: "".to_string(),
            line: 0,
        }
    }

    // Checks if the given reminder is due
    pub fn is_due(&self) -> bool {
        let now = Utc::now().fixed_offset().timestamp();
        return self.datetime.timestamp() <= now;
    }
}

/* Attempts to parse a given datetime string by applying various matching patterns. */
fn parse_datetime(str: &str) -> Option<DateTime<FixedOffset>> {
    // Special handles come first
    if str == "" || str == "now" || str == "_" || str == "-" {
        return Some(Utc::now().fixed_offset());
    }

    // RFC 3339 has preference over custom date formats
    if let Ok(date) = DateTime::parse_from_rfc3339(str) {
        return Some(date);
    }

    // TODO: Add more parsing methods

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
        let now = Utc::now().fixed_offset().timestamp();

        assert!(
            parse_datetime("")
                .expect("parsing of empty string failed")
                .timestamp()
                - now
                < 10
        );
        assert!(
            parse_datetime("-")
                .expect("parsing of '-' failed")
                .timestamp()
                - now
                < 10
        );
        assert!(
            parse_datetime("_")
                .expect("parsing of '-' failed")
                .timestamp()
                - now
                < 10
        );
        assert!(
            parse_datetime("now")
                .expect("parsing of 'now' failed")
                .timestamp()
                - now
                < 10
        );
    }

    #[test]
    fn test_search_reminders() {
        // Should not find any reminders
        assert!(search_reminders("123").unwrap().len() == 0);
        // Should find one empty reminder
        assert!(search_reminders("123\n!remind\n456").unwrap().len() == 1);
        // Should find one non-empty reminder
        let reminders = search_reminders("123\n# !remind now Hello World!\n").unwrap();
        assert!(reminders.len() == 1);
        //assert!(reminders[0].datetime == "now");
        assert!(reminders[0].reminder == "Hello World!");
        assert!(reminders[0].is_due());
    }
}
