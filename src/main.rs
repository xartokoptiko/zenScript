use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;
use std::time::Instant;
use std::process;
use evalexpr::{eval, Value};
use regex::Regex;
use colored::Colorize;

fn separate_zen_code(reader: BufReader<File>, mut zen_args: Vec<i64>) {
    let mut lines: Vec<String> = reader.lines()
        .map(|line| line.unwrap())
        .filter(|line| !line.trim().is_empty()) // Filter out empty lines
        .collect();

    let re = Regex::new(r#""[^"]*"|\S+"#).unwrap(); // Matches quoted strings or sequences of non-whitespace characters

    // For label tracking
    let mut labels = HashMap::new();
    let mut current_line = 0;
    let mut variables: HashMap<String, i64> = HashMap::new();

    #[cfg(debug_assertions)]
    println!("{}", "\n\n====Presenting the lines as vectors====\n\n".cyan());

    // First loop: Print vectors for debugging and collect labels
    for (line_number, line) in lines.iter().enumerate() {
        let tokens: Vec<&str> = re
            .find_iter(line) // Use find_iter to capture entire matched groups
            .map(|m| m.as_str())
            .collect();

        #[cfg(debug_assertions)]
        println!("{} {} {:?}", "Line".yellow(), line_number.to_string().yellow(), tokens);

        // Check for labels
        if tokens.len() > 0 && tokens[0].ends_with(':') {
            let label = tokens[0].trim_end_matches(':');
            labels.insert(label.to_string(), line_number);
        }
    }

    #[cfg(debug_assertions)]
    println!("{} {:?}", "Labels found:".yellow(), labels);

    #[cfg(debug_assertions)]
    println!("{}", "\n\n======Program start======\n\n".cyan());

    // Second loop: Compile each line
    while current_line < lines.len() {

        #[cfg(debug_assertions)]
        println!("{} {} : {}", "Executing line".yellow(), current_line.to_string().yellow(), lines[current_line].to_string().yellow());

        let original_line = &lines[current_line];
        let mut modified_line = original_line.clone(); // Clone the original line for modification

        // Check for variable references and replace them
        let mut i = 0;
        while i < modified_line.len() {
            if modified_line.chars().nth(i) == Some('&') {
                // Ensure there is a next character
                if i + 1 < modified_line.len() {
                    let next_char = modified_line.chars().nth(i + 1);
                    // Check if the next character is not a space
                    if next_char != Some(' ') {
                        // Extract variable name
                        let mut var_name = String::new();
                        let mut j = i + 1;

                        // Collect the variable name until we hit a non-alphanumeric character
                        while j < modified_line.len() &&
                            (modified_line.chars().nth(j).unwrap().is_alphanumeric() || modified_line.chars().nth(j).unwrap() == '_') {
                            var_name.push(modified_line.chars().nth(j).unwrap());
                            j += 1;
                        }

                        // Check if the variable exists and replace "&<variable>" with its value
                        if let Some(value) = variables.get(&var_name) {

                            // Replace both &<variable> with its value
                            modified_line.replace_range(i..j, &value.to_string());
                        } else {
                            println!("{} '{}' ", "ERROR: Variable is not initialized".red(), var_name.red());
                            current_line += 1; // Skip to the next line
                            break; // Exit the loop since we need to continue
                        }
                    }
                }
            }
            i += 1;
        }

        // Update the original line to be the modified line for further processing
        lines[current_line] = modified_line.clone();

        // If a command is found, compile it using the modified line
        let modified_tokens: Vec<&str> = re
            .find_iter(&lines[current_line])
            .map(|m| m.as_str())
            .collect();

        if compile_zen_line(modified_tokens, &labels, &mut current_line, &mut variables, &mut zen_args) {
            #[cfg(debug_assertions)]
            println!("{} {}", "Jump occurred! New line: ".yellow(), current_line.to_string().yellow());

            continue;  // Skip the increment if we jumped
        }
        current_line += 1;
    }
}

fn compile_zen_line(
    line: Vec<&str>,
    labels: &HashMap<String, usize>,
    current_line: &mut usize,
    variables: &mut HashMap<String, i64>,
    zen_args : &mut Vec<i64>
) -> bool {
    if line.is_empty() {
        return false; // Skip empty lines
    }

    match line[0] {
        "&" => {
            if line.len() >= 4 && line[2] == "=" {
                let var_name = line[1].to_string();
                let value_str = line[3];

                let new_value = if value_str.starts_with('!') {
                    let arg_index = value_str[1..].parse::<usize>().unwrap_or(0) - 1;
                    if arg_index < zen_args.len() {
                        Value::Int(zen_args[arg_index]) // Wrap in Value::Int
                    } else {
                        println!("{} '{}' ", "ERROR: Argument index out of bounds".red(), arg_index + 1);
                        return false;
                    }
                } else if value_str.starts_with('(') && value_str.ends_with(')') {
                    let expression = &value_str[1..value_str.len() - 1];
                    eval(expression).unwrap_or(Value::Int(0)) // Eval result as Value::Int
                } else if let Ok(value) = value_str.parse::<i64>() {
                    Value::Int(value) // Wrap in Value::Int for direct numbers
                } else {
                    println!("{} '{}'", "ERROR: Invalid value for variable ".red(), var_name.to_string().red());
                    return false;
                };

                // Convert new_value to i64 if it’s an Int, or handle error if it's not
                let new_value_i64 = match new_value {
                    Value::Int(val) => val,
                    _ => {
                        println!("{} '{}'", "ERROR: Unsupported value type".red(), var_name.to_string().red());
                        return false;
                    }
                };

                variables.insert(var_name.clone(), new_value_i64);
                return false;
            } else {
                println!("{}", "ERROR: Invalid variable declaration syntax".red());
            }
        },
        "print" => {
            if line.len() > 1 {
                let arg = line[1];

                if arg.starts_with('"') && arg.ends_with('"') {
                    let string_literal = &arg[1..arg.len() - 1];

                    // Split on '\n' and print each segment with new lines
                    for (i, segment) in string_literal.split("\\n").enumerate() {
                        if i > 0 { println!(); } // Newline before each new segment except the first
                        print!("{}", segment);
                    }
                }
                // Handle math expression in parentheses as before
                else if arg.starts_with('(') && arg.ends_with(')') && line.len() == 2 {
                    let expression = &arg[1..arg.len() - 1];
                    match eval(expression) {
                        Ok(result) => print!("{}", result),
                        Err(err) => println!("{} '{}' : {}", "ERROR: Could not evaluate expression".red(), expression, err),
                    }
                } else {
                    println!("{} '{}'", "ERROR: Could not parse argument".red(), arg);
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
                                            *current_line = line_num + 1; // Jump to label line
                                            return true; // Indicate jump occurred
                                        } else {
                                            println!("{} {}", "ERROR: Label '{}' not found".red(), label);
                                        }
                                    }
                                }
                            } else {
                                println!("{}", "ERROR: Expected a boolean value from condition evaluation.".red());
                            }
                        }
                        Err(err) => println!("{} {} {}", "ERROR: Could not evaluate condition ".red(), condition_str, err),
                    }
                } else {
                    println!("{} {}", "ERROR: Could not parse condition ".red(), condition);
                }
            }
        },
        "goto" => {
            let label = line[1].trim_start_matches(':');

            #[cfg(debug_assertions)]
            println!("{} {} {} {:?}", "Attempting to goto label : ".yellow() , label , "Known labels: ".yellow() , labels);

            if let Some(&line_num) = labels.get(label) {

                #[cfg(debug_assertions)]
                println!("{} {} {} {}", "Jumping from line -> ".yellow() , current_line ," to line -> ".yellow() , line_num);

                *current_line = line_num + 1; // Keep the +1 to move to line after label
                return true;
            } else {
                println!("{} '{}' {}", "ERROR: Label : ".red(),  label ," not found".red());
            }
        },
        "//" => {
            return false;
        },
        label if label.ends_with(':') => {
            // If it's a label, just return true and continue to the next iteration
            return false;
        }
        _ => {
            println!("{}", "ERROR: No command found or invalid label".red());
        }
    }

    // Increment current_line after processing the command
    false
}


fn main() {
    let start_time = Instant::now();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{}", "Usage: zen <filename> [args]".red());
        process::exit(1);
    }

    let file_path = &args[1];
    let zen_args: Vec<i64> = args[2..].iter()
        .map(|arg| arg.parse::<i64>().unwrap_or(0))
        .collect();


    let absolute_path = match std::fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error resolving file path: {}", e);
            process::exit(1);
        }
    };

    let file = match File::open(&absolute_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    let reader = BufReader::new(file);
    separate_zen_code(reader, zen_args);

    let duration = start_time.elapsed();
    println!("\n\nExecution time: {:?}", duration);
}