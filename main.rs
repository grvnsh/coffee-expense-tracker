use chrono::Local;
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::Path};

/// Tim Hortons Expense Tracker CLI
#[derive(Parser)]
#[command(name = "Tim Hortons Tracker")]
#[command(about = "Track your daily Tim Hortons expenses", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand)]
enum Commands {
    /// Add a new order
    Add {
        item: String,
        quantity: u32,
        price: f64,
        date: Option<String>,
    },
    /// View total expenses for a specific day
    DailyTotal {
        date: Option<String>,
    },
    /// Export all orders to a CSV file
    Export { filepath: String },
}

#[derive(Serialize, Deserialize, Debug)]
struct Order {
    item_name: String,
    quantity: u32,
    price: f64,
    date: String, // Format: YYYY-MM-DD
}

impl Order {
    fn total_cost(&self) -> f64 {
        self.quantity as f64 * self.price
    }
}

fn main() {
    let cli = Cli::parse();
    let conn = init_db();

    match &cli.command {
        Commands::Add {
            item,
            quantity,
            price,
            date,
        } => {
            let today = Local::now().format("%Y-%m-%d").to_string();
            let order_date = date.clone().unwrap_or(today);

            add_order(&conn, item, *quantity, *price, &order_date);
            println!("Order added: {} x{} @ ${:.2} on {}", item, quantity, price, order_date);
        }
        Commands::DailyTotal { date } => {
            let today = Local::now().format("%Y-%m-%d").to_string();
            let query_date = date.clone().unwrap_or(today);

            let total = calculate_daily_total(&conn, &query_date);
            println!("Total for {}: ${:.2}", query_date, total);
        }
        Commands::Export { filepath } => {
            export_to_csv(&conn, filepath);
            println!("Orders exported to {}", filepath);
        }
    }
}

fn init_db() -> Connection {
    let conn = Connection::open("timhortons_tracker.db").expect("Failed to connect to database.");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY,
            item_name TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            price REAL NOT NULL,
            date TEXT NOT NULL
        )",
        [],
    )
    .expect("Failed to create table.");
    conn
}

fn add_order(conn: &Connection, item: &str, quantity: u32, price: f64, date: &str) {
    conn.execute(
        "INSERT INTO orders (item_name, quantity, price, date) VALUES (?1, ?2, ?3, ?4)",
        params![item, quantity, price, date],
    )
    .expect("Failed to add order.");
}

fn calculate_daily_total(conn: &Connection, date: &str) -> f64 {
    let mut stmt = conn
        .prepare("SELECT SUM(quantity * price) FROM orders WHERE date = ?1")
        .expect("Failed to prepare statement.");
    let total: f64 = stmt
        .query_row(params![date], |row| row.get(0))
        .unwrap_or(0.0);
    total
}

fn export_to_csv(conn: &Connection, filepath: &str) {
    let mut stmt = conn
        .prepare("SELECT item_name, quantity, price, date FROM orders")
        .expect("Failed to prepare statement.");
    let orders = stmt
        .query_map([], |row| {
            Ok(Order {
                item_name: row.get(0)?,
                quantity: row.get(1)?,
                price: row.get(2)?,
                date: row.get(3)?,
            })
        })
        .expect("Failed to query orders.");

    let path = Path::new(filepath);
    let mut file = File::create(&path).expect("Failed to create file.");
    let mut wtr = csv::Writer::from_writer(&mut file);

    for order in orders {
        wtr.serialize(order.expect("Failed to serialize order."))
            .expect("Failed to write to CSV.");
    }
    wtr.flush().expect("Failed to flush CSV writer.");
}
