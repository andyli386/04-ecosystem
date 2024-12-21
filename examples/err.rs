use anyhow::Context;
use std::fs;
use std::mem::size_of;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Serialize json error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}

// fn main() -> Result<(), MyError> {
fn main() -> Result<(), anyhow::Error> {
    println!("size of anyhow::Error is {}", size_of::<anyhow::Error>());
    println!("size of std::io::Error is {}", size_of::<std::io::Error>());
    println!(
        "size of std::num::ParseIntError is {}",
        size_of::<std::num::ParseIntError>()
    );
    println!(
        "size of serde_json::Error is {}",
        size_of::<serde_json::Error>()
    );
    println!("size of MyError is {}", size_of::<MyError>());
    println!("size of String is {}", size_of::<String>());

    let filename = "non-existent-file.txt";
    // fs::File::open(filename).context(filename)?;
    fs::File::open(filename).with_context(|| format!("Cannot find file: {}", filename))?;
    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
