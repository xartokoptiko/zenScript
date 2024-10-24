use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;
use std::time::Instant;
use std::process;
use evalexpr::{eval};
use regex::Regex;

fn separate_zen_code(reader: BufReader<File>) {
    let lines: Vec<String> = reader.lines()
        .map(|line| line.unwrap())
        .filter(|line| !line.trim().is_empty()) // Filter out empty lines
        .collect();

    let re = Regex::new(r#""[^"]*"|\S+"#).unwrap(); // Matches quoted strings or sequences of non-whitespace characters

    // For label tracking
    let mut labels = HashMap::new();
    let mut current_line = 0;

    #[cfg(debug_assertions)]
    println!("Presenting the lines as vectors");

    // First loop: Print vectors for debugging and collect labels
    for (line_number, line) in lines.iter().enumerate() {
        let tokens: Vec<&str> = re
            .find_iter(line) // Use find_iter to capture entire matched groups
            .map(|m| m.as_str())
            .collect();

        #[cfg(debug_assertions)]
        println!("Line {}: {:?}", line_number, tokens);

        // Check for labels
        if tokens.len() > 0 && tokens[0].ends_with(':') {
            let label = tokens[0].trim_end_matches(':');
            labels.insert(label.to_string(), line_number);
        }
    }

    #[cfg(debug_assertions)]
    println!("======Program start======");

    // Second loop: Compile each line
    while current_line < lines.len() {
        let line = &lines[current_line];
        let tokens: Vec<&str> = re
            .find_iter(line)
            .map(|m| m.as_str())
            .collect();

        // If a command is found, compile it
        if compile_zen_line(tokens, &labels, &mut current_line) {
            continue; // If a jump occurred, skip to the next iteration
        }

        current_line += 1; // Move to the next line
    }
}

fn compile_zen_line(
    line: Vec<&str>,
    labels: &HashMap<String, usize>,
    current_line: &mut usize
) -> bool {
    if line.is_empty() {
        return false; // Skip empty lines
    }

    match line[0] {
        "print" => {
            if line.len() > 1 {
                let arg = line[1];
                // Check if the argument is a string literal (starts and ends with quotes)
                if arg.starts_with('"') && arg.ends_with('"') {
                    let string_literal = &arg[1..arg.len() - 1];
                    println!("{}", string_literal);
                }
                // Check if the argument is a math expression enclosed in parentheses
                else if arg.starts_with('(') && arg.ends_with(')') && line.len() == 2 {
                    let expression = &arg[1..arg.len() - 1]; // Strip parentheses
                    match eval(expression) {
                        Ok(result) => println!("{}", result),
                        Err(err) => println!("ERROR: Could not evaluate expression '{}': {}", expression, err),
                    }
                } else {
                    println!("ERROR: Could not parse argument '{}'", arg);
                }
            }
        },
        "if" => {
            if line.len() > 1 {
                let condition = line[1];
                if condition.starts_with('(') && condition.ends_with(')') {
                    let condition_str = &condition[1..condition.len() - 1].trim(); // Strip parentheses and trim whitespace
                    match eval(condition_str) {
                        Ok(result) => {
                            // Check if result is true or false
                            if let evalexpr::Value::Boolean(is_true) = result {
                                if is_true {
                                    if line.len() > 3 && line[2] == "goto" {
                                        let label = line[3].trim_start_matches(':');
                                        if let Some(&line_num) = labels.get(label) {
                                            *current_line = line_num+1; // Jump to label line
                                            return true; // Indicate jump occurred
                                        } else {
                                            println!("ERROR: Label '{}' not found", label);
                                        }
                                    }
                                }
                            } else {
                                println!("ERROR: Expected a boolean value from condition evaluation.");
                            }
                        }
                        Err(err) => println!("ERROR: Could not evaluate condition '{}': {}", condition_str, err),
                    }
                } else {
                    println!("ERROR: Could not parse condition '{}'", condition);
                }
            }
        },
        "//" => {
            return false;
        },
        label if label.ends_with(':') => {
            // If it's a label, just return true and continue to the next iteration
            return true;
        }
        _ => {
            println!("ERROR: No command found or invalid label");
        }
    }

    // Increment current_line after processing the command
    false
}


fn main() {
    let start_time = Instant::now();

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
    let absolute_path = match std::fs::canonicalize(file_path) {
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

    let duration = start_time.elapsed();
    println!("\n\nExecution time: {:?}", duration);
}