use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::env;
use std::io::{self, Write};

// --- CONFIGURATION ---
const INDY_VERSION: &str = "0.5.2-fix-e0716";

// --- HELPER FUNCTIONS ---

/// Performs string interpolation: replaces {VAR} with the value from the variable map.
fn interpolate_string(s: &str, variables: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (key, value) in variables.iter() {
        let pattern = format!("{{{}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

/// Safely removes surrounding double quotes from a string value if they exist.
fn clean_string_value(s: &str) -> String {
    let mut value = s.trim().to_string();
    
    // Only strip quotes if they are clearly surrounding the string
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        value.pop();
        value.remove(0);
    }
    value
}

// --- CONTROL FLOW UTILITIES ---

/// Finds the index of the matching 'end if' or 'end loop' for block skipping.
fn find_matching_end(lines: &[&str], start_index: usize, keyword: &str) -> usize {
    let mut depth = 1;
    let end_keyword = format!("end {}", keyword);

    for i in (start_index + 1)..lines.len() {
        let trimmed = lines[i].trim();
        // Check for nested blocks of the same type
        if trimmed.starts_with(keyword) && trimmed != end_keyword {
            depth += 1;
        } else if trimmed == end_keyword {
            depth -= 1;
            if depth == 0 {
                return i; // Found the matching 'end'
            }
        }
    }
    lines.len()
}

/// Utility function to evaluate simple string comparison conditions.
fn evaluate_condition(condition_str: &str, variables: &HashMap<String, String>) -> bool {
    // Find the operator: '==' or '!='
    let (left, op, right) = if let Some(parts) = condition_str.split_once("==") {
        (parts.0.trim(), "==", parts.1.trim())
    } else if let Some(parts) = condition_str.split_once("!=") {
        (parts.0.trim(), "!=", parts.1.trim())
    } else {
        eprintln!("[Error] Invalid condition format. Use VAR == VALUE or VAR != VALUE.");
        return false;
    };

    // 1. Get the value of the left-hand side (must be a variable)
    let left_value = variables.get(left).map(|s| s.as_str()).unwrap_or("");
    
    // 2. Get the value of the right-hand side (can be a variable or a literal)
    // FIX: Introduce a long-lived variable `literal_value` if `right` is not a variable.
    let literal_value;
    let right_value: &str = if variables.contains_key(right) {
        variables.get(right).unwrap().as_str()
    } else {
        // Assume right side is a literal. Store the cleaned string in `literal_value`.
        literal_value = clean_string_value(right);
        literal_value.as_str()
    };
    
    // 3. Perform the comparison
    match op {
        "==" => left_value == right_value,
        "!=" => left_value != right_value,
        _ => false,
    }
}

// Finds the index of the next instruction after a failed conditional block (either 'else' or 'end if')
fn find_next_if_skip_target(lines: &[&str], start_index: usize) -> usize {
    let mut depth = 1;
    for i in (start_index + 1)..lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("if ") {
            depth += 1;
        } else if trimmed == "else" && depth == 1 {
            return i; // Found the matching 'else' for this block
        } else if trimmed == "end if" {
            depth -= 1;
            if depth == 0 {
                return i; // Found the matching 'end if'
            }
        }
    }
    lines.len()
}

// --- CORE FUNCTIONS ---

/// Executes a single line of Indy-lang code.
fn execute_line(line: &str, variables: &mut HashMap<String, String>, is_verbose: bool) {
    let trimmed_line = line.trim();
    let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
    
    if parts.is_empty() {
        return;
    }

    let command = parts[0];

    match command {
        "say" => {
            let raw_message = trimmed_line
                .trim_start_matches(command)
                .trim_start();
            
            let message = clean_string_value(raw_message);
            let output = interpolate_string(&message, variables);
            println!("{}", output);
        },
        "wait" => {
            if parts.len() > 1 {
                if let Ok(duration_sec) = parts[1].parse::<f64>() {
                    if is_verbose {
                        println!("[Indy Engine] Waiting for {} seconds...", duration_sec);
                    }
                    let duration_ms = (duration_sec * 1000.0) as u64;
                    thread::sleep(Duration::from_millis(duration_ms));
                } else {
                    eprintln!("[Error] Invalid duration for 'wait'. Must be a number.");
                }
            } else {
                eprintln!("[Error] 'wait' command requires a duration in seconds.");
            }
        },
        "prompt" => {
            let prompt_args = trimmed_line.trim_start_matches("prompt").trim();

            if let Some((var_name, quoted_prompt)) = prompt_args.split_once('=') {
                let var_name = var_name.trim().to_string();
                let prompt_message = clean_string_value(quoted_prompt);
                let interpolated_prompt = interpolate_string(&prompt_message, variables);

                // Print the prompt message with the required formatting: "[PROMPT TEXT]: "
                print!("{}: ", interpolated_prompt);
                io::stdout().flush().expect("Failed to flush stdout");

                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        let captured_value = input.trim().to_string();
                        variables.insert(var_name, captured_value);
                    },
                    Err(e) => eprintln!("[Error] Failed to read input for prompt: {}", e),
                }

            } else {
                eprintln!("[Error] 'prompt' command syntax is incorrect. Use: prompt VAR=\"Message\"");
            }
        },
        // Handles variable assignment like: Name="bob"
        _ if trimmed_line.contains('=') => {
            if let Some((name, value_str)) = trimmed_line.split_once('=') {
                let name = name.trim().to_string();
                let value = clean_string_value(value_str);

                if !name.contains(' ') {
                    variables.insert(name, value);
                } else {
                    eprintln!("[Error] Variable names cannot contain spaces: '{}'", name);
                }
            }
        },
        _ => {
            // Ignore 'start', 'end', and control flow keywords handled by the runner
            if !command.starts_with('#') && 
               !matches!(command, "start" | "end" | "if" | "else" | "end if" | "loop" | "end loop")
            {
                eprintln!("[Error] Unknown command or bad syntax: '{}'", trimmed_line);
            }
        }
    }
}

/// Executes the script, handling control flow statements.
fn run_indy_script_content(script_content: &str, variables: &mut HashMap<String, String>, is_verbose: bool) -> bool {
    let lines: Vec<&str> = script_content.lines().collect();
    let mut in_script_block = false;
    let mut line_num = 0;
    
    // Stack to manage whether we are currently inside an active control flow block (e.g., executing the true path of an IF)
    let mut block_execution_stack: Vec<bool> = vec![true]; // Starts as true (global execution)

    while line_num < lines.len() {
        let line = lines[line_num];
        let trimmed_line = line.trim();
        
        // Skip empty lines and comments
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            line_num += 1;
            continue;
        }

        if trimmed_line == "start" {
            in_script_block = true;
            if is_verbose { println!("[Indy Engine] Script started."); }
            line_num += 1;
            continue;
        }

        if !in_script_block {
            line_num += 1;
            continue;
        }

        // Check if we are currently skipping the block (i.e., inside a false 'if' block)
        let is_current_block_active = *block_execution_stack.last().unwrap_or(&true);

        // --- Control Flow Logic ---
        if trimmed_line.starts_with("if ") {
            let condition_str = trimmed_line.trim_start_matches("if ").trim();
            let is_condition_true = evaluate_condition(condition_str, variables);
            
            if is_current_block_active && is_condition_true {
                // Condition is true and we're not skipping a parent block. Enter the block.
                block_execution_stack.push(true);
            } else {
                // Condition is false or a parent block is already skipping. Skip to 'else' or 'end if'.
                block_execution_stack.push(false);
                line_num = find_next_if_skip_target(lines.as_slice(), line_num);
                continue;
            }
        } else if trimmed_line == "else" {
            let last_status = block_execution_stack.pop().unwrap_or(false);

            if last_status {
                // The 'if' block was true, so we must skip the 'else' block.
                block_execution_stack.push(false);
                line_num = find_matching_end(lines.as_slice(), line_num, "if");
                continue;
            } else if is_current_block_active {
                // The 'if' block was false, and we are not skipping a parent block, so the 'else' becomes active.
                block_execution_stack.push(true);
            } else {
                // Parent block is skipping. Continue skipping.
                block_execution_stack.push(false);
            }
        } else if trimmed_line == "end if" {
            block_execution_stack.pop();
        } else if trimmed_line.starts_with("loop ") {
            // MVP SIMULATION: True loop execution requires multi-pass/recursive execution, 
            // which complicates the single-pass runner heavily. We simulate the jump by skipping the block.
            if is_current_block_active {
                if is_verbose {
                    println!("[Indy Engine] Loop encountered. (Simulation: Skipping block to continue execution)");
                }
                // Skip the loop body for simulation
                line_num = find_matching_end(lines.as_slice(), line_num, "loop");
                continue;
            }
        } else if trimmed_line == "end loop" {
            // No action needed for simulated loop end
        } else if trimmed_line == "end" {
            if is_verbose { println!("[Indy Engine] Script finished."); }
            return true; 
        }

        // --- Execute Normal Command ---
        // Only execute if the current block is active
        if *block_execution_stack.last().unwrap_or(&true) {
            execute_line(line, variables, is_verbose);
        }

        line_num += 1;
    }
    
    // If we exit the loop, the script did not finish correctly
    in_script_block
}

// --- MAIN ENTRY POINT ---

fn main() {
    println!("--- Indy-lang Interpreter v{} ---", INDY_VERSION);

    // 1. Get arguments and check for flags
    let args: Vec<String> = env::args().collect();
    let is_verbose = args.iter().any(|arg| arg == "--verbose");
    
    // Determine the filepath index by finding the first argument that is NOT the executable name or a flag
    let filepath_index = args.iter().enumerate()
        .skip(1)
        .find(|(_, arg)| !arg.starts_with("--"))
        .map(|(index, _)| index);

    if filepath_index.is_none() {
        eprintln!("Error: Missing input file.");
        eprintln!("Usage: indy <filepath.indy> [--verbose]");
        return;
    }

    let filepath = &args[filepath_index.unwrap()];

    // 2. Read the script file
    let script_content = match std::fs::read_to_string(filepath) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("[Error] Could not read file {}: {}", filepath, e);
            return;
        }
    };

    // 3. Initialize interpreter state
    let mut variables: HashMap<String, String> = HashMap::new();

    // 4. Process script line by line and check for completion
    let finished_correctly = run_indy_script_content(&script_content, &mut variables, is_verbose);
    
    if !finished_correctly {
        eprintln!("[Error] Script ended unexpectedly (missing 'end' keyword or 'start' was never called).");
    }
}
