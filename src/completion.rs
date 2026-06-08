// Completion module for tab completion functionality
use rustyline::{
	Context, Helper,
	completion::Pair,
	highlight::{CmdKind, Highlighter},
	hint::Hinter,
	validate::{ValidationContext, ValidationResult, Validator},
};
use std::{borrow::Cow, cell::RefCell, env, fs, io::Write, path::Path};

// Define the BuiltinCompleter struct which provides tab completion for shell commands
pub struct BuiltinCompleter {
	// List of builtin command names for completion
	builtin_commands: Vec<&'static str>,
}

impl BuiltinCompleter {
	// Constructor method to create a new BuiltinCompleter instance
	pub fn new() -> Self {
		// Initialize with the list of builtin commands
		Self {
			builtin_commands: vec!["echo", "exit", "type", "pwd", "cd"],
		}
	}

	// Method to get executables from PATH that start with the given partial string
	pub fn get_path_executables(&self, partial: &str) -> Vec<String> {
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
	pub fn get_current_directory_files(&self, partial: &str) -> Vec<(String, bool)> {
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
	pub fn get_nested_path_files(&self, partial: &str) -> Vec<(String, bool)> {
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
	) -> Result<ValidationResult, rustyline::error::ReadlineError> {
		// Always consider input valid with no modifications
		Ok(ValidationResult::Valid(None))
	}
}

// Implement the Completer trait for BuiltinCompleter (tab completion)
impl rustyline::completion::Completer for BuiltinCompleter {
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
		let is_command_completion = if line_start.contains(' ') {
			let tokens: Vec<&str> = line_start.split_whitespace().collect();
			if tokens.len() > 1 {
				false // Multiple tokens = argument completion
			} else {
				if line_start.ends_with(' ') { false } else { true }
			}
		} else {
			true
		};

		// Vector to store all completion matches
		let mut matches: Vec<Pair> = Vec::new();
		let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

		if !is_command_completion {
			// Completing an argument - search for files
			let files = if partial.contains('/') {
				self.get_nested_path_files(partial)
			} else {
				self.get_current_directory_files(partial)
			};
			for (file_name, is_dir) in &files {
				if seen_names.insert(file_name.clone()) {
					let trailing = if *is_dir { "/" } else { " " };
					let display = format!("{}{}", file_name, trailing);
					let replacement = format!("{}{}", file_name, trailing);
					matches.push(Pair { display, replacement });
				}
			}
		} else {
			// Completing a command
			for cmd in &self.builtin_commands {
				if cmd.starts_with(partial) {
					if seen_names.insert(cmd.to_string()) {
						let display = format!("{} ", cmd);
						let replacement = format!("{} ", cmd);
						matches.push(Pair { display, replacement });
					}
				}
			}

			let path_executables = self.get_path_executables(partial);
			for exe_name in &path_executables {
				if seen_names.insert(exe_name.clone()) {
					let display = format!("{} ", exe_name);
					let replacement = format!("{} ", exe_name);
					matches.push(Pair { display, replacement });
				}
			}
		}

		matches.sort_by(|a, b| a.display.cmp(&b.display));

		// Simplified logic: no state tracking for repeat TABs
		if matches.is_empty() {
			return Ok((word_start, Vec::new()));
		}

		if matches.len() == 1 {
			return Ok((word_start, matches));
		}

		// Multiple matches - calculate least common prefix and show matches on second TAB
		let match_names: Vec<&str> = matches.iter().map(|p| p.display.trim()).collect();

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

		if lcp != *partial && !lcp.is_empty() {
			// Return LCP for completion
			let display = lcp.clone();
			let replacement = lcp.clone();
			return Ok((word_start, vec![Pair { display, replacement }]));
		}

		// Show all matches
		let current_line = line;
		let current_pos = pos;

		print!("\x07");
		print!("\r\n{}\r\n", match_names.join("  "));
		print!("$ {}", current_line);

		let chars_from_end = current_line.len().saturating_sub(current_pos);
		if chars_from_end > 0 {
			print!("\x1b[{}D", chars_from_end);
		}
		std::io::stdout().flush().ok();

		Ok((word_start, Vec::new()))
	}
}
