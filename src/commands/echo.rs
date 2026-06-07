// Import the Command trait and CommandError type from the parent module
use super::{Command, CommandError};
// Import Redirection and RedirectionType from the crate root
use crate::{Redirection, RedirectionType};
// Import file open options, IO module, and Write trait
use std::{
	fs::OpenOptions,
	io::{self, Write},
};

// Define the Echo struct which represents the 'echo' shell builtin command
// This struct has no fields as it only serves as a marker for implementing the Command trait
pub struct Echo;

// Implement the Command trait for the Echo struct
impl Command for Echo {
	// The execute method is required by the Command trait
	fn execute(
		&self,                             // Borrow self immutably since Echo has no state
		args: &[String],                   // Slice of command line arguments passed to 'echo'
		redirection: Option<&Redirection>, // Optional output redirection configuration
	) -> Result<(), CommandError> {
		// Return unit on success, CommandError on failure
		// Flag to track if the -n option was provided (suppress trailing newline)
		let mut omit_newline = false;
		// Vector to store filtered arguments (excluding the -n flag if present)
		let mut filtered_args: Vec<&str> = Vec::new();

		// Check if the very first argument is the "-n" flag (which suppresses the trailing newline)
		if !args.is_empty() && args[0] == "-n" {
			// Set the flag to omit the newline at the end of output
			omit_newline = true;
			// Skip the flag and take the rest of the arguments
			// Convert each String to &str and skip the first element (the "-n" flag)
			filtered_args.extend(args.iter().map(|s| s.as_str()).skip(1));
		} else {
			// No -n flag, so include all arguments as-is
			// Convert each String to &str and add to filtered_args
			filtered_args.extend(args.iter().map(|s| s.as_str()));
		}

		// Join the arguments with a single space between them to form the output string
		let output = filtered_args.join(" ");

		// Check if output redirection was specified
		if let Some(redir) = redirection {
			// Create the file to ensure it exists (pre-create to handle edge cases)
			let _file = OpenOptions::new().write(true).create(true).open(&redir.file);

			// Check if redirection type is stdout (overwrite mode: >)
			if matches!(redir.redirection_type, RedirectionType::Stdout) {
				// Attempt to open the target file with write, create, and truncate flags
				match OpenOptions::new().write(true).create(true).truncate(true).open(&redir.file) {
					// If the file opened successfully, write the output to it
					Ok(mut file) => {
						// Check if we should omit the newline
						if omit_newline {
							// Write without a trailing newline
							write!(file, "{}", output)
								// Convert any IO error to a CommandError and return early
								.map_err(|e| CommandError { message: e.to_string() })?;
						} else {
							// Write with a trailing newline
							writeln!(file, "{}", output)
								// Convert any IO error to a CommandError and return early
								.map_err(|e| CommandError { message: e.to_string() })?;
						}
						// Flush the file buffer to ensure all data is written to disk
						file.flush().map_err(|e| CommandError { message: e.to_string() })?;
					},
					// If opening the file failed, return a CommandError with the error message
					Err(e) => {
						return Err(CommandError { message: format!("{}: {}", redir.file, e) });
					},
				}
			} else if matches!(redir.redirection_type, RedirectionType::StdoutAppend) {
				// Check if redirection type is stdout append mode (>>)
				// Attempt to open the target file with write, create, and append flags
				match OpenOptions::new().write(true).create(true).append(true).open(&redir.file) {
					// If the file opened successfully, append the output to it
					Ok(mut file) => {
						// Check if we should omit the newline
						if omit_newline {
							// Write without a trailing newline
							write!(file, "{}", output)
								// Convert any IO error to a CommandError and return early
								.map_err(|e| CommandError { message: e.to_string() })?;
						} else {
							// Write with a trailing newline
							writeln!(file, "{}", output)
								// Convert any IO error to a CommandError and return early
								.map_err(|e| CommandError { message: e.to_string() })?;
						}
						// Flush the file buffer to ensure all data is written to disk
						file.flush().map_err(|e| CommandError { message: e.to_string() })?;
					},
					// If opening the file failed, return a CommandError with the error message
					Err(e) => {
						return Err(CommandError { message: format!("{}: {}", redir.file, e) });
					},
				}
			} else {
				// For stderr redirection (2>), output still goes to stdout in this implementation
				// Check if we should omit the newline
				if omit_newline {
					// Print without a trailing newline
					print!("{}", output);
					// Flush stdout to ensure output is displayed immediately (important without \n)
					let _ = io::stdout().flush(); // Ensure it prints right away without \n
				} else {
					// Print with a trailing newline
					println!("{}", output);
				}
			}
		} else {
			// No redirection specified, so output directly to stdout
			// Check if we should omit the newline
			if omit_newline {
				// Print without a trailing newline
				print!("{}", output);
				// Flush stdout to ensure output is displayed immediately (important without \n)
				let _ = io::stdout().flush(); // Ensure it prints right away without \n
			} else {
				// Print with a trailing newline
				println!("{}", output);
			}
		}

		// Return Ok(()) to indicate successful execution
		Ok(())
	}
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import all items from the parent module (Echo, Command, CommandError, etc.)
	use super::*;

	// Test function to verify echo works with simple arguments
	#[test]
	fn test_echo_simple() {
		// Create an instance of the Echo command
		let echo = Echo;
		// Execute echo with two arguments
		assert!(echo.execute(&["hello".to_string(), "world".to_string()], None).is_ok());
	}

	// Test function to verify echo handles the -n flag correctly
	#[test]
	fn test_echo_with_n_flag() {
		// Create an instance of the Echo command
		let echo = Echo;
		// Execute echo with the -n flag (suppress newline)
		assert!(echo.execute(&["-n".to_string(), "test".to_string()], None).is_ok());
	}

	// Test function to verify echo handles empty arguments
	#[test]
	fn test_echo_empty_args() {
		// Create an instance of the Echo command
		let echo = Echo;
		// Execute echo with no arguments (should just print a newline)
		assert!(echo.execute(&[], None).is_ok());
	}

	// Test function to verify echo works with a single argument
	#[test]
	fn test_echo_single_arg() {
		// Create an instance of the Echo command
		let echo = Echo;
		// Execute echo with a single argument
		assert!(echo.execute(&["single".to_string()], None).is_ok());
	}

	// Test function to verify echo works with multiple arguments
	#[test]
	fn test_echo_multiple_args() {
		// Create an instance of the Echo command
		let echo = Echo;
		// Execute echo with three arguments (should be joined with spaces)
		assert!(
			echo.execute(&["one".to_string(), "two".to_string(), "three".to_string()], None)
				.is_ok()
		);
	}

	// Test function to verify echo works with redirection
	#[test]
	fn test_echo_with_redirection() {
		// Create an instance of the Echo command
		let echo = Echo;
		// Create a redirection configuration for stdout to a temp file
		let redirection = Some(Redirection {
			file: "/tmp/test_echo.txt".to_string(),
			redirection_type: RedirectionType::Stdout,
		});
		// Execute echo with redirection
		let result = echo.execute(&["hello".to_string()], redirection.as_ref());
		// Assert that the execution succeeded
		assert!(result.is_ok());
		// Verify the file was created and contains the expected content
		use std::fs;
		let content = fs::read_to_string("/tmp/test_echo.txt").unwrap();
		// Assert the content is "hello\n" (output with newline)
		assert_eq!(content, "hello\n");
		// Clean up the test file
		let _ = fs::remove_file("/tmp/test_echo.txt");
	}
}
