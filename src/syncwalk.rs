//! syncwalk.rs
//!
//! Single threaded traversal of directory usiing the walkdir crate.
//! This is a bit slower than asyncwalk, but returns results in order.
use std::path::PathBuf;
use walkdir::{WalkDir, DirEntry};
use crate::{ errors::AmbleError, constants::SECS_PER_DAY };
use super::traits::Finder;

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

pub struct SyncSearch {
    start_dir: PathBuf,
    days: f32,
    access: bool,
    create: bool,
    modify: bool,
    ignore_hidden: bool,
    skip: Vec<String>,
}

impl SyncSearch {
    pub fn new(start_dir: impl Into<PathBuf>) -> Self {
        Self {
            start_dir: start_dir.into(),
            days: 8.0,
            access: true,
            create: true,
            modify: true,
            ignore_hidden: true,
            skip: Vec::new(),
        }
    }

    /// reset the start directory for a search.
    pub fn start_dir<'a>(&'a mut self, start_dir: impl Into<PathBuf>) -> &'a mut Self {
        self.start_dir = start_dir.into();
        self
    }
    /// Set the number of days to search for.
    pub fn days<'a>(&'a mut self, days: f32) -> &'a mut Self {
        self.days = days;
        self
    }

    /// Set whether or not we are interested in access time.
    pub fn access<'a>(&'a mut self, access: bool) -> &'a mut Self {
        self.access = access;
        self
    }

    /// Set whether or not we are interested in creation time.
    pub fn create<'a>(&'a mut self, create: bool) -> &'a mut Self {
        self.create = create;
        self
    }

    /// Set whether or not we are interested in modification time.
    pub fn modify<'a>(&'a mut self, modify: bool) -> &'a mut Self {
        self.modify = modify;
        self
    }


    /// Set whether or not we should ignore hidden directories by default. Hidden
    /// directories start with a '.'.
    pub fn ignore_hidden<'a>(&'a mut self, ignore_hidden: bool) -> &'a mut Self {
        self.ignore_hidden = ignore_hidden;
        self
    }

    /// Set the skip list.
    pub fn skip<'a>(&'a mut self, skip: Vec<String>) -> &'a mut Self {
        self.skip = skip;
        self
    }

}

fn is_hidden(entry: &DirEntry, check: bool) -> bool {
    if !check { return false; }

    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}
impl Finder for SyncSearch {
    type ReturnType = ();
    fn find_matching(&self) -> Result<Self::ReturnType, AmbleError> {
        if (self.access || self.create || self.modify) == false {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }

    let walker = WalkDir::new(&self.start_dir)
            .follow_links(true)
            .into_iter();

    for entry in walker
    .filter_entry(|e| !matches_list(e, &self.skip) || is_hidden(e, self.ignore_hidden)) {
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
        if self.access {
            if report_accessed(&entry, self.days )? {
                meta.push('a');
            }
        }

        if self.create {
            #[cfg(target_os = "macos")] {
            if report_created(&entry, self.days)? {
                meta.push('c');
            };
            }
        }

        if self.modify {
           if report_modified(&entry, self.days)? {
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