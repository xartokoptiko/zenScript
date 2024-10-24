use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::{env, fs};
use std::process;
use meval::eval_str;
use regex::Regex;

fn separate_zen_code(reader: BufReader<File>) {
    // Updated regex: it captures quoted strings (with spaces) or individual tokens
    let lines: Vec<String> = reader.lines()
        .map(|line| line.unwrap())
        .collect();

    // Updated regex: it captures quoted strings (with spaces) or individual tokens
    let re = Regex::new(r#""[^"]*"|\S+"#).unwrap(); // Matches quoted strings or sequences of non-whitespace characters

    println!("Presenting the lines as vectors");

    // First loop: Print vectors for debugging
    for (line_number, line) in lines.iter().enumerate() {
        let tokens: Vec<&str> = re
            .find_iter(line) // Use find_iter to capture entire matched groups
            .map(|m| m.as_str())
            .collect();
        println!("Line {}: {:?}", line_number, tokens);
    }

    println!("======Program start======");

    // Second loop: Compile each line
    for (_line_number, line) in lines.iter().enumerate() {
        let tokens: Vec<&str> = re
            .find_iter(line) // Use find_iter to capture entire matched groups
            .map(|m| m.as_str())
            .collect();
        compile_zen_line(tokens);
    }
}

fn compile_zen_line(line: Vec<&str>) {
    match line[0] {
        "print" => {
            // Check if the argument is a string literal (starts and ends with quotes)
            if line[1].starts_with('"') && line[1].ends_with('"') {
                // Remove the surrounding quotes
                let string_literal = &line[1][1..line[1].len() - 1];
                println!("{}", string_literal);
            }
            // Check if the argument is a math expression enclosed in parentheses
            else if line[1].starts_with('(') && line[1].ends_with(')') {
                let expression = &line[1][1..line[1].len() - 1]; // Strip parentheses
                match eval_str(expression) {
                    Ok(result) => println!("{}", result),
                    Err(err) => println!("ERROR: Could not evaluate expression '{}': {}", expression, err),
                }
            } else {
                println!("ERROR: Could not parse argument '{}'", line[1]);
            }
        },
        "if" => {
            if line[1].starts_with('(') && line[1].ends_with(')') {
                let expression = &line[1][1..line[1].len() - 1];
                println!("({})", expression);
            }
        }
        _ => {
            println!("ERROR: No command found '{}'", line[0]);
        }
    }
}

fn main() {
    // Get command line arguments as a vector
    let args: Vec<String> = env::args().collect();

    // Check if a file argument was provided
    if args.len() != 2 {
        eprintln!("Usage: zen <filename>");
        process::exit(1);
    }

    // Get the file path from arguments
    let file_path = &args[1];

    // Convert the path to an absolute path relative to current directory
    let absolute_path = match fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error resolving file path: {}", e);
            process::exit(1);
        }
    };

    // Open the file
    let file = match File::open(&absolute_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    // Create a buffered reader
    let reader = BufReader::new(file);

    // Read line by line
    separate_zen_code(reader);
}