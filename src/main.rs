// Declare the modules
mod commands;
mod completion;
mod user_input;

// Import the rustyline library for readline-style input with history
use rustyline::{Editor, history::DefaultHistory};

// Import Redirection types from the completion module
use completion::BuiltinCompleter;

// Derive Debug and Clone for RedirectionType
#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionType {
	Stdout,
	Stderr,
	StdoutAppend,
	StderrAppend,
}

// Derive Debug and Clone for Redirection
#[derive(Debug, Clone)]
pub struct Redirection {
	pub file: String,
	pub redirection_type: RedirectionType,
}

// Function to tokenize input string into a vector of strings
// Handles quotes (single and double), escapes, and redirection operators
fn tokenize(input: &str) -> Vec<String> {
	let mut tokens = Vec::new();
	let mut chars = input.chars().peekable();
	let mut current_token = String::new();

	while let Some(&c) = chars.peek() {
		match c {
			'\'' => {
				chars.next();
				let mut quoted_content = String::new();

				while let Some(&c) = chars.peek() {
					match c {
						'\'' => {
							chars.next();
							current_token.push_str(&quoted_content);
							break;
						},
						_ => {
							quoted_content.push(chars.next().unwrap());
						},
					}
				}
			},
			'"' => {
				chars.next();
				let mut quoted_content = String::new();

				while let Some(&c) = chars.peek() {
					match c {
						'"' => {
							chars.next();
							current_token.push_str(&quoted_content);
							break;
						},
						'\\' => {
							chars.next();
							if let Some(&next_c) = chars.peek() {
								match next_c {
									'"' | '\\' => {
										quoted_content.push(chars.next().unwrap());
									},
									_ => {
										quoted_content.push('\\');
										quoted_content.push(chars.next().unwrap());
									},
								}
							}
						},
						_ => {
							quoted_content.push(chars.next().unwrap());
						},
					}
				}
			},
			' ' | '\t' => {
				chars.next();
				if !current_token.is_empty() {
					tokens.push(current_token.clone());
					current_token.clear();
				}
			},
			'>' => {
				if !current_token.is_empty() {
					tokens.push(current_token.clone());
					current_token.clear();
				}
				chars.next();
				if let Some(&'>') = chars.peek() {
					chars.next();
					tokens.push(">>".to_string());
				} else {
					tokens.push(">".to_string());
				}
			},
			'2' => {
				chars.next();
				if let Some(&'>') = chars.peek() {
					chars.next();
					if let Some(&'>') = chars.peek() {
						chars.next();
						tokens.push("2>>".to_string());
					} else {
						tokens.push("2>".to_string());
					}
				} else {
					current_token.push('2');
				}
			},
			'\\' => {
				chars.next();
				if let Some(&_next_char) = chars.peek() {
					current_token.push(chars.next().unwrap());
				}
			},
			_ => {
				current_token.push(chars.next().unwrap());
			},
		}
	}

	if !current_token.is_empty() {
		tokens.push(current_token);
	}

	tokens
}

// Main function - entry point of the shell program
fn main() {
	let mut rl =
		Editor::<BuiltinCompleter, DefaultHistory>::new().expect("Failed to create editor");
	rl.set_helper(Some(BuiltinCompleter::new()));

	loop {
		let input = match rl.readline("$ ") {
			Ok(line) => line,
			Err(rustyline::error::ReadlineError::Eof) => {
				break;
			},
			Err(rustyline::error::ReadlineError::Interrupted) => {
				continue;
			},
			Err(error) => {
				eprintln!("Error reading input: {}", error);
				break;
			},
		};

		rl.add_history_entry(input.as_str()).ok();

		let trimmed = input.trim();

		if trimmed.is_empty() {
			continue;
		}

		let tokens = tokenize(trimmed);

		let mut redirection: Option<Redirection> = None;
		let mut command_end = tokens.len();
		let mut tokens_to_process = tokens.clone();
		let mut i = 0;

		while i < tokens_to_process.len() {
			if tokens_to_process[i] == "1" &&
				i + 1 < tokens_to_process.len() &&
				(tokens_to_process[i + 1] == ">" || tokens_to_process[i + 1] == ">>")
			{
				tokens_to_process.remove(i);
				break;
			}
			i += 1;
		}

		for (i, token) in tokens_to_process.iter().enumerate() {
			if token == ">" {
				if i + 1 < tokens_to_process.len() {
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::Stdout,
					});
					command_end = i;
					break;
				}
			} else if token == ">>" {
				if i + 1 < tokens_to_process.len() {
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::StdoutAppend,
					});
					command_end = i;
					break;
				}
			} else if token == "2>" {
				if i + 1 < tokens_to_process.len() {
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::Stderr,
					});
					command_end = i;
					break;
				}
			} else if token == "2>>" {
				if i + 1 < tokens_to_process.len() {
					redirection = Some(Redirection {
						file: tokens_to_process[i + 1].clone(),
						redirection_type: RedirectionType::StderrAppend,
					});
					command_end = i;
					break;
				}
			}
		}

		let command_tokens = &tokens_to_process[..command_end];

		if command_tokens.is_empty() {
			continue;
		}

		let user_input = user_input::UserInput::new(
			command_tokens.first().unwrap().clone(),
			command_tokens.iter().skip(1).cloned().collect(),
			redirection,
		);

		match user_input.command.as_str() {
			"exit" => break,
			_ => user_input.evaluate_command(),
		}
	}
}
