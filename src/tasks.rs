use chrono::TimeZone;
use chrono::{serde::ts_seconds_option, serde::ts_seconds, DateTime, Local, NaiveDate, Utc};
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub text: String,

    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,

    #[serde(with = "ts_seconds_option")]
    pub due_date: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(text: String, due_date: Option<String>) -> Result<Task> {
        let created_at: DateTime<Utc> = Utc::now();
        let due_date = match due_date {
            Some(date_str) => {
                let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
                if re.is_match(&date_str) {
                    let naive_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string()))?;
                    let due_date = naive_date.and_hms_opt(0, 0, 0);
                    Some(Utc.from_utc_datetime(due_date.as_ref().unwrap()))
                } else {
                    return Err(Error::new(ErrorKind::InvalidInput, "Invalid date format. Use YYYY-MM-DD format."));
                }
            }
            None => None,
        };
        Ok(Task {
            text,
            created_at,
            due_date,
        })
    }
}

fn collect_tasks(mut file: &File) -> Result<Vec<Task>> {
    file.seek(SeekFrom::Start(0))?; // Rewind the file before.
    let tasks = match serde_json::from_reader(file) {
        Ok(tasks) => tasks,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => Err(e)?,
    };
    file.seek(SeekFrom::Start(0))?; // Rewind the file after.
    Ok(tasks)
}

pub fn add_task(journal_path: PathBuf, task: Task) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(journal_path)?;
    let mut tasks = collect_tasks(&file)?;
    tasks.push(task);
    serde_json::to_writer(file, &tasks)?;
    Ok(())
}

pub fn complete_task(journal_path: PathBuf, task_position: usize) -> Result<()> {
    // Open the file.
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(journal_path)?;

    // Consume file's contents as a vector of tasks.
    let mut tasks = collect_tasks(&file)?;

    // Try to remove the task.
    if task_position == 0 || task_position > tasks.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid Task ID"));
    }
    tasks.remove(task_position - 1);

    // Write the modified task list back into the file.
    file.set_len(0)?;
    serde_json::to_writer(file, &tasks)?;
    Ok(())
}

pub fn list_tasks(journal_path: PathBuf) -> Result<()> {
    // Open the file.
    let file = OpenOptions::new().read(true).open(journal_path)?;
    // Parse the file and collect the tasks.
    let tasks = collect_tasks(&file)?;

    // Print the headers.
    println!("{:<5} {:<50} {:<20} {:<20}", "ID", "Task", "Created At", "Due Date");

    // Enumerate and display tasks, if any.
    if tasks.is_empty() {
        println!("Task list is empty!");
    } else {
        let mut order: u32 = 1;
        for task in tasks {
            println!("{}: {}", order, task);
            order += 1;
        }
    }

    Ok(())
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created_at = self.created_at.with_timezone(&Local).format("%F %H:%M");
        let due_date = self.due_date.map_or("".to_string(), |d| d.with_timezone(&Local).format("%F").to_string());
        write!(f, "{:<50} {:<20} {:<20}", self.text, created_at, due_date)
    }
}