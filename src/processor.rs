use {git2, rustc_serialize};
use git2::{Repository, Commit, Diff, DiffFormat, Oid, DiffDelta};
use std::collections::HashMap;
use std::fmt;
use std::str;

pub fn commits(repo: Repository, old_id: Oid, new_id: Oid) -> Result<OutputSet, git2::Error> {
    // Compare a specific commit pair.
    let old = repo.find_commit(old_id);
    let new = repo.find_commit(new_id);
    match (old, new) {
        (Ok(old_commit), Ok(new_commit)) => {
            let output = try!(find_moves(&repo, &old_commit, &new_commit));
            Ok(OutputSet {
                old: TransitOid(old_id),
                old_time: old_commit.time().seconds(), // Seconds from Epoch
                new: TransitOid(new_id),
                new_time: new_commit.time().seconds(), // Seconds from Epoch
                outputs: output,
            })
        },
        _ => Err(git2::Error::from_str("Commit IDs were not valid.")),
    }
}

pub fn repo(repo: Repository) -> Result<Vec<OutputSet>, git2::Error> {
    // Pull up the revwalk.
    // Revwalk.
    let mut revwalk = try!(repo.revwalk());
    // Setup some options.
    revwalk.simplify_first_parent(); // TODO: Maybe remove?
    let mut flags = git2::Sort::empty();
    flags.insert(git2::SORT_TIME);
    flags.insert(git2::SORT_TOPOLOGICAL);
    revwalk.set_sorting(flags);
    // Push HEAD to the revwalk.
    try!(revwalk.push_head());
    // We sadly must collect here to use `.windows()`
    let history = revwalk.collect::<Vec<Oid>>();
    let mut output = Vec::with_capacity(history.len());
    // Walk through each pair of commits.
    for pair in history.windows(2) {
        let (old_id, new_id) = (pair[0], pair[1]);
        if let (Ok(old), Ok(new)) = (repo.find_commit(old_id), repo.find_commit(new_id)) {
            let out = OutputSet {
                old: TransitOid(old_id),
                old_time: old.time().seconds(), // Seconds from Epoch
                new: TransitOid(new_id),
                new_time: new.time().seconds(), // Seconds from Epoch
                outputs: try!(find_moves(&repo, &old, &new)),
            };
            output.push(out);
        } else {
            continue;
        }
    }
    Ok(output)
}

#[derive(Debug, PartialEq, Eq)]
enum FoundState {
    Added, Deleted
}

fn format_key(key: String) -> String {
    let remove_whitespace = regex!(r"\s{2,}"); // 2 or more whitespaces
    let trim = regex!(r"^[\s]+|[\s]+$");
    let result = remove_whitespace.replace_all(&key[..], "");
    trim.replace_all(&result[..], "")
}

fn format_key_rust(key: String) -> String {
    let trimmed = format_key(key);
    let rust_vars = regex!(r"let\s+(?P<mut>mut\s+)?\s*(?P<vars>[a-zA-Z0-09_\(\),\s]+)");

    // TODO Remove this after debugging. It hides extra output for debugging.
    if !rust_vars.is_match(&trimmed[..]) {
        return trimmed;
    }

    println!("\n==========\ntrimmed={:?}", trimmed);

    let mut replacements: HashMap<String, String> = HashMap::new();

    let mut new_key = String::new();
    let mut count : u64 = 0;
    let mut index : usize = 0;

    for capture in rust_vars.captures_iter(&trimmed[..]) {

        //println!("pos0: {:?}, pos1: {:?}, pos2: {:?}", capture.pos(0), capture.pos(1), capture.pos(2));

        let whole_capture_start_pos = capture.pos(0).unwrap().0;
        let whole_capture_last_pos  = capture.pos(0).unwrap().1;

        if index < whole_capture_start_pos {
            let sliced = trimmed.slice_chars(index, whole_capture_start_pos);
            //println!("leading sliced={:?}", sliced);
            new_key = format!("{}{}", new_key, sliced);
            index = whole_capture_start_pos;
        }

        let var_key = format!("v{}", count);
        let var_value = format!("{}", capture.name("vars").unwrap());   // TODO Trim

        index = capture.pos(2).unwrap().1;

        // If contains ( or ), ignore for now.
        let rust_tuple = regex!(r"([a-zA-Z0-9_]+)(\s*,\s*)?");
        /* Tuple vars
        (a, b)
        (Ok(a), Ok(b))
        (

        */

        new_key = format!("{}let {}{} ", new_key, capture.name("mut").unwrap_or(""), var_key);

/*
        let var_capture_last_pos = capture.pos(2).unwrap().1;

        let sliced = trimmed.slice_chars(var_capture_last_pos, whole_capture_last_pos);
        println!("trailing sliced={:?}", sliced);
        new_key = format!("{}{}", new_key, sliced);
        index = whole_capture_last_pos;
*/
        replacements.insert(var_key, var_value);

        // TODO Do replacements.

        count += 1;
    }

    // Grab remainder of string.
    new_key = format!("{}{}", new_key, trimmed.slice_chars(index, trimmed.len()));

    println!("new_key={:?}", new_key);

    new_key
}

#[derive(Debug)]
struct Found {
    filename: String,
    key: String,
    state: FoundState,
    start_position: u32,
    line_count: u32,
}

#[derive(Debug)]
enum State {
    Other, Addition, Deletion
}

fn which_key_format_function(delta : DiffDelta) -> (fn(String) -> String) {
    let oldpath = delta.old_file().path().unwrap(); // TODO Do additions have old files?
    let newpath = delta.new_file().path().unwrap(); // TODO Do deletions have new files?

    if oldpath.extension() != newpath.extension() {
        println!("File extensions are different.");
        return format_key;
    }

    if let Some(ext) = oldpath.extension() {
        if ext == "rs" {
            return format_key_rust;
        }
    }

    return format_key;
}

fn find_additions_and_deletions(diff: Diff) -> Vec<Found> {

    let mut founds: Vec<Found> = Vec::new();

    let mut state = State::Other;
    let mut added = String::new();
    let mut deleted = String::new();
    let mut old_path = String::new();
    let mut new_path = String::new();

    let mut line_count: u32 = 0;
    let mut start_position: u32 = 0;

    // Read about this function in http://alexcrichton.com/git2-rs/git2/struct.Diff.html#method.print
    // It's a bit weird, but I think it will provide the necessary information.
    diff.print(DiffFormat::Patch, |delta, maybe_hunk, line| -> bool {

        // if delta.nfiles() != 2 {
        //     // This is diff with only one side, and thus can't have a move.
        //     return true;
        // }
        // assert!(delta.nfiles() == 2, "This only works on diffs between exactly 2 files. Found {} files.", delta.nfiles());

        // Thinking:
        //  * If is not a hunk, keep going.
        //  * If it's a hunk, do regex magic.
        //  * Stick regex output into a hashmap as a hash.
        //  * Later, we will iterate through and output pased on the values.
        // Filter out all the boring context lines.
        // If we're not interested in this line just return since it will iterate to the next.
        if maybe_hunk.is_none() { return true }; // Return early.

        //dump_diffline(&line);
        //dump_diffdelta(&delta);
        //dump_diffhunk(&maybe_hunk.unwrap());

        old_path = match delta.old_file().path()
        .and_then(|x| x.to_str()) {
            Some(path) => String::from_str(path),
            None => return false,
        };
        new_path = match delta.new_file().path()
        .and_then(|x| x.to_str()) {
            Some(path) => String::from_str(path),
            None => return false,
        };

        let format_key = which_key_format_function(delta);

        match line.origin() {
            // Additions
            '+' | '>' => {
                // If we attempt to unwrap and get `InvalidBytes(_)` it's probably just junk.
                // TODO: Is it?
                let line_str = str::from_utf8(line.content()).unwrap_or("");
                added.push_str(line_str);

                match state {
                    State::Deletion => {
                        founds.push(Found {
                            filename: old_path.clone(),
                            key: format_key(deleted.clone()),
                            state: FoundState::Deleted,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        deleted = String::new();
                        line_count = 0;
                        start_position = match line.new_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
                    },
                    State::Addition => {
                        line_count += 1;
                    },
                    State::Other => {
                        line_count = 1;
                        start_position = match line.new_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
                    },
                }

                state = State::Addition;
                true
            },
            // Deletions
            '-' | '<' => {
                // If we attempt to unwrap and get `InvalidBytes(_)` it's probably just junk.
                // TODO: Is it?
                let line_str = str::from_utf8(line.content()).unwrap_or("");
                deleted.push_str(line_str);

                match state {
                    State::Addition => {
                        founds.push(Found {
                            filename: new_path.clone(),
                            key: format_key(added.clone()),
                            state: FoundState::Added,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        added = String::new();
                        line_count = 0;
                        start_position = match line.old_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
                    },
                    State::Deletion => {
                        line_count += 1;
                    },
                    State::Other => {
                        line_count = 1;
                        start_position = match line.old_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
                    },
                }

                state = State::Deletion;
                true
            },
            // Other
            _         => {
                match state {
                    State::Addition => {
                        founds.push(Found {
                            filename: new_path.clone(),
                            key: format_key(added.clone()),
                            state: FoundState::Added,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        added = String::new();
                    },
                    State::Deletion => {
                        founds.push(Found {
                            filename: old_path.clone(),
                            key: format_key(deleted.clone()),
                            state: FoundState::Deleted,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        deleted = String::new();
                    },
                    _ => (),
                }

                state = State::Other;
                true
            }
        }
    }).ok(); // We don't care if we exit early. That's fine.

    // Grab last one.
    match state {
        State::Addition => {
            if added.len() > 0 {
                founds.push(Found {
                    filename: new_path.clone(),
                    key: format_key(added.clone()),
                    state: FoundState::Added,
                    start_position: start_position,
                    line_count: line_count,
                });
            }
        },
        State::Deletion => {
            if deleted.len() > 0 {
                founds.push(Found {
                    filename: old_path.clone(),
                    key: format_key(deleted.clone()),
                    state: FoundState::Deleted,
                    start_position: start_position,
                    line_count: line_count,
                });
            }
        },
        _ => (),
    }

    return founds;
}

fn find_moves(repo: &Repository, old: &Commit, new: &Commit) -> Result<Vec<Output>, git2::Error> {
    let old_tree = try!(old.tree());
    let new_tree = try!(new.tree());
    // Build up a diff of the two trees.
    let diff = try!(Diff::tree_to_tree(repo, Some(&old_tree), Some(&new_tree), None));

    let founds: Vec<Found> = find_additions_and_deletions(diff);

    let mut moves: Vec<Output> = Vec::new();
    let mut map: HashMap<String, Found> = HashMap::new();

    for f in founds {
        if f.key.len() == 0 { continue; }

        if map.contains_key(&f.key) {
            let q = map.get(&f.key).unwrap();

            // If both states are added, there are no moves.
            // assert!(f.state != q.state, format!("States {:?}, {:?}. Should be Addition/Deletion ", f.state, q.state));
            if f.state == q.state {
                return Ok(Vec::<Output>::new())
            }

            let output: Output;

            match f.state {
                FoundState::Added => {
                    output = Output {
                        old_filename: q.filename.clone(),
                        new_filename: f.filename.clone(),
                        origin_line: q.start_position,
                        destination_line: f.start_position,
                        num_lines: f.line_count,
                    };
                },
                FoundState::Deleted => {
                    output = Output {
                        old_filename: f.filename.clone(),
                        new_filename: q.filename.clone(),
                        origin_line: f.start_position,
                        destination_line: q.start_position,
                        num_lines: f.line_count,
                    };
                },
            }

            moves.push(output);
        } else {
            map.insert(f.key.clone(), f);
        }
    }

    return Ok(moves);
}

#[derive(Debug, RustcEncodable)]
pub struct OutputSet {
    old: TransitOid,
    old_time: i64, // Seconds from Epoch
    new: TransitOid,
    new_time: i64, // Seconds from Epoch
    outputs: Vec<Output>,
}

#[derive(Debug, RustcEncodable)]
pub struct Output {
    old_filename: String,
    new_filename: String,
    origin_line: u32,
    destination_line: u32,
    num_lines: u32,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TransitOid(Oid);
impl fmt::Display for TransitOid {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.0, f)
    }
}

impl rustc_serialize::Encodable for TransitOid {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&format!("{}", self)[..])
    }
}
