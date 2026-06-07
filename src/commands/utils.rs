// Import the filesystem module (fs) for file metadata operations
// Import PermissionsExt from the Unix-specific OS module for checking file permissions
// Import Path for working with file paths
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

/// Find executable in PATH and return the full path if found
/// This function searches through the PATH environment variable to locate executable commands
pub fn find_executable(command: &str) -> Option<String> {
	// Try to get the PATH environment variable
	// .ok() converts Result to Option, returning None if PATH is not set
	if let Some(path_env) = std::env::var("PATH").ok() {
		// Split the PATH string by ':' to get individual directory paths (Unix-style)
		for dir in path_env.split(':') {
			// Join the directory path with the command name to form the full path
			let full_path = Path::new(dir).join(command);
			// Check if the file at the full path exists
			if full_path.exists() {
				// Check if file has execute permissions
				// Get metadata, then check permissions mode & 0o111 (execute bits for user/group/other)
				// unwrap_or(false) defaults to false if metadata check fails
				if fs::metadata(&full_path)
					.map(|meta| meta.permissions().mode() & 0o111 != 0) // Check if any execute bit is set
					.unwrap_or(false)
				{
					// Convert path to string, handling non-UTF8 characters gracefully with to_string_lossy
					return Some(full_path.to_string_lossy().to_string());
				}
			}
		}
	}
	// If no executable was found in PATH, return None
	None
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import all items from the parent module (find_executable function)
	use super::*;

	// Test function to verify find_executable works with 'ls'
	#[test]
	fn test_find_executable_ls() {
		// ls should be found in PATH on most Unix systems
		let result = find_executable("ls");
		// Assert that the result is Some (command was found)
		assert!(result.is_some(), "ls should be found in PATH");
	}

	// Test function to verify find_executable handles nonexistent commands
	#[test]
	fn test_find_executable_nonexistent() {
		// nonexistent_command_12345 should not be found
		let result = find_executable("nonexistent_command_12345");
		// Assert that the result is None (command was not found)
		assert!(result.is_none(), "nonexistent command should not be found");
	}

	// Test function to verify find_executable works with 'cat'
	#[test]
	fn test_find_executable_cat() {
		// cat should be found in PATH on most Unix systems
		let result = find_executable("cat");
		// Assert that the result is Some (command was found)
		assert!(result.is_some(), "cat should be found in PATH");
	}
}
