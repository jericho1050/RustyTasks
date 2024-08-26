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
    // Get the command-line arguments.
    let CommandLineArgs {
        action,
        journal_file,
    } = CommandLineArgs::parse();

    // Unpack the journal file.
    let journal_file = journal_file
        .or_else(find_default_journal_file)
        .ok_or(anyhow!("Failed to find journal file."))?;

    // Perform the action.
    match action {
        Add { task, due_date } => {
            let mut new_task = Task::new(task, due_date)?;

            // Prompt the user for the priority.
            let valid_priorities = ["low", "medium", "high"];
            let priority = loop {
                println!("Enter the priority for the task (Low, Medium, High): ");
                let mut priority = String::new();
                io::stdin().read_line(&mut priority)?;
                let priority = priority.trim().to_lowercase(); // Convert to lowercase

                if valid_priorities.contains(&priority.as_str()) {
                    break priority;
                } else {
                    println!("Invalid priority. Please enter 'Low', 'Medium', or 'High'.");
                }
            };

            thread::sleep(Duration::from_secs(1));

            let mut category = String::new();
            println!("Enter the category for the task: ");
            io::stdin().read_line(&mut category)?;

            // Update the task with the priority.
            new_task.priority = Some(priority);
            new_task.category = Some(category);

            tasks::add_task(journal_file, new_task)
        }
        List { category, sort_order } => tasks::list_tasks(journal_file, category, sort_order),
        Done { position } => tasks::complete_task(journal_file, position),
    }?;
    Ok(())
}