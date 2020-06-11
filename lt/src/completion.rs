//! Performs completion by searching for filenames matching the word prefix.
//! Modified from the linefeed original for PathCompleter.
extern crate linefeed;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::fs::read_dir;
use std::path::{is_separator, MAIN_SEPARATOR};

use linefeed::complete::{Completer, Completion, Suffix};
use linefeed::prompter::Prompter;
use linefeed::terminal::Terminal;

/// Implelements a `linefeed::complete::Completer`, for completing file paths in interactive mode.
pub struct PathCompleter;

impl<Term: Terminal> Completer<Term> for PathCompleter {
    fn complete(
        &self,
        word: &str,
        reader: &Prompter<Term>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        Some(complete_path(word, reader, start, end))
    }

    fn word_start(&self, line: &str, end: usize, _reader: &Prompter<Term>) -> usize {
        escaped_word_start(&line[..end])
    }

    fn quote<'a>(&self, word: &'a str) -> Cow<'a, str> {
        // escape(word)
        Borrowed(word)
    }

    fn unquote<'a>(&self, word: &'a str) -> Cow<'a, str> {
        unescape(word)
    }
}

/// Returns a sorted list of paths whose prefix matches the given path.
pub fn complete_path<Term: Terminal>(
    path: &str,
    _reader: &Prompter<Term>,
    _start: usize,
    _end: usize,
) -> Vec<Completion> {
    // println!(
    //     "\ncomplete_path\n\tpath: {:?}\n\tbuffer: {:?}\n\tstart: {:?}\n\tend: {:?}",
    //     path,
    //     reader.buffer(),
    //     start,
    //     end
    // );
    let (base_dir, fname) = split_path(path);
    // println!("\tSplitpat -  base: {:?}, fname: {:?}", base_dir, fname);
    let mut res = Vec::new();

    let lookup_dir = base_dir.unwrap_or(".");

    if let Ok(list) = read_dir(lookup_dir) {
        for ent in list {
            if let Ok(ent) = ent {
                // Need to escape spaces in these strings to support
                // file names with spaces.
                let ent_name = ent.file_name();
                // println!("\tChecking file name: {:?}", ent_name);

                // TODO: Deal with non-UTF8 paths in some way
                if let Ok(path) = ent_name.into_string() {
                    // Let's enable matching with some of what's in the name.
                    if (path.to_lowercase()).contains(&fname.to_lowercase()) {
                        // Fill out the directory if there is a maatch for the full directory
                        // Note that this is different from the display we'd like to show.
                        let (name, mut display) = if let Some(dir) = base_dir {
                            (format!("{}{}{}", dir, MAIN_SEPARATOR, path), Some(path))
                        } else {
                            (path, None)
                        };

                        // Add a separator to the end of directories.
                        let is_dir = ent.metadata().ok().map_or(false, |m| m.is_dir());
                        let suffix = if is_dir {
                            Suffix::Some(MAIN_SEPARATOR)
                        } else {
                            Suffix::Default
                        };

                        if display.is_none() {
                            display = Some(name.clone());
                        }

                        // Escape spaces into the name.
                        // If we don't do this we end up getting back
                        // a string with unescaped spaces in it which
                        // we can't really identify as part of a cohesive
                        // path. So put the escapes in there to make the path
                        // an string unseparated by strings.
                        let name = escape(&name).to_string();

                        res.push(Completion {
                            completion: name,
                            #[allow(clippy::redundant_field_names)]
                            display: display,
                            suffix: suffix,
                        });
                    }
                }
            }
        }
    }

    res.sort_by(|a, b| a.completion.cmp(&b.completion));
    // println!("complete path returning: {:?}", res);
    res
}

/// Returns the start position of the word that ends at the end of the string.
pub fn word_break_start(s: &str, word_break: &str) -> usize {
    // println!(
    //     "\nword_break_start s: {:?}, word_break: {:?}",
    //     s, word_break
    // );
    let mut start = s.len();

    for (idx, ch) in s.char_indices().rev() {
        if word_break.contains(ch) {
            break;
        }
        start = idx;
    }

    start
}

/// Returns the start position of a word with non-word characters escaped by
/// backslash (`\\`).
pub fn escaped_word_start(s: &str) -> usize {
    // println!("\nescaped_word_start s: {:?}", s);

    let mut chars = s.char_indices().rev();
    let mut start = s.len();

    while let Some((idx, ch)) = chars.next() {
        // println!("Checking idx: {}, char: {:?}", idx, ch);
        if needs_escape(ch) {
            // println!("Needs escape: idx: {}, char: {:?}", idx, ch);
            let n = {
                let mut n = 0;

                loop {
                    let mut clone = chars.clone();

                    let ch = match clone.next() {
                        Some((_, ch)) => ch,
                        None => break,
                    };

                    if ch == '\\' {
                        chars = clone;
                        n += 1;
                    } else {
                        break;
                    }
                }

                n
            };

            if n % 2 == 0 {
                break;
            }
        }

        start = idx;
    }

    // println!("escaped_word_start retunring: {}", start);
    start
}

/// Escapes a word by prefixing a backslash (`\\`) to non-word characters.
/// This checks for existing escapes and doesn't repeat them.
/// ie. it does not turn "hello\ world" into "hello\\ world".
pub fn escape(s: &str) -> Cow<str> {
    // println!("escape {:?}", s);
    let n = s.chars().filter(|&ch| needs_escape(ch)).count();
    // println!("Needs {} escape chars.", n);

    if n == 0 {
        Borrowed(s)
    } else {
        let mut res = String::with_capacity(s.len() + n);

        let mut mem = ' '; // hack, this is  not '\'.
        for ch in s.chars() {
            // don't continue to escape escaped chracters.
            if needs_escape(ch) && (mem != '\\') {
                res.push('\\');
            }
            res.push(ch);
            mem = ch;
        }

        Owned(res)
    }
}

/// Unescapes a word by removing the backslash (`\\`) from escaped characters.
pub fn unescape(s: &str) -> Cow<str> {
    // println!("unescape: {:?}", s);
    if s.contains('\\') {
        let mut res = String::with_capacity(s.len());
        let mut chars = s.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(ch) = chars.next() {
                    res.push(ch);
                }
            } else {
                res.push(ch);
            }
        }

        Owned(res)
    } else {
        Borrowed(s)
    }
}

fn needs_escape(ch: char) -> bool {
    match ch {
        ' ' | '\t' | '\n' | '\\' => true,
        _ => false,
    }
}

fn split_path(path: &str) -> (Option<&str>, &str) {
    // println!("Splitting string: {:?}", path);
    match path.rfind(is_separator) {
        Some(pos) => (Some(&path[..pos]), &path[pos + 1..]),
        None => (None, path),
    }
}
