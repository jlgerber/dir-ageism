//! syncwalk.rs
//!
//! Single threaded traversal of directory usiing the walkdir crate.
//! This is a bit slower than asyncwalk, but returns results in order.
use std::path::Path;
use walkdir::{WalkDir, DirEntry};
use crate::{ errors::AmbleError, constants::SECS_PER_DAY };

// predicate to determine if a directory matches one or more
// directory names
fn matches_list(entry: &DirEntry, list: &Vec<String> ) -> bool {
    if list.len() == 0 {
        return false;
    }

    for item in list {
        if entry.file_name()
            .to_str()
            .map(|s| s == item)
            .unwrap_or(false) {
                return true;
            }
    }
    return false;
}

pub struct SyncSearch {}

use super::traits::Finder;

fn is_hidden(entry: &DirEntry, check: bool) -> bool {
    if !check { return false; }

    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}
impl Finder for SyncSearch {
    fn find_matching(
        start_dir: &Path,
        days: f32,
        access: bool,
        create: bool,
        modify: bool,
        skip: &Vec<String>, // list of directory names we want to skip
        ignore_hidden: bool,// list of directory names we want to skip
        _threads: Option<u8>,
    ) -> Result<(), AmbleError> {
        if (access || create || modify) == false {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }

    let walker = WalkDir::new(start_dir)
            .follow_links(true)
            .into_iter();

    for entry in walker
    .filter_entry(|e| !matches_list(e, skip) || is_hidden(e, ignore_hidden)) {
        // filter out errors (like for permissions)
        let entry = match entry {
            Ok(e) => {
                // need to test to make sure that symlinks
                // get followed before this test
                if !e.file_type().is_file() { continue;}
                e
            },
            Err(_) => continue,
        };
        // doing this roughly in code above.
        //if !entry.file_type().is_file() { continue; };

        let mut meta = "".to_string();
        if access {
            if report_accessed(&entry, days )? {
                meta.push('a');
            }
        }

        if create {
            #[cfg(target_os = "macos")] {
            if report_created(&entry, days)? {
                meta.push('c');
            };
            }
        }

        if modify {
           if report_modified(&entry, days)? {
               meta.push('m');
           }
        }

        if meta.len() > 0 {
            let f_name = entry.path().to_string_lossy();
            println!("{} ({})", f_name, meta);
        }
    }

    Ok(())
}
}

// was the entry modified within the last `days` # of days
fn report_modified(entry: &walkdir::DirEntry, days: f32) -> Result<bool, AmbleError> {
    let modified = entry.metadata()?.modified()?;
    Ok(modified.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * days as f64).ceil() as u64))
}

// was the entry accessed iwthint the last `days` # of days
fn report_accessed(entry: &walkdir::DirEntry, days: f32) -> Result<bool, AmbleError> {
    let accessed = entry.metadata()?.accessed()?;
    Ok(accessed.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * days as f64).ceil() as u64))
}

// was the entry created in the last `days` number of days
fn report_created(entry: &walkdir::DirEntry, days: f32) -> Result<bool, AmbleError> {
    let created = entry.metadata()?.created()?;
    Ok(created.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * days as f64).ceil() as u64))
}