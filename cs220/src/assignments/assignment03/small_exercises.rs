//! Small problems.

use std::collections::{HashMap, HashSet};
use std::fmt;
use regex::Regex;

/// Day of week.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayOfWeek {
    /// Sunday.
    Sun,
    /// Monday.
    Mon,
    /// Tuesday.
    Tue,
    /// Wednesday.
    Wed,
    /// Thursday.
    Thu,
    /// Friday.
    Fri,
    /// Saturday.
    Sat,
}

/// The next day of week.
///
/// `next_weekday(Thu)` is `Fri`; and `next_weekday(Fri)` is `Mon`.
pub fn next_weekday(day: DayOfWeek) -> DayOfWeek {
    match day {
        DayOfWeek::Thu => DayOfWeek::Fri,
        DayOfWeek::Fri | DayOfWeek::Sat | DayOfWeek::Sun => DayOfWeek::Mon,
        DayOfWeek::Mon => DayOfWeek::Tue,
        DayOfWeek::Tue => DayOfWeek::Wed,
        DayOfWeek::Wed => DayOfWeek::Thu,
    }
}

/// Given a list of integers, returns its median (when sorted, the value in the middle position).
///
/// For a data set `x` of `n` elements, the median can be defined as follows:
///
/// - If `n` is odd, the median is `(n+1)/2`-th smallest element of `x`.
/// - If `n` is even, the median is `(n/2)+1`-th smallest element of `x`.
///
/// For example, the following list of seven numbers,
///
/// ```ignore
/// vec![1, 3, 3, 6, 7, 8, 9]
/// ```
///
/// has the median of 6, which is the fourth value. And for this data set of eight numbers,
///
/// ```ignore
/// vec![1, 2, 3, 4, 5, 6, 8, 9]
/// ```
///
/// it has the median of 5, which is the fifth value.
///
/// Returns `None` if the list is empty.
pub fn median(values: Vec<isize>) -> Option<isize> {
    if values.is_empty() {
        return None
    }

    let mut v = values.clone();

    v.sort();

    if v.len().is_multiple_of(2) {
        return Some(v[v.len()/2]);
    }

    Some(v[v.len().div_ceil(2)-1])
}

/// Given a list of integers, returns its smallest mode (the value that occurs most often; a hash
/// map will be helpful here).
///
/// Returns `None` if the list is empty.
pub fn mode(values: Vec<isize>) -> Option<isize> {
    if values.is_empty() {
        return None;
    }

    let mut counts = HashMap::new();
    for &v in &values {
        *counts.entry(v).or_insert(0) += 1;
    }

    let mut smallest_mode = values[0];
    let mut max_count = *counts.get(&values[0]).unwrap();

    for (&num, &count) in &counts {
        if count > max_count {
            max_count = count;
            smallest_mode = num;
        } else if count == max_count && num < smallest_mode {
            smallest_mode = num;
        }
    }

    Some(smallest_mode)
}

/// Converts the given string to Pig Latin. Use the rules below to translate normal English into Pig
/// Latin.
///
/// 1. If a word starts with a consonant and a vowel, move the first letter of the word at the end
///    of the word and add "ay".
///
/// Example: "happy" -> "appyh" + "ay" -> "appyhay"
///
/// 2. If a word starts with multiple consonants, move them to the end of the word and add "ay".
///
/// Example: "string" -> "ingstr" + "ay" -> "ingstray"
///
/// 3. If a word starts with a vowel, add the word "hay" at the end of the word.
///
/// Example: "explain" -> "explain" + "hay" -> "explainhay"
///
/// Keep in mind the details about UTF-8 encoding!
///
/// You may assume the string only contains lowercase alphabets, and it contains at least one vowel.
pub fn piglatin(input: String) -> String {
    let vowel: HashSet<char> = ['a', 'e', 'i', 'o', 'u'].into_iter().collect();

    if let Some(first_chr) = input.chars().next() {
        // Case 3: a word starts with a vowel
        if vowel.contains(&first_chr) {
            let mut output = input.clone();
            output.push_str("hay");
            return output;
        } else {
            // Case 1/2. a word starts with multiple consonants
            let first_vowel_pos = input
                .char_indices()
                .find(|(_, c)| vowel.contains(c))
                .map(|(index, _)| index);
            if let Some(index) = first_vowel_pos {
                let (before, after) = input.split_at(index);
                let mut rearranged_str = format!("{}{}", after, before);
                rearranged_str.push_str("ay");
                return rearranged_str;
            }
        }
    }

    input
}

/// Converts HR commands to the organization table.
///
/// If the commands are as follows:
///
/// ```ignore
/// vec!["Add Amir to Engineering", "Add Sally to Sales", "Remove Jeehoon from Sales", "Move Amir from Engineering to Sales"]
/// ```
///
/// The return value should be:
///
/// ```ignore
/// ["Sales" -> ["Amir", "Sally"]]
/// ```
///
/// - The result is a map from department to the list of its employees.
/// - An empty department should not appear in the result.
/// - There are three commands: "Add {person} to {department}", "Remove {person} from {department}",
///   and "Move {person} from {department} to {department}".
/// - If a command is not executable, then it's ignored.
/// - There is no space in the name of the person and department.
///
/// See the test function for more details.
pub fn organize(commands: Vec<String>) -> HashMap<String, HashSet<String>> {
    let re_add = Regex::new(r"^Add (?<person>[\w-]+) to (?<dept>[\w-]+)$").unwrap();
    let re_rm = Regex::new(r"^Remove (?<person>[\w-]+) from (?<dept>[\w-]+)$").unwrap();
    let re_move = Regex::new(r"^Move (?<person>[\w-]+) from (?<from_dept>[\w-]+) to (?<to_dept>[\w-]+)$").unwrap();

    let mut map: HashMap<String, HashSet<String>> = HashMap::new();

    for command in &commands {
        if let Some(caps) = re_add.captures(command) {
            let person = caps["person"].to_string();
            let dept = caps["dept"].to_string();
            
            assert!(map.entry(dept).or_default().insert(person));
        } else if let Some(caps) = re_rm.captures(command) {
            let person = &caps["person"];
            let dept = &caps["dept"];

            let is_executable = map.get(dept)
                .is_some_and(|persons| persons.contains(person));

            if is_executable
                && let Some(persons) = map.get_mut(dept) {
                    assert!(persons.remove(person));
                    if persons.is_empty() {
                        assert_ne!(map.remove(dept), None);
                    }
                }
        } else if let Some(caps) = re_move.captures(command) {
            let person = &caps["person"];
            let from_dept = &caps["from_dept"];
            let to_dept = &caps["to_dept"];

            let is_executable = map.get(from_dept)
                .is_some_and(|persons| persons.contains(person));

            if is_executable {
                if let Some(persons) = map.get_mut(from_dept) {
                    assert!(persons.remove(person));
                    if persons.is_empty() {
                        assert_ne!(map.remove(from_dept), None);
                    }
                }
                assert!(map.entry(to_dept.to_string()).or_default().insert(person.to_string()));
            }
        }
    }

    map
}

/// Events in a text editor.
#[derive(Debug)]
pub enum TypeEvent {
    /// A character is typed.
    Type(char),
    /// The last character is removed.
    Backspace,
    /// The whole string is copied to the clipboard.
    Copy,
    /// The string in the clipboard is appended.
    Paste,
}

/// Starting from an empty string and an empty clipboard,
/// processes the given `events` in order and returns the resulting string.
///
/// See the test function `test_editor` for examples.
pub fn use_editor(events: Vec<TypeEvent>) -> String {
    let mut buf = String::new();
    let mut cb = String::new();

    for event in &events {
        match event {
            TypeEvent::Type(c) => {
                buf.push(*c);
            },
            TypeEvent::Backspace => {
                _ = buf.pop();
            },
            TypeEvent::Copy => {
                cb = buf.clone();
            },
            TypeEvent::Paste => {
                buf.push_str(&cb);
            },
        }
    }

    buf
}
