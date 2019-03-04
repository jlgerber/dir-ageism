//! dir-ageism
//!
//!
use std::path::Path;
use walkdir::{WalkDir, DirEntry};

const SECS_PER_DAY: u64 = 86400;

use crate::errors::AmbleError;

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

pub fn find_matching(
        start_dir: &Path,
        days: u8,
        access: bool,
        create: bool,
        modify: bool,
        skip: &Vec<String>, // list of directory names we want to skip
    ) -> Result<(), AmbleError> {
        if (access || create || modify) == false {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }

    let walker = WalkDir::new(start_dir)
            .follow_links(true)
            .into_iter();

    for entry in walker
    .filter_entry(|e| !matches_list(e, skip)) {
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
            if report_accessed(&entry, days as u64)? {
                meta.push('a');
            }
        }

        if create {
            if report_created(&entry, days as u64)? {
                meta.push('c');
            };
        }

        if modify {
           if report_modified(&entry, days as u64)? {
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

// was the entry modified within the last `days` # of days
fn report_modified(entry: &walkdir::DirEntry, days: u64) -> Result<bool, AmbleError> {
    let modified = entry.metadata()?.modified()?;
    Ok(modified.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}

// was the entry accessed iwthint the last `days` # of days
fn report_accessed(entry: &walkdir::DirEntry, days: u64) -> Result<bool, AmbleError> {
    let accessed = entry.metadata()?.accessed()?;
    Ok(accessed.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}

// was the entry created in the last `days` number of days
fn report_created(entry: &walkdir::DirEntry, days: u64) -> Result<bool, AmbleError> {
    let created = entry.metadata()?.created()?;
    Ok(created.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}