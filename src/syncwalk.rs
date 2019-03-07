//! syncwalk.rs
//!
//! Single threaded traversal of directory usiing the walkdir crate.
//! This is a bit slower than asyncwalk, but returns results in order.
use std::path::PathBuf;
use walkdir::{WalkDir, DirEntry};
use crate::{ errors::AmbleError, constants::SECS_PER_DAY };
use super::traits::Finder;


/// Implements the Finder trait to perform syncronous searching of
/// directory tree for files whose access, create, and/or modify
/// metadata values are less than or equal to the supplied age in
/// days, or fraction thereof.
///
/// SyncSearch implements a builder pattern to make it more convenient
/// to set the various options, but comes with reasonable defaults.
///
/// The only struct field which needs to be initialized is the start_dir,
/// which is set in the `new` function. All of the other fields have
/// corresponding builder functions which take a parameter of the
/// matchinh type and return a mutable reference to Self.
///
/// # Example
///
/// ```rust
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<Error>> {
/// use std::path::PathBuf;
/// use dir_ageism::{syncwalk::SyncSearch, traits::Finder};
///
/// let matching = SyncSearch::new("./")
///     .days(1.0)
///     .access(true)
///     .ignore_hidden(true)
///     .find_matching();
///
/// #
/// #     Ok(())
/// # }
/// ```
pub struct SyncSearch {
    /// The root directory to search
    start_dir: PathBuf,
    /// The number of days back to search
    days: f32,
    /// Whether or not to check access time
    access: bool,
    /// Whether or not to check create time (not available on Linux)
    create: bool,
    /// Whether or not to check modification time
    modify: bool,
    /// Whether or not to ignore hidden files (files starting with a '.')
    ignore_hidden: bool,
    /// A list of zero or more names to skip. These may either be directory names,
    /// in which case we skip any children, or file names, in which case
    /// we skip checking them.
    skip: Vec<String>,
}

impl SyncSearch {

    /// New up a SyncSearch instance, supplying a start_dir.
    ///
    /// We default to:
    /// - days: 8
    /// - access: true
    /// - create: true
    /// - modify: true
    /// - ignore_hidden: true
    /// - skip: []
    ///
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
    pub fn start_dir(&mut self, start_dir: impl Into<PathBuf>) -> &mut Self {
        self.start_dir = start_dir.into();
        self
    }
    /// Set the number of days to search for.
    pub fn days(&mut self, days: f32) -> &mut Self {
        self.days = days;
        self
    }

    /// Set whether or not we are interested in access time.
    pub fn access(&mut self, access: bool) -> &mut Self {
        self.access = access;
        self
    }

    /// Set whether or not we are interested in creation time.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Set whether or not we are interested in modification time.
    pub fn modify(&mut self, modify: bool) -> &mut Self {
        self.modify = modify;
        self
    }

    /// Set whether or not we should ignore hidden directories by default. Hidden
    /// directories start with a '.'.
    pub fn ignore_hidden(&mut self, ignore_hidden: bool) -> &mut Self {
        self.ignore_hidden = ignore_hidden;
        self
    }

    /// Set the skip list.
    pub fn skip(&mut self, skip: Vec<String>) -> &mut Self {
        self.skip = skip;
        self
    }

    // Was the entry modified within the last `self.days` # of days?
    fn report_modified(entry: &walkdir::DirEntry, days: f32) -> Result<bool, AmbleError> {
        let modified = entry.metadata()?.modified()?;
        Ok(modified.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * f64::from(days)).ceil() as u64))
    }

    // Was the entry accessed iwthint the last `self.days` # of days?
    fn report_accessed(entry: &walkdir::DirEntry, days: f32) -> Result<bool, AmbleError> {
        let accessed = entry.metadata()?.accessed()?;
        Ok(accessed.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * f64::from(days)).ceil() as u64))
    }

    // Was the entry created in the last `self.days` number of days?
    fn report_created(entry: &walkdir::DirEntry, days: f32) -> Result<bool, AmbleError> {
        let created = entry.metadata()?.created()?;
        Ok(created.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * f64::from(days)).ceil() as u64))
    }

    // is the DirEntry hidden? If check is false, we dont bother
    // actually checking; instead we automatically return false.
    fn is_hidden(entry: &DirEntry, check: bool) -> bool {
        if !check { return false; }
        entry.file_name()
            .to_str()
            .map(|s| s.starts_with('.') && s != "./")
            .unwrap_or(false)
    }

    // predicate to determine if a directory matches one or more
    // directory names
    fn matches_list(entry: &DirEntry, list: &[String] ) -> bool {
        if list.is_empty() {
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

        false
    }
}


impl Finder for SyncSearch {
    type ReturnType = ();

    fn find_matching(&self) -> Result<Self::ReturnType, AmbleError> {
        if !(self.access || self.create || self.modify) {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }

        let walker = WalkDir::new(&self.start_dir)
                .follow_links(true)
                .into_iter();

        for entry in walker
        .filter_entry(|e| {
                !(SyncSearch::is_hidden(e, self.ignore_hidden) ||
                  SyncSearch::matches_list(e, &self.skip))
            }
        ) {
            // filter out errors (like for permissions)
            let entry = match entry {
                Ok(e) => {
                    // need to test to make sure that symlinks
                    // get followed before this test
                    if !e.file_type().is_file() {continue;}
                    e
                },
                Err(_) => continue,
            };
            // doing this roughly in code above.
            //if !entry.file_type().is_file() { continue; };
            let mut meta = "".to_string();
            if self.access && SyncSearch::report_accessed(&entry, self.days )? {
                    meta.push('a');

            }

            if self.create {
                #[cfg(target_os = "macos")] {
                if SyncSearch::report_created(&entry, self.days)? {
                    meta.push('c');
                };
                }
            }

            if self.modify && SyncSearch::report_modified(&entry, self.days)? {
                    meta.push('m');

            }

            if !meta.is_empty() {
                let f_name = entry.path().to_string_lossy();
                println!("{} ({})", f_name, meta);
            }
        }

        Ok(())
    }
}
