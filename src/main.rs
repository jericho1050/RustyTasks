use anyhow::anyhow;
use clap::Parser;
use std::path::PathBuf;
use std::{thread, time::Duration};
mod cli;
mod tasks;

use cli::{Action::*, CommandLineArgs};
use tasks::Task;

fn find_default_journal_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push(".rusty-journal.json");
        path
    })
}

use std::io;

fn main() -> anyhow::Result<()> {
    let CommandLineArgs {
        action,
        journal_file,
    } = CommandLineArgs::parse();

    let journal_file = journal_file
        .or_else(find_default_journal_file)
        .ok_or(anyhow!("Failed to find journal file."))?;

    match action {
        Add { task, due_date } => {
            let mut new_task = Task::new(task, due_date)?;
            new_task.priority = Some(prompt_for_priority()?);
            new_task.category = Some(prompt_for_category()?);
            tasks::add_task(journal_file, new_task)
        }
        List {
            category,
            sort_order,
        } => tasks::list_tasks(journal_file, category, sort_order),
        Done { position } => tasks::complete_task(journal_file, position),
        Search { keyword } => tasks::search_tasks(journal_file, keyword),
    }?;
    Ok(())
}

fn prompt_for_priority() -> anyhow::Result<String> {
    let valid_priorities = ["low", "medium", "high"];
    loop {
        println!("Enter the priority for the task (Low, Medium, High): ");
        let mut priority = String::new();
        io::stdin().read_line(&mut priority)?;
        let priority = priority.trim().to_lowercase();

        match valid_priorities.contains(&priority.as_str()) {
            true => {
                thread::sleep(Duration::from_millis(500));
                println!("...");
                thread::sleep(Duration::from_millis(500));
                println!(".");
                return Ok(priority);
            }
            false => {
                thread::sleep(Duration::from_millis(500));
                println!("...");
                thread::sleep(Duration::from_millis(500));
                println!(".");
                println!("Invalid priority. Please enter 'Low', 'Medium', or 'High'.")
            }
        }
    }
}

fn prompt_for_category() -> anyhow::Result<String> {
    println!("Enter the category for the task: ");
    let mut category = String::new();
    io::stdin().read_line(&mut category)?;
    thread::sleep(Duration::from_millis(500));
    println!("...");
    thread::sleep(Duration::from_millis(500));
    println!(".");
    thread::sleep(Duration::from_millis(500));
    println!("Done!");
    Ok(category.trim().to_string())
}
