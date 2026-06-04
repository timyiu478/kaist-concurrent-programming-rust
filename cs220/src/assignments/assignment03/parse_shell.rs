//! Parsing a shell command.
//!
//! Shell commands are text-based instructions that you can enter in a command-line interface (CLI)
//! to interact with operating systems (e.g. Linux) and others. For example, you can use the `ls`
//! command to list files in a directory.
//!
//! You will parse a given string consists of a small number of shell commands.

/// Parse the string as a shell command.
///
/// Usually, a shell command is whitespace-separated array of strings.
///
/// ```text
/// cat file  -->  ["cat", "file"]
/// ```
///
/// But sometimes, you may want to include whitespaces in each argument.  In that case, you can use
/// quotes.
///
/// ```text
/// ls 'VirtualBox VMs'  -->  ["ls", 'VirtualBox VMs']
/// ls VirtualBox' 'VMs  -->  ["ls", 'VirtualBox VMs']
/// ```
///
/// For simplicity, you may assume that the string only contains alphanumeric characters, spaces
/// (" "), and single quotes ("'").
///
/// See `test_shell` for more examples.
pub fn parse_shell_command(command: &str) -> Vec<String> {
    let mut vec: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut single_quoted = false;

    for c in command.chars() {
        if c == '\'' {
            // Toggle the quote state; do not push the quote itself into the word
            single_quoted = !single_quoted;
        } else if single_quoted {
            // Inside quotes, keep everything (even spaces)
            word.push(c);
        } else if c == ' ' {
            // Outside quotes, a space means we hit a delimiter
            if !word.is_empty() {
                vec.push(word.clone());
                word.clear();
            }
        } else {
            // Keep alphanumeric or any other valid non-space characters
            word.push(c);
        }
    }

    // Push the final word if the string didn't end with a space
    if !word.is_empty() {
        vec.push(word);
    }

    vec
}
