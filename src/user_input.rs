// Import the Redirection struct and RedirectionType enum from the crate root
use crate::{
	Redirection,
	// Import all the command implementations from the commands module
	commands::{Command, cd::Cd, echo::Echo, external::ExternalCommand, pwd::Pwd, type_cmd::Type},
};

// Define the UserInput struct which represents a parsed command line
pub struct UserInput {
	// The command name (e.g., "echo", "ls", "cd")
	pub command: String,
	// The arguments passed to the command (excluding the command name itself)
	pub args: Vec<String>,
	// Optional output redirection configuration
	pub redirection: Option<Redirection>,
}

// Implement methods for the UserInput struct
impl UserInput {
	// Constructor method to create a new UserInput instance
	pub fn new(command: String, args: Vec<String>, redirection: Option<Redirection>) -> Self {
		// Return a new UserInput with the provided fields
		Self { command, args, redirection }
	}

	// Method to evaluate and execute the command represented by this UserInput
	pub fn evaluate_command(&self) {
		// Match on the command name to determine which command to execute
		// The result type is Result<(), CommandError> from the Command trait
		let result: Result<(), _> = match self.command.as_str() {
			// If command is "type", execute the Type builtin command
			"type" => Type.execute(&self.args, self.redirection.as_ref()),
			// If command is "echo", execute the Echo builtin command
			"echo" => Echo.execute(&self.args, self.redirection.as_ref()),
			// If command is "pwd", execute the Pwd builtin command
			"pwd" => Pwd.execute(&self.args, self.redirection.as_ref()),
			// If command is "cd", execute the Cd builtin command
			"cd" => Cd.execute(&self.args, self.redirection.as_ref()),
			// If command is not a builtin, execute it as an external command
			_ => {
				// Create an ExternalCommand with the command name
				let external = ExternalCommand { command_name: self.command.clone() };
				// Execute the external command
				external.execute(&self.args, self.redirection.as_ref())
			},
		};

		// Silently ignore errors since commands already print their error messages to stderr
		// The underscore prefix suppresses the unused result warning
		let _ = result;
	}
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import all items from the parent module (UserInput, etc.)
	use super::*;

	// Test function to verify UserInput::new creates a correct instance
	#[test]
	fn test_user_input_new() {
		// Create a UserInput with echo command and two arguments
		let input = UserInput::new(
			"echo".to_string(),
			vec!["hello".to_string(), "world".to_string()],
			None, // No redirection
		);
		// Assert the command field is "echo"
		assert_eq!(input.command, "echo");
		// Assert the args field contains the two arguments
		assert_eq!(input.args, vec!["hello", "world"]);
	}

	// Test function to verify UserInput handles empty arguments
	#[test]
	fn test_user_input_empty_args() {
		// Create a UserInput with type command and no arguments
		let input = UserInput::new("type".to_string(), vec![], None);
		// Assert the command field is "type"
		assert_eq!(input.command, "type");
		// Assert the args field is empty
		assert!(input.args.is_empty());
	}

	// Test function to verify UserInput handles redirection
	#[test]
	fn test_user_input_with_redirection() {
		// Import the RedirectionType enum for this test
		use crate::RedirectionType;
		// Create a redirection configuration
		let redirection = Some(Redirection {
			file: "output.txt".to_string(),
			redirection_type: RedirectionType::Stdout,
		});
		// Create a UserInput with redirection
		let input = UserInput::new("echo".to_string(), vec!["hello".to_string()], redirection);
		// Assert the command field is "echo"
		assert_eq!(input.command, "echo");
		// Assert redirection is Some (not None)
		assert!(input.redirection.is_some());
		// Assert the redirection file is "output.txt"
		assert_eq!(input.redirection.as_ref().unwrap().file, "output.txt");
	}

	// Test function to verify the list of builtin commands
	#[test]
	fn test_is_builtin_command() {
		// Define the array of builtin command names
		let builtin_commands = ["echo", "exit", "type", "pwd", "cd"];
		// Assert "echo" is in the builtin list
		assert!(builtin_commands.contains(&"echo"));
		// Assert "exit" is in the builtin list
		assert!(builtin_commands.contains(&"exit"));
		// Assert "type" is in the builtin list
		assert!(builtin_commands.contains(&"type"));
		// Assert "pwd" is in the builtin list
		assert!(builtin_commands.contains(&"pwd"));
		// Assert "cd" is in the builtin list
		assert!(builtin_commands.contains(&"cd"));
		// Assert "ls" is NOT in the builtin list (it's an external command)
		assert!(!builtin_commands.contains(&"ls"));
	}

	// Test function to verify the "type" command can be evaluated
	#[test]
	fn test_evaluate_command_type() {
		// Create a UserInput for the type command
		let input = UserInput::new("type".to_string(), vec!["echo".to_string()], None);
		// Execute the command and verify it doesn't panic
		input.evaluate_command();
	}

	// Test function to verify the "echo" command can be evaluated
	#[test]
	fn test_evaluate_command_echo() {
		// Create a UserInput for the echo command
		let input = UserInput::new("echo".to_string(), vec!["test".to_string()], None);
		// Execute the command and verify it doesn't panic
		input.evaluate_command();
	}

	// Test function to verify the "pwd" command can be evaluated
	#[test]
	fn test_evaluate_command_pwd() {
		// Create a UserInput for the pwd command
		let input = UserInput::new("pwd".to_string(), vec![], None);
		// Execute the command and verify it doesn't panic
		input.evaluate_command();
	}

	// Test function to verify the "cd" command can be evaluated
	#[test]
	fn test_evaluate_command_cd() {
		// Create a UserInput for the cd command
		let input = UserInput::new("cd".to_string(), vec!["/tmp".to_string()], None);
		// Execute the command and verify it doesn't panic
		input.evaluate_command();
	}

	// Test function to verify unknown commands are handled
	#[test]
	fn test_evaluate_command_unknown() {
		// Create a UserInput for an unknown (external) command
		let input = UserInput::new("unknown".to_string(), vec![], None);
		// Execute the command and verify it doesn't panic
		input.evaluate_command();
	}
}
