// Import the Redirection struct from the crate root (defined in main.rs)
use crate::Redirection;
// Import the fmt module for implementing Display trait
use std::fmt;

// Declare the cd module - contains the cd (change directory) builtin command
pub mod cd;
// Declare the echo module - contains the echo builtin command
pub mod echo;
// Declare the external module - contains external command execution logic
pub mod external;
// Declare the pwd module - contains the pwd (print working directory) builtin command
pub mod pwd;
// Declare the type_cmd module - contains the type builtin command
pub mod type_cmd;
// Declare the utils module - contains utility functions like find_executable
pub mod utils;

/// Error type for command execution
/// This struct wraps error messages for commands that fail during execution
#[derive(Debug)] // Derive Debug trait for error formatting
pub struct CommandError {
	// Public field containing the error message string
	pub message: String,
}

// Implement the Display trait for CommandError to enable user-friendly error output
impl fmt::Display for CommandError {
	// Implementation of the fmt method required by the Display trait
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Write the message field to the formatter
		write!(f, "{}", self.message)
	}
}

// Implement the standard Error trait for CommandError
// This allows CommandError to be used with error handling idioms
impl std::error::Error for CommandError {}

/// Trait that all commands must implement
/// This is the core abstraction that enables polymorphic command execution
pub trait Command {
	/// Execute the command with the given arguments and optional redirection
	/// All builtin commands implement this method
	fn execute(
		&self,                        // Borrow self immutably (no mutable state needed)
		args: &[String],              // Slice of command line arguments
		redirection: Option<&Redirection>, // Optional output redirection configuration
	) -> Result<(), CommandError>;   // Return unit on success, CommandError on failure
}
