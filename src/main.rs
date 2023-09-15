#![allow(unused)]
use std::io;
use time::Date;
use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result, params};
use rand::Rng;

#[derive(Parser)]
#[command(name = "tasky", version = "0.1.0", about = "a CLI task manager for disorganized people")]
struct Cli {
    /// Action to perform
    #[command(subcommand)]
    command: Commands,

}

#[derive(Subcommand)]
enum Commands {
    Add { name: Vec<String>, },
    Remove { id: u32, },
    End { id: u32, },
    List,
    Reset,
    Random,
    // TODO: implement sub-tasks
    Sub { 
        id: u32, 
        task: Vec<String>,
    },
}

fn name_prompt() -> String {
    let mut buf = String::new();
    while buf.trim().len() == 0 {
        println!("Enter task name:");
        io::stdin().read_line(&mut buf).unwrap();
    }
    buf.trim().to_string()
}

fn get_string(str: Vec<String>) -> String {
    if str.len() != 0 {
        return str.join(" ");
    } else {
        return name_prompt();
    }
}

fn main() -> Result<()>{
    let args = Cli::parse();

    let con = Connection::open("tasky.db")?;

    let query = "CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        completed BOOLEAN NOT NULL
    )";
    con.execute(query, [])?;

    let query = "CREATE TABLE IF NOT EXISTS subs (
        sub_id INTEGER PRIMARY KEY,
        task_id INTEGER,
        name TEXT NOT NULL,
        completed BOOLEAN NOT NULL,
        FOREIGN KEY(task_id) REFERENCES tasks(id)
    )";
    con.execute(query, [])?;

    match args.command {
        Commands::Add { name } => {
            let name = get_string(name);
            println!("Added task \"{}\"", name);

            let query = "INSERT INTO tasks (name, completed) VALUES (?1, ?2)";
            con.execute(query, params![name, false])?;
        }

        Commands::Remove { id } => {
            println!("Removing task with id {}", id);

            let query = "DELETE FROM tasks WHERE id = ?1";
            con.execute(query, params![id])?;
        }

        Commands::End { id } => {
            println!("Ending task with id: {}", id);
            let query = "UPDATE tasks SET completed = true WHERE id = ?1";
            match con.execute(query, params![id]) {
                Ok(0) => println!("No task with id {}", id),
                Ok(_) => println!("Task with id {} completed!", id),
                Err(e) => println!("Error: {}", e),
            }
        }

        Commands::List => {
            println!("Listing tasks...");
            let query = "SELECT id, name, completed FROM tasks";
            let mut stmt = con.prepare(query)?;
            let map = stmt.query_map([], |row| {
                let id: u32 = row.get(0)?;
                let name: String = row.get(1)?;
                let completed: bool = row.get(2)?;
                Ok((id, name, completed))
            })?;

            println!("ID | Name | Completed");
            for task in map.flatten() {
                println!("{} | {} | {}", task.0, task.1, task.2);
            }
        }

        Commands::Reset => {
            println!("Are you sure you want to delete all tasks? If so, type \"delete everything\"");
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            match buf.trim() {
                "delete everything" => {
                    println!("Resetting tasks");
                    let query = "DELETE FROM tasks";
                    con.execute(query, [])?;
                }
                _ => {
                    println!("Aborting reset");
                }
            }

        }

        Commands::Random => {
            println!("Picking random task...");
            let query = "SELECT id, name FROM tasks WHERE completed = false";
            let mut stmt = con.prepare(query)?;
            let map = stmt.query_map([], |row| {
                let id: u32 = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((id, name))
            })?;

            let tasks: Vec<(u32, String)> = map.flatten().collect();
            let rng = rand::thread_rng().gen_range(0..tasks.len());
            let task = &tasks[rng];
            println!("Task: {} | {}", task.0, task.1);
        }

        Commands::Sub { id, task } => {
            todo!("Subtasks are not yet implemented")
        }
    }

    Ok(())
}
