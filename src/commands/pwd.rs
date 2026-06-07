// Import the Command trait and CommandError type from the parent module
use super::{Command, CommandError};
// Import Redirection and RedirectionType from the crate root
use crate::{Redirection, RedirectionType};
// Import environment variable functions, file open options, and write trait
use std::{env, fs::OpenOptions, io::Write};

// Define the Pwd struct which represents the 'pwd' shell builtin command
// This struct has no fields as it only serves as a marker for implementing the Command trait
pub struct Pwd;

// Implement the Command trait for the Pwd struct
impl Command for Pwd {
	// The execute method is required by the Command trait
	fn execute(
		&self, // Borrow self immutably since Pwd has no state
		_args: &[String], /* Unused - pwd doesn't use arguments (underscore
		        * prefix suppresses warning) */
		redirection: Option<&Redirection>, // Optional output redirection configuration
	) -> Result<(), CommandError> {
		// Return unit on success, CommandError on failure
		// Get the current working directory from the environment
		match env::current_dir() {
			// If successfully got the current directory
			Ok(path) => {
				// Convert the path to a string using display formatting
				let path_str = format!("{}", path.display());
				// Check if output redirection was specified
				if let Some(redir) = redirection {
					// Create the file to ensure it exists (pre-create to handle edge cases)
					let _file = OpenOptions::new().write(true).create(true).open(&redir.file);

					// Check if redirection type is stdout (overwrite mode: >)
					if matches!(redir.redirection_type, RedirectionType::Stdout) {
						// Attempt to open the target file with write, create, and truncate flags
						match OpenOptions::new()
							.write(true)         // Enable writing to the file
							.create(true)        // Create the file if it doesn't exist
							.truncate(true)      // Truncate the file to zero length (overwrite)
							.open(&redir.file)   // Open the file specified in the redirection
						{
							// If the file opened successfully, write the path to it
							Ok(mut file) => {
								// Write the path string followed by a newline to the file
								writeln!(file, "{}", path_str)
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
						// Check if redirection type is stdout append mode (>>)
						// Attempt to open the target file with write, create, and append flags
						match OpenOptions::new()
							.write(true)         // Enable writing to the file
							.create(true)        // Create the file if it doesn't exist
							.append(true)        // Append to the end of the file if it exists
							.open(&redir.file)   // Open the file specified in the redirection
						{
							// If the file opened successfully, append the path to it
							Ok(mut file) => {
								// Write the path string followed by a newline to the file
								writeln!(file, "{}", path_str)
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
						// For stderr redirection (2>), output still goes to stdout in this
						// implementation Print the path to standard output with a newline
						println!("{}", path_str);
					}
				} else {
					// No redirection specified, so print the path directly to stdout
					println!("{}", path_str);
				}
				// Return Ok(()) to indicate successful execution
				Ok(())
			},
			// If getting the current directory failed
			Err(e) => {
				// Print the error to stderr with the "pwd:" prefix
				eprintln!("pwd: {}", e);
				// Return a CommandError containing the error message
				Err(CommandError { message: e.to_string() })
			},
		}
	}
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import all items from the parent module (Pwd, Command, CommandError, etc.)
	use super::*;

	// Test function to verify pwd executes successfully
	#[test]
	fn test_pwd_execute() {
		// Create an instance of the Pwd command
		let pwd = Pwd;
		// Execute pwd with no arguments and no redirection
		assert!(pwd.execute(&[], None).is_ok());
	}

	// Test function to verify pwd ignores extra arguments
	#[test]
	fn test_pwd_execute_with_args() {
		// Create an instance of the Pwd command
		let pwd = Pwd;
		// pwd should ignore arguments (they're prefixed with _ to indicate unused)
		assert!(pwd.execute(&["extra".to_string()].to_vec(), None).is_ok());
	}

	// Test function to verify pwd doesn't panic
	#[test]
	fn test_pwd_does_not_panic() {
		// Create an instance of the Pwd command
		let pwd = Pwd;
		// Discard the result - just verify it doesn't panic
		let _ = pwd.execute(&[], None);
	}

	// Test function to verify pwd works with redirection
	#[test]
	fn test_pwd_with_redirection() {
		// Create an instance of the Pwd command
		let pwd = Pwd;
		// Create a redirection configuration for stdout to a temp file
		let redirection = Some(Redirection {
			file: "/tmp/test_pwd.txt".to_string(),
			redirection_type: RedirectionType::Stdout,
		});
		// Execute pwd with redirection
		let result = pwd.execute(&[], redirection.as_ref());
		// Assert that the execution succeeded
		assert!(result.is_ok());
		// Verify the file was created and contains the path
		use std::fs;
		let content = fs::read_to_string("/tmp/test_pwd.txt").unwrap();
		// Assert that the file is not empty
		assert!(!content.is_empty());
		// Clean up the test file
		let _ = fs::remove_file("/tmp/test_pwd.txt");
	}
}
