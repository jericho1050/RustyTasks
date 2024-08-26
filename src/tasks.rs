use chrono::TimeZone;
use chrono::{
    serde::ts_seconds, serde::ts_seconds_option, DateTime, Datelike, Local, NaiveDate, Utc,
};
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    #[serde(default)]
    pub id: usize,
    pub text: String,

    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,

    #[serde(with = "ts_seconds_option")]
    pub due_date: Option<DateTime<Utc>>,

    pub priority: Option<String>,
    pub category: Option<String>,
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
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "Invalid date format. Use YYYY-MM-DD format.",
                    ));
                }
            }
            None => None,
        };
        Ok(Task {
            id: 0,
            text,
            created_at,
            due_date,
            priority: None, // Initialize priority as None
            category: None, // Initialize category as None
                            // we'll just fill this later
        })
    }

    fn priority_order(&self) -> u8 {
        match self.priority.as_deref() {
            Some("high") => 1,
            Some("medium") => 2,
            Some("low") => 3,
            _ => 4,
        }
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority_order().cmp(&other.priority_order())
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority_order() == other.priority_order()
    }
}

impl Eq for Task {}

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

pub fn add_task(journal_path: PathBuf, mut task: Task) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(journal_path)?;
    let mut tasks = collect_tasks(&file)?;

    // Assign an id to the new task.
    task.id = tasks.len() + 1;
    tasks.push(task);

    // Sort the tasks by their priority.
    tasks.sort();
    // Write the updated tasks back to the file.
    file.set_len(0)?;
    serde_json::to_writer_pretty(file, &tasks)?;
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

    // Sort the tasks by their priority.
    tasks.sort();
    // Write the modified task list back into the file.
    file.set_len(0)?;
    serde_json::to_writer_pretty(file, &tasks)?;
    Ok(())
}

pub fn list_tasks(
    journal_path: PathBuf,
    category: Option<String>,
    sort_order: String,
) -> Result<()> {
    // Open the file.
    let file = OpenOptions::new().read(true).open(journal_path)?;
    // Parse the file and collect the tasks.
    let mut tasks = collect_tasks(&file)?;

    // Sort tasks based on the sort_order parameter.
    match sort_order.as_str() {
        "desc" => tasks.sort_by(|a, b| b.cmp(a)),
        _ => tasks.sort_by(|a, b| a.cmp(b)),
    }

    // Enumerate and display tasks, if any.
    if tasks.is_empty() {
        println!("Task list is empty!");
    } else {
        // Print the headers.
        println!(
            "{:<5} {:<50} {:<20} {:<20} {:<25} {:<25}",
            "ID", "Task", "Created At", "Due Date", "Priority", "Category"
        );
        let mut order: u32 = 1;
        for task in tasks {
            if category
                .as_ref()
                .map_or(true, |c| task.category.as_ref() == Some(c))
            {
                println!("{}: {}", order, task);
                order += 1;
            }
        }
    }

    Ok(())
}

pub fn search_tasks(journal_path: PathBuf, keyword: String) -> Result<()> {
    // Open the file.
    let file = OpenOptions::new().read(true).open(journal_path)?;
    // Parse the file and collect the tasks.
    let tasks = collect_tasks(&file)?;

    // Filter tasks based on the keyword.
    let filtered_tasks: Vec<&Task> = tasks
        .iter()
        .filter(|task| task.text.contains(&keyword))
        .collect();

    // Enumerate and display tasks, if any.
    if filtered_tasks.is_empty() {
        println!("No tasks found with the keyword '{}'", keyword);
    } else {
        // Print the headers.
        println!(
            "{:<5} {:<50} {:<20} {:<20} {:<25} {:<25}",
            "ID", "Task", "Created At", "Due Date", "Priority", "Category"
        );
        for task in filtered_tasks {
            println!("{}", task);
        }
    }

    Ok(())
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created_at = self.created_at.with_timezone(&Local).format("%F %H:%M");
        let due_date = self.due_date.map_or("".to_string(), |d| {
            d.with_timezone(&Local).format("%F").to_string()
        });
        write!(
            f,
            "{:<50} {:<20} {:^15} {:^25} {:^25}",
            self.text,
            created_at,
            due_date,
            self.priority.as_ref().unwrap_or(&"".to_string()),
            self.category.as_ref().unwrap_or(&"".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::remove_file, io::Write};

    use super::*;

    #[test]
    fn test_new_task_with_due_date() {
        let text = String::from("Finish project");
        let due_date = Some(String::from("2022-12-31"));
        let task = Task::new(text, due_date).unwrap();

        assert_eq!(task.text, "Finish project");
        assert_eq!(task.due_date.unwrap().year(), 2022);
        assert_eq!(task.due_date.unwrap().month(), 12);
        assert_eq!(task.due_date.unwrap().day(), 31);
    }

    #[test]
    fn test_new_task_without_due_date() {
        let text = String::from("Buy groceries");
        let task = Task::new(text, None).unwrap();

        assert_eq!(task.text, "Buy groceries");
        assert_eq!(task.due_date, None);
    }

    #[test]
    fn test_priority_order_high() {
        let task = Task {
            id: 1,
            text: String::from("Task 1"),
            created_at: Utc::now(),
            due_date: None,
            priority: Some(String::from("high")),
            category: None,
        };

        assert_eq!(task.priority_order(), 1);
    }

    #[test]
    fn test_priority_order_medium() {
        let task = Task {
            id: 2,
            text: String::from("Task 2"),
            created_at: Utc::now(),
            due_date: None,
            priority: Some(String::from("medium")),
            category: None,
        };

        assert_eq!(task.priority_order(), 2);
    }

    #[test]
    fn test_priority_order_low() {
        let task = Task {
            id: 3,
            text: String::from("Task 3"),
            created_at: Utc::now(),
            due_date: None,
            priority: Some(String::from("low")),
            category: None,
        };

        assert_eq!(task.priority_order(), 3);
    }

    #[test]
    fn test_priority_order_default() {
        let task = Task {
            id: 4,
            text: String::from("Task 4"),
            created_at: Utc::now(),
            due_date: None,
            priority: None,
            category: None,
        };

        assert_eq!(task.priority_order(), 4);
    }

    #[test]
    fn test_collect_tasks() {
        let task = Task {
            id: 1,
            text: String::from("Test Task"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("I don't know")),
            priority: Some(String::from("I don't know either")),
        };

        // Serialize the task to JSON as an array
        let tasks = vec![task];

        // Define the path for the temporary JSON file
        let path = std::path::Path::new("temp_task.json");

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.set_len(0).unwrap();
        let _ = serde_json::to_writer_pretty(&file, &tasks);

        // Perform your test here
        // For example, read the file and deserialize it back to a Task struct
        let collected_task = collect_tasks(&file).unwrap();

        assert_eq!(collected_task[0], tasks[0]);

        // tear_down
        remove_file(&path).unwrap();
    }

    #[test]
    fn test_add_task() -> Result<()> {
        // Define the path for the temporary JSON file
        let path = PathBuf::from("temp_journal_add_task.json");

        // Create an empty file for testing
        let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
        file.set_len(0)?;
        file.write_all(b"[]")?;

        // Create a new task
        let new_task = Task {
            id: 0, // This will be set by add_task
            text: String::from("Test Task"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("Test Category")),
            priority: Some(String::from("high")),
        };

        // Call the add_task function
        add_task(path.clone(), new_task.clone())?;

        // Read the file and verify the task was added
        let file = File::open(&path)?;
        let tasks: Vec<Task> = serde_json::from_reader(file)?;

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].text, new_task.text);
        assert_eq!(tasks[0].category, new_task.category);
        assert_eq!(tasks[0].priority, new_task.priority);

        // Clean up the temporary file
        remove_file(&path)?;

        Ok(())
    }

    #[test]
    fn test_complete_task() -> Result<()> {
        // Define the path for the temporary JSON file
        let path = PathBuf::from("temp_journal_complete_task.json");

        // Create some tasks
        let task1 = Task {
            id: 1,
            text: String::from("Task 1"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("Category 1")),
            priority: Some(String::from("high")),
        };

        let task2 = Task {
            id: 2,
            text: String::from("Task 2"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("Category 2")),
            priority: Some(String::from("medium")),
        };

        let tasks = vec![task1.clone(), task2.clone()];

        // Serialize the tasks to JSON and write to the file
        let json = serde_json::to_string(&tasks)?;
        let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
        file.set_len(0)?;
        file.write_all(json.as_bytes())?;

        // Call the complete_task function to remove the first task
        complete_task(path.clone(), 1)?;

        // Read the file and verify the task was removed
        let file = File::open(&path)?;
        let remaining_tasks: Vec<Task> = serde_json::from_reader(file)?;

        assert_eq!(remaining_tasks.len(), 1);
        assert_eq!(remaining_tasks[0].text, task2.text);
        assert_eq!(remaining_tasks[0].category, task2.category);
        assert_eq!(remaining_tasks[0].priority, task2.priority);

        // Clean up the temporary file
        remove_file(&path)?;

        Ok(())
    }

    #[test]
    fn test_list_tasks_returns_ok() -> Result<()> {
        // Define the path for the temporary JSON file
        let path = PathBuf::from("temp_journal_list_tasks.json");

        // Create some tasks
        let task1 = Task {
            id: 1,
            text: String::from("Task 1"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("Category 1")),
            priority: Some(String::from("high")),
        };

        let task2 = Task {
            id: 2,
            text: String::from("Task 2"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("Category 2")),
            priority: Some(String::from("medium")),
        };

        let tasks = vec![task1.clone(), task2.clone()];

        // Serialize the tasks to JSON and write to the file
        let json = serde_json::to_string(&tasks)?;
        let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
        file.set_len(0)?;
        file.write_all(json.as_bytes())?;

        // Call the list_tasks function and assert it returns Ok(())
        let result = list_tasks(
            path.clone(),
            Some(String::from("Category 1")),
            String::from("asc"),
        );
        assert!(result.is_ok());

        // Clean up the temporary file
        remove_file(&path)?;

        Ok(())
    }
    #[test]
    fn test_search_tasks_returns_ok() -> Result<()> {
        // Define the path for the temporary JSON file
        let path = PathBuf::from("temp_journal_search_tasks.json");

        // Create some tasks
        let task1 = Task {
            id: 1,
            text: String::from("Task 1 with keyword"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("Category 1")),
            priority: Some(String::from("high")),
        };

        let task2 = Task {
            id: 2,
            text: String::from("Task 2 without keyword"),
            created_at: Utc::now(),
            due_date: Some(Utc::now() + chrono::Duration::days(7)),
            category: Some(String::from("Category 2")),
            priority: Some(String::from("medium")),
        };

        let tasks = vec![task1.clone(), task2.clone()];

        // Serialize the tasks to JSON and write to the file
        let json = serde_json::to_string(&tasks)?;
        let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
        file.set_len(0)?;
        file.write_all(json.as_bytes())?;

        // Call the search_tasks function and assert it returns Ok(())
        let result = search_tasks(path.clone(), String::from("keyword"));
        assert!(result.is_ok());

        // Clean up the temporary file
        remove_file(&path)?;

        Ok(())
    }
}
