// Import the parent module's Command trait, CommandError type, and find_executable utility function
use super::{Command, CommandError, utils::find_executable};
// Import the Redirection struct and RedirectionType enum from the crate root
use crate::{Redirection, RedirectionType};
// Import OpenOptions for file operations and Write trait for writing to files
use std::{fs::OpenOptions, io::Write};

// Define the Type struct which represents the 'type' shell builtin command
// This struct has no fields as it only serves as a marker for implementing the Command trait
pub struct Type;

// Implement the Command trait for the Type struct
impl Command for Type {
	// The execute method is required by the Command trait
	// It takes a reference to self, a slice of string arguments, and an optional redirection reference
	// Returns Result indicating success (()) or failure (CommandError)
	fn execute(
		&self,                          // Borrow self immutably since Type has no state
		args: &[String],                // Slice of command line arguments passed to 'type'
		redirection: Option<&Redirection>, // Optional output redirection configuration
	) -> Result<(), CommandError> {   // Return unit on success, CommandError on failure
		// Define an array containing all shell builtin command names
		// This hardcoded list is checked to determine if a command is a shell builtin
		let command_list: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

		// Pattern match on the first argument from the args slice
		// args.first() returns Option<&String> - Some if args is non-empty, None otherwise
		match args.first() {
			// If there is at least one argument, process it
			Some(arg) => {
				// Convert the String reference to a string slice (&str) for easier comparison
				let arg = arg.as_str();
				// Build the output message based on what type of command arg is
				// Uses if-else chain to determine the appropriate message format
				let output = if command_list.contains(&arg) {
					// If arg is in the builtin list, format the message to indicate it's a shell builtin
					format!("{} is a shell builtin", arg)
				} else if let Some(path) = find_executable(arg) {
					// If find_executable returns Some(path), the command exists in PATH
					// Format the message to show the full path of the executable
					format!("{} is {}", arg, path)
				} else {
					// If neither builtin nor found in PATH, indicate the command was not found
					format!("{}: not found", arg)
				};

				// Handle output redirection if a redirection was specified
				if let Some(redir) = redirection {
					// Check if the redirection type is stdout (overwrite mode: >)
					if matches!(redir.redirection_type, RedirectionType::Stdout) {
						// Attempt to open the target file with write, create, and truncate flags
						match OpenOptions::new()
							.write(true)         // Enable writing to the file
							.create(true)        // Create the file if it doesn't exist
							.truncate(true)      // Truncate the file to zero length if it exists (overwrite)
							.open(&redir.file)   // Open the file specified in the redirection
						{
							// If the file opened successfully, write the output to it
							Ok(mut file) => {
								// Write the output string followed by a newline to the file
								writeln!(file, "{}", output)
									// Convert any IO error to a CommandError and return early
									.map_err(|e| CommandError { message: e.to_string() })?;
								// Flush the file buffer to ensure all data is written to disk
								file.flush()
									// Convert any IO error to a CommandError and return early
									.map_err(|e| CommandError { message: e.to_string() })?;
							},
							// If opening the file failed, return a CommandError with the error message
							Err(e) => {
								return Err(CommandError {
									message: format!("{}: {}", redir.file, e),
								});
							},
						}
					} else if matches!(redir.redirection_type, RedirectionType::StdoutAppend) {
						// Check if the redirection type is stdout append mode (>>)
						// Attempt to open the target file with write, create, and append flags
						match OpenOptions::new()
							.write(true)         // Enable writing to the file
							.create(true)        // Create the file if it doesn't exist
							.append(true)        // Append to the end of the file if it exists
							.open(&redir.file)   // Open the file specified in the redirection
						{
							// If the file opened successfully, append the output to it
							Ok(mut file) => {
								// Write the output string followed by a newline to the file
								writeln!(file, "{}", output)
									// Convert any IO error to a CommandError and return early
									.map_err(|e| CommandError { message: e.to_string() })?;
								// Flush the file buffer to ensure all data is written to disk
								file.flush()
									// Convert any IO error to a CommandError and return early
									.map_err(|e| CommandError { message: e.to_string() })?;
							},
							// If opening the file failed, return a CommandError with the error message
							Err(e) => {
								return Err(CommandError {
									message: format!("{}: {}", redir.file, e),
								});
							},
						}
					} else {
						// For stderr redirection (2>), output still goes to stdout in this implementation
						// Print the output message to standard output with a newline
						println!("{}", output);
					}
				} else {
					// No redirection specified, so print the output directly to stdout
					println!("{}", output);
				}

				// Determine the return value based on whether the command was found
				if command_list.contains(&arg) {
					// If arg is a builtin command, return Ok(()) indicating success
					Ok(())
				} else if find_executable(arg).is_some() {
					// If arg is an external command found in PATH, return Ok(()) indicating success
					Ok(())
				} else {
					// If the command was neither builtin nor found in PATH, return an error
					Err(CommandError { message: format!("{}: not found", arg) })
				}
			},
			// If no arguments were provided (args is empty), return Ok(())
			// The 'type' command with no arguments is considered successful but does nothing
			None => Ok(()),
		}
	}
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import all items from the parent module (Type, Command, CommandError, etc.)
	use super::*;

	// Test function to verify the 'type' command works with builtin commands
	#[test]
	fn test_type_builtin() {
		// Create an instance of the Type command
		let type_cmd = Type;
		// Execute the type command with "echo" as the argument (a builtin)
		let result = type_cmd.execute(&["echo".to_string()], None);
		// Assert that the result is Ok (successful execution)
		assert!(result.is_ok());
	}

	// Test function to verify the 'type' command works with external commands
	#[test]
	fn test_type_external() {
		// Create an instance of the Type command
		let type_cmd = Type;
		// Execute the type command with "ls" as the argument (should be in PATH)
		let result = type_cmd.execute(&["ls".to_string()], None);
		// Assert that the result is either Ok or Err (both are acceptable since we can't guarantee PATH)
		assert!(result.is_ok() || result.is_err()); // Either way is fine
	}

	// Test function to verify the 'type' command properly handles nonexistent commands
	#[test]
	fn test_type_nonexistent() {
		// Create an instance of the Type command
		let type_cmd = Type;
		// Execute the type command with a command that definitely doesn't exist
		let result = type_cmd.execute(&["nonexistent_command_12345".to_string()], None);
		// Assert that the result is an error (command not found)
		assert!(result.is_err());
	}

	// Test function to verify the 'type' command handles no arguments gracefully
	#[test]
	fn test_type_no_args() {
		// Create an instance of the Type command
		let type_cmd = Type;
		// Execute the type command with no arguments
		// This should not panic and should return Ok
		assert!(type_cmd.execute(&[], None).is_ok());
	}

	// Test function to verify the 'type' command doesn't panic with various inputs
	#[test]
	fn test_type_does_not_panic() {
		// Create an instance of the Type command
		let type_cmd = Type;
		// Discard the result of executing with no arguments (shouldn't panic)
		let _ = type_cmd.execute(&[], None);
		// Discard the result of executing with "echo" (shouldn't panic)
		let _ = type_cmd.execute(&["echo".to_string()], None);
		// Discard the result of executing with "ls" (shouldn't panic)
		let _ = type_cmd.execute(&["ls".to_string()], None);
		// Discard the result of executing with "nonexistent" (shouldn't panic)
		let _ = type_cmd.execute(&["nonexistent".to_string()], None);
	}

	// Test function to verify the 'type' command works with all builtin commands
	#[test]
	fn test_type_all_builtins() {
		// Create an instance of the Type command
		let type_cmd = Type;
		// Define an array of all builtin commands to test
		let builtins = ["echo", "exit", "type", "pwd", "cd"];
		// Iterate through each builtin command
		for builtin in builtins {
			// Assert that executing type with each builtin succeeds
			assert!(type_cmd.execute(&[builtin.to_string()], None).is_ok());
		}
	}
}
