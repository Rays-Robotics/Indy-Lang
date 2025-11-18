Indy-lang InterpreterA Concise, Readable Scripting Engine (Version 0.5.2)Indy-lang is a simple, domain-specific language designed for rapidly creating interactive command-line programs, user guides, and sequential logic scripts. Its primary focus is delivering maximum clarity and ease of use through a highly readable, declarative syntax.The name Indy-lang is an acronym for "I'm Not Doing YAML," reflecting the project's goal of offering a simplified scripting solution over more complex configuration management formats.The core interpreter is implemented in Rust, ensuring robust and reliable execution.Core FeaturesLanguage CapabilitiesSequential Execution: Instructions are processed one by one, line-by-line.Variables & Interpolation: Supports untyped string variables (Name="Ray") and automatically interpolates variables within strings using curly braces ({Name}).User Input (prompt): Displays a message to the user and captures their input, storing it in a variable.Conditional Control (if/else): Supports basic branching logic based on string comparisons (== and !=).Pausing (wait): Suspends script execution for a specified duration in seconds.Interpreter FeaturesVerbose Debugging: Use the --verbose flag when running the script to display internal engine messages, which is useful for debugging.Iteration Status (Looping)The loop command is currently recognized by the interpreter but is executed as a simulation. The engine identifies the block but skips the commands inside it to continue linear script execution. Full, functional iteration is planned for the next major release.UsageRequirementsYou must have the Rust toolchain installed to compile and execute the interpreter source code.Running an Indy ScriptCompile the interpreter and execute an Indy script file (e.g., my_script.indy) from your terminal:# Standard Execution (Normal output)
indy my_script.indy

# Diagnostic Execution (Shows engine status)
indy my_script.indy --verbose
Command ReferenceA summary of supported commands and their corresponding syntax is provided below:CommandExampleDescriptionScript Blockstart...endDefines the main executable content of the script.AssignmentName="Ray"Sets the value of a variable.Saysay "Hello, {Name}"Prints the message to the console, replacing variables.Waitwait 1.5Pauses execution for a specified time in seconds.Promptprompt Age="How old are you"Requests user input and stores the response in the Age variable.If/Elseif Var == "Yes"...end ifControls which code block runs based on a condition.Looploop 10 or loop forever(Simulated) Identifies an iteration block but skips execution.Comment# This is a noteText preceded by # is ignored by the interpreter.Example Script (greeting.indy)This script demonstrates variable assignment, user input, conditional branching, and loop simulation.start

# Variable setup
GreetingType="Positive"
UserDecision="false"

say "Welcome to the Indy-lang Demo!"
wait 1.0

# Prompt will display as: "Are you having a good day? (yes/no): "
prompt UserDecision="Are you having a good day? (yes/no)"

if UserDecision == "yes"
    say "That's fantastic news! We hope you have a productive session."
else
    say "We're sorry to hear that. We hope this demo brightens your day."
end if

# Simulated loop execution
loop 3
    say "This line is skipped in the current MVP simulation."
end loop

say "Script finished. Thanks for participating."

end
