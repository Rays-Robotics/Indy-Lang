use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::env;
use std::io::{self, Write};
use std::process::Command;

// --- CONFIGURATION ---
const INDY_VERSION: &str = "0.6.3-fix-loop-move";

// --- DATA STRUCTURES ---

/// Stores the state required for an active loop block.
// FIX: Add Copy and Clone traits to prevent the "use of moved value" error (E0382)
// when pushing the frame back onto the stack and immediately reading from it.
#[derive(Debug, Clone, Copy)]
struct LoopFrame {
    start_line_index: usize,
    max_iterations: usize,
    current_iteration: usize,
}

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

/// Splits a string into shell-like arguments, respecting single quotes ('). 
/// The quotes themselves are stripped from the resulting arguments.
fn split_shell_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quote = false;

    for c in s.chars() {
        if c == '\'' {
            in_quote = !in_quote;
            // Do not add the quote character itself to the argument string
        } else if c.is_whitespace() && !in_quote {
            if !current_arg.is_empty() {
                args.push(current_arg.clone());
            }
            current_arg.clear();
        } else {
            current_arg.push(c);
        }
    }
    if !current_arg.is_empty() {
        args.push(current_arg);
    }
    
    args
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

/// Finds the index of the next instruction after a failed conditional block (either 'else' or 'end if')
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
            // Use the cleaner strip_prefix method
            let raw_message = trimmed_line.strip_prefix(command)
                .unwrap_or("")
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
        "run" => {
            // 1. Isolate the quoted command argument and clean the quotes
            let raw_args = trimmed_line.trim_start_matches("run").trim();
            let cleaned_cmd_arg = clean_string_value(raw_args);
            
            // 2. Perform interpolation on the command string
            let interpolated_cmd = interpolate_string(&cleaned_cmd_arg, variables);

            if interpolated_cmd.is_empty() {
                eprintln!("[Run Error] 'run' requires a quoted command string.");
                return;
            }

            // 3. Split into command and arguments using the shell-like parser
            let cmd_parts = split_shell_args(&interpolated_cmd);
            let cmd = &cmd_parts[0];
            // Arguments are the elements after the command name. 
            let args_refs: Vec<&str> = cmd_parts[1..].iter().map(|s| s.as_str()).collect();

            if is_verbose {
                println!("[Indy Engine] Running system command: '{}' with args: {:?}", cmd, args_refs);
            }

            // 4. Execute the system command
            match Command::new(cmd).args(args_refs).output() {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        print!("{}", stdout);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("[Run Error] Command failed (Exit code: {:?}): {}", output.status.code(), stderr);
                    }
                },
                Err(e) => eprintln!("[Run Error] Could not execute command '{}': {}", cmd, e),
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
    
    // Stack to manage loop execution state for jumps
    let mut loop_stack: Vec<LoopFrame> = Vec::new();


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
            if is_current_block_active {
                let loop_args = trimmed_line.trim_start_matches("loop ").trim();
                
                // Interpolate loop argument
                let interpolated_args = interpolate_string(loop_args, variables);
                
                // Simplified loop argument parsing (only supports integer count for now)
                let count = interpolated_args.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("[Error] 'loop' requires a positive integer count. Defaulting to 1.");
                    1
                });
                
                if is_verbose {
                    println!("[Indy Engine] Starting loop ({} iterations) at line {}", count, line_num);
                }
                
                // Push the new loop frame onto the stack
                loop_stack.push(LoopFrame {
                    start_line_index: line_num + 1, // Store index of line AFTER 'loop' command
                    max_iterations: count,
                    current_iteration: 0,
                });
            } else {
                // Parent block is inactive, so skip the whole loop body
                line_num = find_matching_end(lines.as_slice(), line_num, "loop");
                continue;
            }

        } else if trimmed_line == "end loop" {
            if let Some(mut frame) = loop_stack.pop() {
                if frame.current_iteration < frame.max_iterations - 1 {
                    // Loop is not finished: increment counter, push frame back, and jump
                    frame.current_iteration += 1;
                    if is_verbose {
                        println!("[Indy Engine] Looping back to line {} (Iteration {}/{})", 
                                frame.start_line_index, frame.current_iteration + 1, frame.max_iterations);
                    }
                    // Since LoopFrame now implements Copy, this push uses a copy, 
                    // allowing frame to still be used on the next line.
                    loop_stack.push(frame); 
                    line_num = frame.start_line_index; // Jump directly to the first instruction inside the loop
                    continue; // Skip line_num += 1, as line_num was manually set
                } else {
                    // Loop finished. Just continue linear execution.
                    if is_verbose {
                         println!("[Indy Engine] Loop finished.");
                    }
                }
            } else {
                eprintln!("[Error] 'end loop' without matching 'loop' found.");
            }
        
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
