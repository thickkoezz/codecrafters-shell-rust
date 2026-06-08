// Declare the commands module (contains all builtin command implementations)
mod commands;
// Declare the user_input module (contains UserInput parsing and execution logic)
mod user_input;

// Import the rustyline library for readline-style input with history and completion
use rustyline::{
	Context,
	Editor,
	Helper,                        // Core rustyline types
	completion::{Completer, Pair}, // Tab completion types
	error::ReadlineError,          /* Error type for readline
	                                * operations */
	highlight::{CmdKind, Highlighter}, // Syntax highlighting types
	hint::Hinter,                      // Hint types (inline suggestions)
	history::DefaultHistory,           // Default history implementation
	validate::{ValidationContext, ValidationResult, Validator}, // Input validation types
};
// Import borrowed Cow, environment functions, filesystem, and path handling
use std::{borrow::Cow, cell::RefCell, env, fs, io::Write, path::Path, rc::Rc};

// Define the BuiltinCompleter struct which provides tab completion for shell commands
struct BuiltinCompleter {
	// List of builtin command names for completion
	builtin_commands: Vec<&'static str>,
	// Track the last partial input to detect repeated tab presses
	last_partial: Rc<RefCell<String>>,
	// Track tab press count for the current partial
	tab_count: Rc<RefCell<usize>>,
}

// Implement methods for BuiltinCompleter
impl BuiltinCompleter {
	// Constructor method to create a new BuiltinCompleter instance
	fn new() -> Self {
		// Initialize with the list of builtin commands and state tracking
		Self {
			builtin_commands: vec!["echo", "exit", "type", "pwd", "cd"],
			last_partial: Rc::new(RefCell::new(String::new())),
			tab_count: Rc::new(RefCell::new(0)),
		}
	}

	// Method to get executables from PATH that start with the given partial string
	fn get_path_executables(&self, partial: &str) -> Vec<String> {
		// Vector to store matching executable names
		let mut executables: Vec<String> = Vec::new();

		// Get PATH environment variable
		let path_env = match env::var("PATH") {
			Ok(p) => p,                   // If PATH is set, use it
			Err(_) => return executables, // If PATH is not set, return empty list
		};

		// Split PATH by ':' (Unix-style path separator)
		for dir in path_env.split(':') {
			// Skip empty directory entries
			if dir.is_empty() {
				continue;
			}

			// Create a Path from the directory string
			let path = Path::new(dir);

			// Skip directories that don't exist (graceful handling)
			if !path.exists() || !path.is_dir() {
				continue;
			}

			// Read directory entries
			if let Ok(entries) = fs::read_dir(path) {
				// Iterate through each entry in the directory
				for entry in entries.flatten() {
					// Get the full path of the entry
					let file_path = entry.path();

					// Check if it's a file (not a directory)
					if file_path.is_file() {
						// Check if file is executable by testing if we can access it
						// On Unix, we check the executable bit
						#[cfg(unix)]
						{
							// Import Unix-specific permissions extension
							use std::os::unix::fs::PermissionsExt;
							// Get the file metadata
							if let Ok(metadata) = fs::metadata(&file_path) {
								// Get the file permissions
								let permissions = metadata.permissions();
								// Get the mode (permission bits)
								let mode = permissions.mode();
								// Check if any execute bit is set (user, group, or other)
								// 0o111 is the mask for execute bits (binary: 001001001)
								if mode & 0o111 != 0 {
									// Extract the file name from the path
									if let Some(name) = file_path.file_name() {
										// Convert the file name to a string slice
										if let Some(name_str) = name.to_str() {
											// Only add if it starts with partial and isn't already
											// in the list (avoid duplicates)
											if name_str.starts_with(partial) &&
												!executables.contains(&name_str.to_string())
											{
												// Add the executable name to the list
												executables.push(name_str.to_string());
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}

		// Sort for consistent ordering (alphabetically)
		executables.sort();

		// Return the sorted list of executables
		executables
	}

	// Method to get files in the current directory that start with the given partial string
	fn get_current_directory_files(&self, partial: &str) -> Vec<(String, bool)> {
		// Vector to store matching file names with directory status
		let mut files: Vec<(String, bool)> = Vec::new();

		// Get the current working directory
		let current_dir = match env::current_dir() {
			Ok(dir) => dir,
			Err(_) => return files,
		};

		// Read directory entries
		if let Ok(entries) = fs::read_dir(&current_dir) {
			// Iterate through each entry in the directory
			for entry in entries.flatten() {
				// Get the file name from the entry
				if let Some(name) = entry.file_name().to_str() {
					// Only add if it starts with the partial string
					// Skip hidden files (starting with '.') unless partial also starts with '.'
					if name.starts_with(partial) &&
						(!name.starts_with('.') || partial.starts_with('.'))
					{
						// Check if this entry is a directory
						let is_dir = entry.path().is_dir();
						files.push((name.to_string(), is_dir));
					}
				}
			}
		}

		// Sort for consistent ordering (alphabetically)
		files.sort_by(|a, b| a.0.cmp(&b.0));

		// Return the sorted list of files with directory status
		files
	}

	// Method to get files in a nested path that start with the given prefix
	fn get_nested_path_files(&self, partial: &str) -> Vec<(String, bool)> {
		// Vector to store matching file names with directory status
		let mut files: Vec<(String, bool)> = Vec::new();

		// Get the current working directory
		let current_dir = match env::current_dir() {
			Ok(dir) => dir,
			Err(_) => return files,
		};

		// Find the last '/' to split directory path from prefix
		if let Some(last_slash_pos) = partial.rfind('/') {
			// Directory path includes everything up to and including the last '/'
			let dir_path = &partial[..last_slash_pos + 1];
			// Prefix is everything after the last '/'
			let prefix = &partial[last_slash_pos + 1..];

			// Build the full path to search
			// Remove trailing slash from dir_path for reliable path joining
			let search_path = if dir_path.ends_with('/') && dir_path != "/" {
				current_dir.join(&dir_path[..dir_path.len() - 1])
			} else {
				current_dir.join(dir_path)
			};

			// Read directory entries from the nested path
			if let Ok(entries) = fs::read_dir(&search_path) {
				// Iterate through each entry in the directory
				for entry in entries.flatten() {
					// Get the file name from the entry
					if let Some(name) = entry.file_name().to_str() {
						// Only add if it starts with the prefix
						// Skip hidden files (starting with '.') unless prefix also starts with '.'
						if name.starts_with(prefix) &&
							(!name.starts_with('.') || prefix.starts_with('.'))
						{
							// Check if this entry is a directory
							let is_dir = entry.path().is_dir();
							// Return the full path (directory path + file name)
							files.push((format!("{}{}", dir_path, name), is_dir));
						}
					}
				}
			}
		}

		// Sort for consistent ordering (alphabetically)
		files.sort_by(|a, b| a.0.cmp(&b.0));

		// Return the sorted list of files with directory status
		files
	}
}

// Implement the Helper trait for BuiltinCompleter (empty implementation required by rustyline)
impl Helper for BuiltinCompleter {}

// Implement the Hinter trait for BuiltinCompleter (provides inline suggestions)
impl Hinter for BuiltinCompleter {
	// The type of hint we provide (String)
	type Hint = String;
}

// Implement the Highlighter trait for BuiltinCompleter (syntax highlighting)
impl Highlighter for BuiltinCompleter {
	// Highlight a line of input (returns the line unchanged, no highlighting)
	fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
		// Return the line as-is (no syntax highlighting)
		Cow::Borrowed(line)
	}

	// Highlight a single character (returns false, no highlighting)
	fn highlight_char(&self, _line: &str, _pos: usize, _cmd: CmdKind) -> bool {
		// Return false to indicate no character highlighting
		false
	}
}

// Implement the Validator trait for BuiltinCompleter (input validation)
impl Validator for BuiltinCompleter {
	// Validate the input (always returns Valid with None modifier)
	fn validate(
		&self,
		_ctx: &mut ValidationContext<'_>, // Context for validation (unused)
	) -> Result<ValidationResult, ReadlineError> {
		// Always consider input valid with no modifications
		Ok(ValidationResult::Valid(None))
	}
}

// Implement the Completer trait for BuiltinCompleter (tab completion)
impl Completer for BuiltinCompleter {
	// The type of completion candidate we provide (Pair with display and replacement)
	type Candidate = Pair;

	// Complete the input at the given position
	fn complete(
		&self,
		line: &str,         // The full input line
		pos: usize,         // The cursor position in the line
		_ctx: &Context<'_>, // Context for completion (unused)
	) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
		// Find the start of the word being completed
		// Get the portion of the line up to the cursor position
		let line_start = &line[..pos];
		// Find the last space before the cursor (to get word start)
		let word_start = line_start.rfind(' ').map_or(0, |i| i + 1);
		// Get the partial word being completed
		let partial = &line_start[word_start..];

		// Determine if we are completing a command or an argument
		// by checking the line structure and first word
		let is_command_completion = if line_start.contains(' ') {
			// There are multiple parts - check if first word is a command
			let tokens: Vec<&str> = line_start.split_whitespace().collect();
			if tokens.len() > 1 {
				// Check if first token is a valid command (builtin or in PATH)
				let is_first_token_command = self.builtin_commands.contains(&tokens[0]) ||
					!self.get_path_executables(&tokens[0]).is_empty();
				if is_first_token_command {
					// First word is a command, so we are completing an argument
					false
				} else {
					// First word is not a known command
					true
				}
			} else {
				// Single token - check if line ends with space
				if line_start.ends_with(' ') {
					// Line ends with space - completing an argument
					false
				} else {
					// Still typing the command
					true
				}
			}
		} else {
			// No space - completing a command
			true
		};

		// Vector to store all completion matches
		let mut matches: Vec<Pair> = Vec::new();
		// HashSet to track seen names and avoid duplicates
		let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

		if !is_command_completion {
			// Completing an argument - search for files
			// Use nested path completion if partial contains a '/', otherwise use current directory
			let files = if partial.contains('/') {
				self.get_nested_path_files(partial)
			} else {
				self.get_current_directory_files(partial)
			};
			for (file_name, is_dir) in &files {
				if seen_names.insert(file_name.clone()) {
					// Use trailing slash for directories, space for files
					let trailing = if *is_dir { "/" } else { " " };
					// Display version with trailing character
					let display = format!("{}{}", file_name, trailing);
					// Replacement string (what gets inserted)
					let replacement = format!("{}{}", file_name, trailing);
					// Add the completion candidate
					matches.push(Pair { display, replacement });
				}
			}
		} else {
			// Completing a command - search for builtin commands and PATH executables
			// Find matching builtin commands
			for cmd in &self.builtin_commands {
				// Check if the builtin command starts with the partial input
				if cmd.starts_with(partial) {
					// Try to insert the command name into the seen set (returns false if already
					// present)
					if seen_names.insert(cmd.to_string()) {
						// Display version with a space after (so user can continue typing)
						let display = format!("{} ", cmd);
						// Replacement string (what gets inserted)
						let replacement = format!("{} ", cmd);
						// Add the completion candidate
						matches.push(Pair { display, replacement });
					}
				}
			}

			// Find matching executables in PATH
			let path_executables = self.get_path_executables(partial);
			for exe_name in &path_executables {
				// Try to insert the executable name into the seen set (returns false if already
				// present)
				if seen_names.insert(exe_name.clone()) {
					// Display version with a space after (so user can continue typing)
					let display = format!("{} ", exe_name);
					// Replacement string (what gets inserted)
					let replacement = format!("{} ", exe_name);
					// Add the completion candidate
					matches.push(Pair { display, replacement });
				}
			}
		}

		// Sort matches alphabetically for consistent display
		matches.sort_by(|a, b| a.display.cmp(&b.display));

		// Track tab press state for traditional shell completion behavior
		let mut last_partial = self.last_partial.borrow_mut();
		let mut tab_count = self.tab_count.borrow_mut();

		// Check if this is the same partial as last time
		if *last_partial == partial.to_string() {
			// Same partial, increment tab count
			*tab_count += 1;
		} else {
			// Different partial, reset tab count
			*last_partial = partial.to_string();
			*tab_count = 1;
		}

		// Handle no matches
		if matches.is_empty() {
			*tab_count = 0;
			return Ok((word_start, Vec::new()));
		}

		// Handle single match - complete with trailing space
		if matches.len() == 1 {
			*tab_count = 0;
			return Ok((word_start, matches));
		}

		// Multiple matches: use longest common prefix (LCP) completion
		let match_names: Vec<&str> = matches.iter().map(|p| p.display.trim()).collect();

		// Find the longest common prefix of all matches
		let lcp = match_names.iter().skip(1).fold(match_names[0].to_string(), |acc, name| {
			let mut common = String::new();
			for (a, b) in acc.chars().zip(name.chars()) {
				if a == b {
					common.push(a);
				} else {
					break;
				}
			}
			common
		});

		// If LCP is different from the partial, complete to LCP
		if lcp != *partial {
			*tab_count = 0;
			// Complete to LCP without trailing space (since there are multiple matches)
			let display = lcp.clone();
			let replacement = lcp.clone();
			return Ok((word_start, vec![Pair { display, replacement }]));
		}

		// LCP equals partial - traditional shell completion behavior
		if *tab_count == 1 {
			// First tab press - ring the bell by returning empty matches
			Ok((word_start, Vec::new()))
		} else {
			// Second (or later) tab press - display all matches
			// Store the current line and position for restoration
			let current_line = line;
			let current_pos = pos;

			// Ring the bell and print matches using terminal control codes
			print!("\x07"); // Ring the bell
			print!("\r\n{}\r\n", match_names.join("  ")); // Print matches on new line

			// Manually restore the prompt and input using ANSI escape codes
			print!("$ {}", current_line); // Print prompt and original input

			// Move cursor back to original position
			let chars_from_end = current_line.len().saturating_sub(current_pos);
			if chars_from_end > 0 {
				// Use ANSI escape code to move cursor left
				print!("\x1b[{}D", chars_from_end);
			}
			// Flush stdout to ensure output is displayed immediately
			std::io::stdout().flush().ok();

			// Reset tab count for next completion
			*tab_count = 0;
			// Return empty matches to prevent auto-completion
			Ok((word_start, Vec::new()))
		}
	}
}

// Derive Debug and Clone for RedirectionType (useful for debugging and copying)
#[derive(Debug, Clone, PartialEq)]
// Enum representing the different types of output redirection
pub enum RedirectionType {
	// Redirect stdout with truncation (overwrite): >
	Stdout,
	// Redirect stderr with truncation (overwrite): 2>
	Stderr,
	// Redirect stdout with append: >>
	StdoutAppend,
	// Redirect stderr with append: 2>>
	StderrAppend,
}

// Derive Debug and Clone for Redirection (useful for debugging and copying)
#[derive(Debug, Clone)]
// Struct representing an output redirection configuration
pub struct Redirection {
	// The file path to redirect output to
	pub file: String,
	// The type of redirection (stdout/stderr, append/truncate)
	pub redirection_type: RedirectionType,
}

// Function to tokenize input string into a vector of strings
// Handles quotes (single and double), escapes, and redirection operators
fn tokenize(input: &str) -> Vec<String> {
	// Vector to store the resulting tokens
	let mut tokens = Vec::new();
	// Peekable iterator over input characters (allows looking ahead)
	let mut chars = input.chars().peekable();
	// String buffer for the current token being built
	let mut current_token = String::new();

	// Process each character in the input
	while let Some(&c) = chars.peek() {
		// Match on the current character
		match c {
			'\'' => {
				// Start of single quoted string
				chars.next(); // consume opening quote
				// String buffer for quoted content
				let mut quoted_content = String::new();

				// Process characters until closing quote
				while let Some(&c) = chars.peek() {
					match c {
						'\'' => {
							// Found closing quote
							chars.next(); // consume closing quote
							// Append quoted content to current token (concatenation)
							current_token.push_str(&quoted_content);
							// Break out of the inner loop
							break;
						},
						_ => {
							// Regular character inside quotes
							// Consume and add to quoted content
							quoted_content.push(chars.next().unwrap());
						},
					}
				}
			},
			'"' => {
				// Start of double quoted string
				chars.next(); // consume opening quote
				// String buffer for quoted content
				let mut quoted_content = String::new();

				// Process characters until closing quote
				while let Some(&c) = chars.peek() {
					match c {
						'"' => {
							// Found closing quote
							chars.next(); // consume closing quote
							// Append quoted content to current token (concatenation)
							current_token.push_str(&quoted_content);
							// Break out of the inner loop
							break;
						},
						'\\' => {
							// Backslash escape sequence
							chars.next(); // consume the backslash
							// Check if there's a next character
							if let Some(&next_c) = chars.peek() {
								match next_c {
									'"' | '\\' => {
										// Escaped quote or backslash - add the literal character
										quoted_content.push(chars.next().unwrap());
									},
									_ => {
										// For all other characters, backslash is treated literally
										// So we add both the backslash and the next character
										quoted_content.push('\\');
										quoted_content.push(chars.next().unwrap());
									},
								}
							} else {
								// Backslash at end of string - treat literally
								quoted_content.push('\\');
							}
						},
						_ => {
							// Regular character inside quotes
							// Consume and add to quoted content
							quoted_content.push(chars.next().unwrap());
						},
					}
				}
			},
			' ' | '\t' => {
				// Whitespace outside quotes - delimiter
				chars.next(); // consume the whitespace character
				// If there's a current token, add it to the tokens list
				if !current_token.is_empty() {
					tokens.push(current_token.clone());
					// Clear the current token buffer
					current_token.clear();
				}
			},
			'>' => {
				// Redirection operator - treat as delimiter
				// If there's a current token, add it to the tokens list
				if !current_token.is_empty() {
					tokens.push(current_token.clone());
					// Clear the current token buffer
					current_token.clear();
				}
				chars.next(); // consume the first '>'
				// Check if this is >> (append mode)
				if let Some(&'>') = chars.peek() {
					chars.next(); // consume the second '>'
					// Add ">>" as a token
					tokens.push(">>".to_string());
				} else {
					// Single ">" redirection
					tokens.push(">".to_string());
				}
			},
			'2' => {
				// Check if this is the start of "2>" or "2>>" (stderr redirection)
				chars.next(); // consume the '2'
				// Check if the next character is '>'
				if let Some(&'>') = chars.peek() {
					chars.next(); // consume the first '>'
					// Check if this is 2>> (append mode)
					if let Some(&'>') = chars.peek() {
						chars.next(); // consume the second '>'
						// Add "2>>" as a token
						tokens.push("2>>".to_string());
					} else {
						// Single "2>" redirection
						tokens.push("2>".to_string());
					}
				} else {
					// Just a regular '2' character, add to current token
					current_token.push('2');
				}
			},
			'\\' => {
				// Backslash escapes the next character (outside quotes)
				chars.next(); // consume the backslash
				// Check if there's a next character to escape
				if let Some(&_next_char) = chars.peek() {
					// Add the next character literally (without the backslash)
					current_token.push(chars.next().unwrap());
				}
				// If there's no next character, the backslash is just ignored
			},
			_ => {
				// Regular character, add to current token
				current_token.push(chars.next().unwrap());
			},
		}
	}

	// Don't forget the last token (if not empty)
	if !current_token.is_empty() {
		tokens.push(current_token);
	}

	// Return the vector of tokens
	tokens
}

// Main function - entry point of the shell program
fn main() {
	// Create a new rustyline Editor with BuiltinCompleter and DefaultHistory
	let mut rl =
		Editor::<BuiltinCompleter, DefaultHistory>::new().expect("Failed to create editor");
	// Set the helper (our completer) for the editor
	rl.set_helper(Some(BuiltinCompleter::new()));

	// Main REPL (Read-Eval-Print Loop)
	loop {
		// Read the line of input from the user with tab completion support
		let input = match rl.readline("$ ") {
			// If the line was read successfully
			Ok(line) => line,
			// If Ctrl+D (EOF) was pressed
			Err(rustyline::error::ReadlineError::Eof) => {
				// Exit the loop (terminate the shell)
				break;
			},
			// If Ctrl+C was pressed
			Err(rustyline::error::ReadlineError::Interrupted) => {
				// Continue to the next iteration (ignore the interrupt)
				continue;
			},
			// For any other error
			Err(error) => {
				// Print the error to stderr
				eprintln!("Error reading input: {}", error);
				// Exit the loop (terminate the shell)
				break;
			},
		};

		// Add the input to history (for up-arrow recall)
		rl.add_history_entry(input.as_str()).ok();

		// Clean up trailing newline characters (\n or \r\n)
		let trimmed = input.trim();

		// Skip empty inputs (if user just hits Enter)
		if trimmed.is_empty() {
			continue;
		}

		// Parse the input into tokens with quote support
		let tokens = tokenize(trimmed);

		// Check for redirection operator
		let mut redirection: Option<Redirection> = None;
		// Index where command tokens end (before redirection)
		let mut command_end = tokens.len();

		// Handle "1>" and "1>>" syntax (same as ">" and ">>")
		let mut tokens_to_process = tokens.clone();
		let mut i = 0;
		// Iterate through tokens looking for "1>" or "1>>" patterns
		while i < tokens_to_process.len() {
			// Check if current token is "1" and next token is ">" or ">>"
			if tokens_to_process[i] == "1" &&
				i + 1 < tokens_to_process.len() &&
				(tokens_to_process[i + 1] == ">" || tokens_to_process[i + 1] == ">>")
			{
				// Remove "1" since "1>" is same as ">" and "1>>" is same as ">>"
				tokens_to_process.remove(i);
				// Break out of the loop (found and handled)
				break;
			}
			// Move to the next token
			i += 1;
		}

		// Find the redirection operators: '>', '>>', '2>', '2>>'
		for (i, token) in tokens_to_process.iter().enumerate() {
			// Check for stdout redirect with truncate
			if token == ">" {
				// Check if there's a filename after '>'
				if i + 1 < tokens_to_process.len() {
					// Create the redirection configuration
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::Stdout,
					});
					// Set command_end to current index (command tokens end before redirection)
					command_end = i;
					// Break out of the loop (found the redirection)
					break;
				}
			} else if token == ">>" {
				// Check if there's a filename after '>>'
				if i + 1 < tokens_to_process.len() {
					// Create the redirection configuration
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::StdoutAppend,
					});
					// Set command_end to current index (command tokens end before redirection)
					command_end = i;
					// Break out of the loop (found the redirection)
					break;
				}
			} else if token == "2>" {
				// Check if there's a filename after '2>'
				if i + 1 < tokens_to_process.len() {
					// Create the redirection configuration
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::Stderr,
					});
					// Set command_end to current index (command tokens end before redirection)
					command_end = i;
					// Break out of the loop (found the redirection)
					break;
				}
			} else if token == "2>>" {
				// Check if there's a filename after '2>>'
				if i + 1 < tokens_to_process.len() {
					// Create the redirection configuration
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::StderrAppend,
					});
					// Set command_end to current index (command tokens end before redirection)
					command_end = i;
					// Break out of the loop (found the redirection)
					break;
				}
			}
		}

		// Extract command and args (excluding redirection part)
		let command_tokens = &tokens_to_process[..command_end];
		// Skip if there are no command tokens (only redirection was provided)
		if command_tokens.is_empty() {
			continue;
		}

		// Create a UserInput from the parsed tokens
		let user_input = user_input::UserInput::new(
			// First token is the command name
			command_tokens.first().unwrap().clone(),
			// Remaining tokens are the arguments
			command_tokens.iter().skip(1).cloned().collect(),
			// Redirection configuration (if any)
			redirection,
		);

		// Evaluate the command
		match user_input.command.as_str() {
			// If command is "exit", break the loop (terminate the shell)
			"exit" => break,
			// For all other commands, evaluate and execute
			_ => user_input.evaluate_command(),
		}
	}
}

// Conditional compilation attribute - only compile the following module when running tests
#[cfg(test)]
mod tests {
	// Import items from parent module and std library
	use super::*;
	use std::{env, fs, path::Path};

	// Test function to verify get_path_executables works correctly
	#[test]
	fn test_get_path_executables() {
		// Create a completer instance
		let completer = BuiltinCompleter::new();

		// Save original PATH (so we can restore it later)
		let original_path = env::var("PATH").unwrap_or_default();

		// Create a temporary directory with a test executable
		let temp_dir = "/tmp/test_shell_completion";
		fs::create_dir_all(temp_dir).unwrap();

		// Create a test executable file
		let test_exe = Path::new(temp_dir).join("test_executable");
		fs::write(&test_exe, "#!/bin/sh\necho test").unwrap();

		// Make the file executable (Unix-specific)
		#[cfg(unix)]
		{
			use std::os::unix::fs::PermissionsExt;
			// Get current permissions
			let mut perms = fs::metadata(&test_exe).unwrap().permissions();
			// Set executable bit (0o755 = rwxr-xr-x)
			perms.set_mode(0o755);
			// Apply the new permissions
			fs::set_permissions(&test_exe, perms).unwrap();
		}

		// Set PATH to only include our temp directory (unsafe because it modifies process state)
		unsafe {
			env::set_var("PATH", temp_dir);
		};

		// Test that we can find the executable
		let results = completer.get_path_executables("test");
		// Assert that "test_executable" is in the results
		assert!(results.contains(&"test_executable".to_string()));

		// Test that non-matching partial returns empty
		let results = completer.get_path_executables("nomatch");
		// Assert that no executables match "nomatch"
		assert!(results.is_empty());

		// Cleanup - remove the temp directory
		fs::remove_dir_all(temp_dir).unwrap();
		// Restore original PATH (unsafe because it modifies process state)
		unsafe {
			env::set_var("PATH", original_path);
		};
	}
}
