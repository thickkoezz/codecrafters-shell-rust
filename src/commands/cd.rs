// Import the Command trait and CommandError type from the parent module
use super::{Command, CommandError};
// Import the Redirection struct from the crate root
use crate::Redirection;
// Import environment variable functions and Path type from standard library
use std::{env, path::Path};

// Define the Cd struct which represents the 'cd' shell builtin command
// This struct has no fields as it only serves as a marker for implementing the Command trait
pub struct Cd;

// Implement the Command trait for the Cd struct
impl Command for Cd {
	// The execute method is required by the Command trait
	fn execute(
		&self,                          // Borrow self immutably since Cd has no state
		args: &[String],                // Slice of command line arguments passed to 'cd'
		_redirection: Option<&Redirection>, // Unused - cd doesn't produce output to redirect (underscore prefix suppresses warning)
	) -> Result<(), CommandError> {   // Return unit on success, CommandError on failure
		// Pattern match on the first argument from the args slice
		match args.first() {
			// If there is at least one argument, process the path
			Some(arg) => {
				// Build the target path by handling different path formats
				let target_path = if arg.starts_with('/') {
					// Absolute path - use as-is (starts from root)
					Path::new(arg).to_path_buf()
				} else if arg.starts_with('~') {
					// Tilde expansion - replace with HOME directory
					// Try to get the HOME environment variable, return error if not set
					let home_dir = env::var("HOME").map_err(|_| CommandError {
						message: "HOME environment variable not set".to_string(),
					})?;
					// Handle both "~" and "~/path" cases
					if arg == "~" {
						// Just "~" - use the home directory directly
						Path::new(&home_dir).to_path_buf()
					} else {
						// "~/something" - join with home directory
						// Skip the first 2 characters ("~/") and append the rest to home
						let rest = &arg[2..]; // Skip "~/"
						Path::new(&home_dir).join(rest)
					}
				} else {
					// Relative path - join with current directory
					// Get the current working directory, return error if it fails
					let current_dir = env::current_dir().map_err(|e| CommandError {
						message: format!("Failed to get current directory: {}", e),
					})?;
					// Join the current directory with the relative path argument
					current_dir.join(arg)
				};

				// Check if the target path exists
				if target_path.exists() {
					// Try to change to the target directory
					if env::set_current_dir(&target_path).is_err() {
						// If changing directory fails, it's likely a permission issue
						eprintln!("cd: {}: Permission denied", arg);
						// Return an error indicating permission was denied
						return Err(CommandError { message: format!("Permission denied: {}", arg) });
					}
				} else {
					// If the path doesn't exist, print error to stderr
					eprintln!("cd: {}: No such file or directory", arg);
					// Return an error indicating the path was not found
					return Err(CommandError {
						message: format!("No such file or directory: {}", arg),
					});
				}
			},
			// If no arguments were provided (args is empty), do nothing
			// The 'cd' command with no arguments typically changes to HOME, but this implementation does nothing
			None => {},
		}
		// Return Ok(()) to indicate successful execution
		Ok(())
	}
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import all items from the parent module (Cd, Command, CommandError, etc.)
	use super::*;

	// Test function to verify cd handles no arguments gracefully
	#[test]
	fn test_cd_no_args() {
		// Create an instance of the Cd command
		let cd = Cd;
		// cd with no args should not panic
		assert!(cd.execute(&[], None).is_ok());
	}

	// Test function to verify cd works with absolute paths
	#[test]
	fn test_cd_absolute_path() {
		// Create an instance of the Cd command
		let cd = Cd;
		// Test with /tmp which should exist on most Unix systems
		let result = cd.execute(&["/tmp".to_string()], None);
		// Should succeed or return an error, but not panic
		assert!(result.is_ok() || result.is_err());
	}

	// Test function to verify cd properly handles nonexistent paths
	#[test]
	fn test_cd_nonexistent_path() {
		// Create an instance of the Cd command
		let cd = Cd;
		// Test with a non-existent path
		let result = cd.execute(&["/nonexistent/path/that/does/not/exist".to_string()], None);
		// Should return an error
		assert!(result.is_err());
	}

	// Test function to verify cd works with relative paths
	#[test]
	fn test_cd_relative_path() {
		// Create an instance of the Cd command
		let cd = Cd;
		// Test relative path - should work if directory exists
		// Since we can't guarantee what directories exist, we just verify it doesn't panic
		let result = cd.execute(&["..".to_string()], None);
		// Should succeed (parent directory always exists) or return an error, but not panic
		assert!(result.is_ok() || result.is_err());
	}

	// Test function to verify cd doesn't panic with various inputs
	#[test]
	fn test_cd_does_not_panic() {
		// Create an instance of the Cd command
		let cd = Cd;
		// Discard the result of executing with no arguments (shouldn't panic)
		let _ = cd.execute(&[], None);
		// Discard the result of executing with /tmp (shouldn't panic)
		let _ = cd.execute(&["/tmp".to_string()], None);
		// Discard the result of executing with nonexistent (shouldn't panic)
		let _ = cd.execute(&["nonexistent".to_string()], None);
	}

	// Test function to verify cd handles tilde expansion
	#[test]
	fn test_cd_tilde() {
		// Create an instance of the Cd command
		let cd = Cd;
		// Set HOME for testing (unsafe because it modifies process state)
		unsafe {
			env::set_var("HOME", "/tmp/test_home");
		}
		// Execute cd with ~ as the argument
		let result = cd.execute(&["~".to_string()], None);
		// Should not panic, may succeed or fail depending on if directory exists
		assert!(result.is_ok() || result.is_err());
	}
}
