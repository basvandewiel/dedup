use clap::{App, Arg};
use rusqlite::Connection;
use std::fs::File;
use std::io::{Read, BufRead, BufReader, stdin};
use std::sync::{Arc, Mutex};
use std::thread;

use num_cpus;
use sha2::{Digest, Sha256};

fn main() {
    // Define the command line arguments using clap
    let matches = App::new("checksum")
        .arg(
            Arg::with_name("input")
                .value_name("FILE")
                .help("The input file containing the paths to the files to process. If not specified, reads from stdin"),
        )
        .get_matches();

    // Get the value of the "input" argument
    let input_file = matches.value_of("input").unwrap_or("-");

    // Open a connection to the database file
    let conn = Arc::new(Mutex::new(
        Connection::open("checksums.db").expect("Failed to create database"),
    ));

    // Create the table to store the checksums and file paths
    conn.lock()
        .expect("Failed to acquire mutex")
        .execute(
            "CREATE TABLE IF NOT EXISTS files (hash TEXT NOT NULL, path TEXT NOT NULL)",
            [],
        )
        .expect("Failed to create table");

    // Read the input file(s) containing the paths to the files to process
    let reader: Box<dyn BufRead> = if input_file == "-" {
        Box::new(BufReader::new(stdin()))
    } else {
        Box::new(BufReader::new(
            File::open(input_file).expect("Failed to open input file"),
        ))
    };

    // Determine the number of worker threads to use
    let num_threads = num_cpus::get();

    // Create a vector to hold the worker threads
    let mut workers = Vec::new();

    // Spawn a worker thread for each line in the input file, up to the maximum number of threads
    for line in reader.lines() {
        let path = line.expect("Failed to read line");

        // Spawn the worker thread
        let conn = Arc::clone(&conn);
        let handle = thread::spawn(move || {
            let mut hasher = Sha256::new();
            let file = File::open(&path).expect("Failed to open file");
            let mut reader = BufReader::new(file);
            let mut buffer = [0; 1024];
            loop {
                let count = reader.read(&mut buffer).expect("Failed to read file");
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            let hash = hex::encode(hasher.finalize());

            conn.lock()
                .expect("Failed to acquire mutex")
                .execute(
                    "INSERT INTO files (hash, path) VALUES (?1, ?2)",
                    &[&hash, &path],
                )
                .expect("Failed to insert row");
        });

        workers.push(handle);

        // If we have spawned the maximum number of worker threads, wait for one of them to finish before spawning the next
        if workers.len() == num_threads {
            workers.remove(0).join().unwrap();
        }
    }

    // Wait for all remaining worker threads to finish
    for handle in workers {
        handle.join().unwrap();
    }
}
