use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "Rusty Journal", about = "A command line to-do app written in Rust")]
pub struct CommandLineArgs {
    #[command(subcommand)]
    pub action: Action,

    /// Use a different journal file.
    #[arg(short, long, value_name = "FILE")]
    pub journal_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Write tasks to the journal file.
    /// OPTIONS:
    ///     [--due-date "yyyy-mm-dd"]
    Add {
        /// The task description text.
        #[arg()]
        task: String,

        /// The due date for the task (optional).
        #[arg(short, long)]
        due_date: Option<String>,
    },
    /// Remove an entry from the journal file by position.
    Done {
        #[arg()]
        position: usize,
    },
    /// List all tasks in the journal file.
    List {
        /// The category to filter tasks by (optional).
        #[arg(short, long)]
        category: Option<String>,

        /// The sort order for the tasks (optional).
        #[arg(short, long, default_value = "asc")]
        sort_order: String,
    },
    /// Search for tasks by keyword.
    Search {
        /// The keyword to search for.
        #[arg()]
        keyword: String,
    },
}