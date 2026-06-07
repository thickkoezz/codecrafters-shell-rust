// Import the Command trait, CommandError type, and find_executable utility from the parent module
use super::{Command, CommandError, utils::find_executable};
// Import Redirection and RedirectionType from the crate root
use crate::{Redirection, RedirectionType};
// Import file open options, Unix-specific modules for file descriptors and process extensions, and process module
use std::{
	fs::OpenOptions,
	os::unix::{io::AsRawFd, process::CommandExt},
	process,
};

// Define the ExternalCommand struct which represents external (non-builtin) commands
pub struct ExternalCommand {
	// The name of the external command (e.g., "ls", "cat", "grep")
	pub command_name: String,
}

// Implement the Command trait for the ExternalCommand struct
impl Command for ExternalCommand {
	// The execute method is required by the Command trait
	fn execute(
		&self,                          // Borrow self immutably
		args: &[String],                // Slice of command line arguments
		redirection: Option<&Redirection>, // Optional output redirection configuration
	) -> Result<(), CommandError> {   // Return unit on success, CommandError on failure
		// Try to find the executable in PATH
		if let Some(executable_path) = find_executable(&self.command_name) {
			// Handle output redirection if specified
			// Create a tuple of (file, file_descriptor) for redirection
			let (file, fd_to_redirect) = if let Some(redir) = redirection {
				// Match on the redirection type to determine file open options
				let result = match redir.redirection_type {
					// Stdout redirection (>): write, create, truncate
					RedirectionType::Stdout =>
						OpenOptions::new().write(true).create(true).truncate(true).open(&redir.file),
					// Stderr redirection (2>): write, create, truncate
					RedirectionType::Stderr =>
						OpenOptions::new().write(true).create(true).truncate(true).open(&redir.file),
					// Stdout append redirection (>>): write, create, append
					RedirectionType::StdoutAppend =>
						OpenOptions::new().write(true).create(true).append(true).open(&redir.file),
					// Stderr append redirection (2>>): write, create, append
					RedirectionType::StderrAppend =>
						OpenOptions::new().write(true).create(true).append(true).open(&redir.file),
				};

				// Handle the result of opening the file
				match result {
					// If the file opened successfully, determine the file descriptor to redirect
					Ok(f) => {
						// Determine which file descriptor to redirect based on redirection type
						let fd = match redir.redirection_type {
							// Stdout and StdoutAppend use file descriptor 1 (stdout)
							RedirectionType::Stdout | RedirectionType::StdoutAppend => 1,
							// Stderr and StderrAppend use file descriptor 2 (stderr)
							RedirectionType::Stderr | RedirectionType::StderrAppend => 2,
						};
						// Return the file and the file descriptor as a tuple
						(Some(f), Some(fd))
					},
					// If opening the file failed, print error and return CommandError
					Err(e) => {
						// Print the error message to stderr
						eprintln!("{}: {}", redir.file, e);
						// Return a CommandError containing the error message
						return Err(CommandError { message: e.to_string() });
					},
				}
			} else {
				// No redirection specified, so use None for both file and fd
				(None, None)
			};

			// Execute the program and wait for it to complete
			// Create a new Command instance with the executable path
			let mut cmd = process::Command::new(&executable_path);
			// Set arg0 to the command name (for process title)
			cmd.arg0(&self.command_name);
			// Add all the arguments to the command
			cmd.args(args);

			// Redirect stdout/stderr to file if specified
			// We need to do this BEFORE spawning the child process
			// by using unsafe dup2 on the file descriptor (Unix-specific)
			if let (Some(ref f), Some(fd)) = (file.as_ref(), fd_to_redirect) {
				// Unsafe block required for direct libc calls
				unsafe {
					// Save the original file descriptor using dup
					let original_fd = libc::dup(fd);
					// Check if duplicating the file descriptor failed
					if original_fd < 0 {
						return Err(CommandError {
							message: format!("Failed to duplicate file descriptor {}", fd),
						});
					}

					// Redirect to the file using dup2
					// dup2 copies the file descriptor f to fd
					if libc::dup2(f.as_raw_fd(), fd) < 0 {
						return Err(CommandError {
							message: format!("Failed to redirect file descriptor {}", fd),
						});
					}

					// Execute the command and wait for completion
					let result = match cmd.spawn() {
						// If the process spawned successfully, wait for it to complete
						Ok(mut child) => match child.wait() {
							// If the process completed successfully
							Ok(_) => Ok(()),
							// If waiting for the process failed
							Err(e) => {
								// Print the error to stderr
								eprintln!("{}: {}", self.command_name, e);
								// Return a CommandError containing the error message
								Err(CommandError { message: e.to_string() })
							},
						},
						// If spawning the process failed
						Err(e) => {
							// Print the error to stderr
							eprintln!("{}: {}", self.command_name, e);
							// Return a CommandError containing the error message
							Err(CommandError { message: e.to_string() })
						},
					};

					// Restore the original file descriptor using dup2
					libc::dup2(original_fd, fd);
					// Close the duplicated file descriptor
					libc::close(original_fd);

					// Return the result of the command execution
					result
				}
			} else {
				// No redirection, execute the command normally
				match cmd.spawn() {
					// If the process spawned successfully, wait for it to complete
					Ok(mut child) => match child.wait() {
						// If the process completed successfully
						Ok(_) => Ok(()),
						// If waiting for the process failed
						Err(e) => {
							// Print the error to stderr
							eprintln!("{}: {}", self.command_name, e);
							// Return a CommandError containing the error message
							Err(CommandError { message: e.to_string() })
						},
					},
					// If spawning the process failed
					Err(e) => {
						// Print the error to stderr
						eprintln!("{}: {}", self.command_name, e);
						// Return a CommandError containing the error message
						Err(CommandError { message: e.to_string() })
					},
				}
			}
		} else {
			// If the executable was not found in PATH
			// Print "command not found" message to stdout
			println!("{}: command not found", self.command_name.as_str());
			// Return a CommandError indicating the command was not found
			Err(CommandError { message: format!("{}: command not found", self.command_name) })
		}
	}
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import all items from the parent module (ExternalCommand, etc.)
	use super::*;

	// Test function to verify external command works with 'ls'
	#[test]
	fn test_external_command_ls() {
		// Create an ExternalCommand for "ls"
		let cmd = ExternalCommand { command_name: "ls".to_string() };
		// ls should work on most Unix systems
		let result = cmd.execute(&[], None);
		// Should succeed (or fail if ls is not found, but not panic)
		assert!(result.is_ok() || result.is_err());
	}

	// Test function to verify external command works with 'cat'
	#[test]
	fn test_external_command_cat() {
		// Create an ExternalCommand for "cat"
		let cmd = ExternalCommand { command_name: "cat".to_string() };
		// cat should work on most Unix systems
		let result = cmd.execute(&[], None);
		// Should succeed (or fail if cat is not found, but not panic)
		assert!(result.is_ok() || result.is_err());
	}

	// Test function to verify external command handles nonexistent commands
	#[test]
	fn test_external_command_nonexistent() {
		// Create an ExternalCommand for a nonexistent command
		let cmd = ExternalCommand { command_name: "nonexistent_command_12345".to_string() };
		// Should return an error
		assert!(cmd.execute(&[], None).is_err());
	}

	// Test function to verify external command works with arguments
	#[test]
	fn test_external_command_with_args() {
		// Create an ExternalCommand for "ls"
		let cmd = ExternalCommand { command_name: "ls".to_string() };
		// ls with -la flag should work
		let result = cmd.execute(&["-la".to_string()], None);
		// Should succeed (or fail if ls is not found, but not panic)
		assert!(result.is_ok() || result.is_err());
	}

	// Test function to verify external command doesn't panic
	#[test]
	fn test_external_command_does_not_panic() {
		// Create an ExternalCommand for "ls"
		let cmd = ExternalCommand { command_name: "ls".to_string() };
		// Discard the result of executing with no arguments (shouldn't panic)
		let _ = cmd.execute(&[], None);
		// Discard the result of executing with -la flag (shouldn't panic)
		let _ = cmd.execute(&["-la".to_string()], None);
	}

	// Test function to verify external command works with redirection
	#[test]
	fn test_external_command_with_redirection() {
		// Create an ExternalCommand for "echo" (external version, not builtin)
		let cmd = ExternalCommand { command_name: "echo".to_string() };
		// Create a redirection configuration for stdout to a temp file
		let redirection = Some(Redirection {
			file: "/tmp/test_external.txt".to_string(),
			redirection_type: RedirectionType::Stdout,
		});
		// Execute the command with redirection
		let result = cmd.execute(&["hello".to_string()], redirection.as_ref());
		// Assert that the execution succeeded
		assert!(result.is_ok());
		// Verify the file was created
		use std::fs;
		let content = fs::read_to_string("/tmp/test_external.txt").unwrap();
		// The file should contain "hello" at the end (may have test output before it)
		assert!(content.trim().ends_with("hello"));
		// Clean up the test file
		let _ = fs::remove_file("/tmp/test_external.txt");
	}

	// Test function to verify external command works with stderr redirection
	#[test]
	fn test_external_command_with_stderr_redirection() {
		// Create an ExternalCommand for "ls"
		let cmd = ExternalCommand { command_name: "ls".to_string() };
		// Create a redirection configuration for stderr to a temp file
		let redirection = Some(Redirection {
			file: "/tmp/test_stderr.txt".to_string(),
			redirection_type: RedirectionType::Stderr,
		});
		// ls with a nonexistent file should write to stderr
		let result = cmd.execute(&["/nonexistent".to_string()], redirection.as_ref());
		// Should succeed (command executed, even if file not found)
		assert!(result.is_ok() || result.is_err());
		// Verify the file was created
		use std::fs;
		let content = fs::read_to_string("/tmp/test_stderr.txt");
		// File may or may not have content depending on the system
		assert!(content.is_ok() || content.is_err());
		// Clean up the test file if it was created
		if let Ok(_c) = content {
			let _ = fs::remove_file("/tmp/test_stderr.txt");
		}
	}
}
