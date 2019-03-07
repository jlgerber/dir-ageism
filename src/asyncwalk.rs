//! asyncwalk.rs
//!
//! Implementation of asyncronous traversal of directory.
//! This should be faster than the sync version, with the caveat
//! that entries will not be returned in order, as we are using
//! multiple threads to traverse in parallel.
//!
//! asyncwalk uses the ignore crate for the parallel directory traversal
//! iterator, and the crossbeam_channel crate for communication between
//! threads.
//!
//! All results are printed to stdout.
//!
//! All errors are printed to stderr.

// replacement channel that is more efficient
use crossbeam_channel as channel;
// embed color codes in strings
use colored::*;
// ignore crate written for ripgrep
use ignore::{WalkBuilder,DirEntry, WalkState};
use std::path::PathBuf;
use std::thread;

// internal imports
use crate::{ constants::SECS_PER_DAY, errors::AmbleError, traits::Finder };

/// Provides implementation of Finder trait via AsyncSearch struct.
/// AsyncSearch implements a builder pattern to make it more convenient
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
/// use amble::{asyncwalk::AsyncSearch, traits::Finder};
///
/// let matching = AsyncSearch::new("./")
///     .days(1.0)
///     .access(true)
///     .ignore_hidden(true)
///     .threads(Some(6))
///     .find_matching();
///
/// #
/// #     Ok(())
/// # }
/// ```
pub struct AsyncSearch {
    start_dir: PathBuf,
    days: f32,
    access: bool,
    create: bool,
    modify: bool,
    ignore_hidden: bool,
    skip: Vec<String>,
    threads: Option<u8>
}

impl AsyncSearch {
    /// New up an AsyncSearch instance, passing a PathBuf
    pub fn new(start_dir: impl Into<PathBuf>) -> Self {
        Self {
            start_dir: start_dir.into(),
            days: 8.0,
            access: true,
            create: true,
            modify: true,
            ignore_hidden: true,
            skip: Vec::new(),
            threads: None,
        }
    }

    /// Reset the start directory for a search.
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

    /// Set the number of threads
    pub fn threads(&mut self, threads: Option<u8>) -> &mut Self {
        self.threads = threads;
        self
    }

    // Process a single entry to determine whether or not it matches criteria.
    // If it matches, we return an Ok wrapping a tuple of WalkState, Some(path).
    // If we want to skip an entry, we return Ok wrapping a tuple of WalkState, None.
    // If there is an error, we return an Err wrrapping AmbleError.
    fn process_entry(result: std::result::Result<ignore::DirEntry, ignore::Error>,
                     days: f32, access: bool, create: bool, modify: bool,
                     skip: &[String])
    -> Result<(WalkState, Option<String>),AmbleError> {
        let entry = result?;
        let entry_type = entry.file_type().unwrap();

        // Filter out directory if its name matches one of the provided
        // names in the skip list.
        if entry_type.is_dir() {
            if  !skip.is_empty() && AsyncSearch::matches_list(&entry, &skip) {
                return Ok((WalkState::Skip, None));
            }
        } else if entry_type.is_file() {
            let f_name = entry.path().to_string_lossy();

            // Test the various metadata statuses
            let mut meta = "".to_string();
            if access && AsyncSearch::report_accessed(&entry, days)? {
                meta.push('a');

            }

            if create {
                #[cfg(target_os = "macos")] {
                if AsyncSearch::report_created(&entry, days)? {
                    meta.push('c');
                };
                }
            }

            if modify && AsyncSearch::report_modified(&entry, days)? {
                meta.push('m');

            }

            if !meta.is_empty() {
                return Ok((WalkState::Continue, Some( format!("{} ({})", f_name, meta))));
            }
            return Ok((WalkState::Continue, None));
        };

        Ok((WalkState::Continue, None))
    }

    // was the entry modified within the last `days` # of days
    fn report_modified(entry: &DirEntry, days: f32) -> Result<bool, AmbleError> {
        let modified = entry.metadata()?.modified()?;
        Ok(modified.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * f64::from(days)).ceil() as u64))
    }

    // was the entry accessed iwthint the last `days` # of days
    fn report_accessed(entry: &DirEntry, days: f32) -> Result<bool, AmbleError> {
        let accessed = entry.metadata().unwrap().accessed()?;
        Ok(accessed.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * f64::from(days)).ceil() as u64))
    }

    // was the entry created in the last `days` number of days
    fn report_created(entry: &DirEntry, days: f32) -> Result<bool, AmbleError> {
        let created = entry.metadata()?.created()?;
        Ok(created.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * f64::from(days)).ceil() as u64))
    }

    fn matches_list(entry: &DirEntry, list: &[String] ) -> bool {
        if !list.is_empty() {
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

impl Finder for AsyncSearch {
    type ReturnType = ();
    fn find_matching(&self
    ) -> Result<Self::ReturnType, AmbleError> {
        if !(self.access || self.create || self.modify) {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }
        // for stdout
        //let (tx, rx) = channel::bounded::<String>(100);
        let (tx, rx) = channel::unbounded::<String>();

        // for errors
        //let (tex, rex) = channel::bounded::<String>(100);
        let (tex, rex) = channel::unbounded::<String>();

        let stdout_thread = thread::spawn(move || {
            for dent in rx {
                println!("{}", dent)
            }
        });

        // If we want to capture the errors and print them out after
        // the thread has finished its thing, we could do this
        // let stderr_thread = thread::spawn(move || -> Vec<String> {
        //     let mut stderr_result = Vec:: new();
        //     for dent in rex {
        //         stderr_result.push(dent);
        //     }
        //     stderr_result
        // });

        let stderr_thread = thread::spawn(move || {
            for dent in rex {
                eprintln!("{}", dent.red());
            }
        });

        let walker = match self.threads {
            Some(th) => WalkBuilder::new(&self.start_dir)
                                    .hidden(self.ignore_hidden)
                                    .threads(th as usize)
                                    .follow_links(true)
                                    .build_parallel(),

            None => WalkBuilder::new(&self.start_dir)
                                .hidden(self.ignore_hidden)
                                .follow_links(true)
                                .build_parallel(),
        };

        walker.run(|| {
            let tx = tx.clone();
            let tex = tex.clone();
            let myskip = self.skip.clone();
            let days = self.days;
            let access = self.access;
            let create = self.create;
            let modify = self.modify;

            Box::new(move |result| {
                match AsyncSearch::process_entry(result, days, access, create,
                                                 modify, &myskip ) {
                    Ok((state,Some(meta))) => {
                        tx.send(meta).unwrap();
                        state
                    },
                    Err(e) => {
                        tex.send(e.to_string()).unwrap();
                        WalkState::Continue
                    },
                    Ok((state, None))=>{
                        state
                    }
                }
            })
        });

        drop(tx);
        drop(tex);
        stdout_thread.join().unwrap();
        let _err_vals = stderr_thread.join().unwrap();

        // if we wanted to print out errors after the fact, we could do this
        // if err_vals.len() > 0  {
        //     println!("{}","\nERRORS\n".red());
        //     for err in err_vals {
        //         eprintln!("{}", err.red());
        //     }
        // }

        Ok(())
    }


}
