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
use ignore::{WalkBuilder,DirEntry, WalkState};
use crossbeam_channel as channel;
use colored::*;
use std::thread;

use crate::traits::Finder;
use std::path::PathBuf;
use crate::{ errors::AmbleError, constants::SECS_PER_DAY };

/// Provides implementation of Finder.
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

impl Finder for AsyncSearch {
    type ReturnType = ();
    fn find_matching(&self
        // start_dir: &Path,
        // days: f32,
        // access: bool,
        // create: bool,
        // modify: bool,
        // skip: &Vec<String>,
        // ignore_hidden: bool,
        // threads: Option<u8>,
    ) -> Result<Self::ReturnType, AmbleError> {
        if (self.access || self.create || self.modify) == false {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }
        // for stdout
        let (tx, rx) = channel::bounded::<String>(100);

        // for errors
        let (tex, rex) = channel::bounded::<String>(100);

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
            Box::new(move |result| {
                match process_entry(result, self.days, self.access, self.create,
                                    self.modify, &myskip, self.ignore_hidden ) {
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

// process a single entry to determine whether or not it matches criteria.
// If it matches, we return an Ok wrapping a tuple of WalkState, Some(path).
// If we want to skip an entry, we return Ok wrapping a tuple of WalkState, None.
// If there is an error, we return an Err wrrapping AmbleError.
fn process_entry(result: std::result::Result<ignore::DirEntry, ignore::Error>,
   days: f32, access: bool, create: bool, modify: bool, skip: &Vec<String>, ignore_hidden: bool,
)
-> Result<(WalkState, Option<String>),AmbleError> {
    let entry = result?;
    let entry_type = entry.file_type().unwrap();

    // filter out directory if its name matches one of the provided
    // names in the skip list.
    if entry_type.is_dir() {
        if  skip.len() > 0 && matches_list(&entry, &skip) {
            return Ok((WalkState::Skip, None));
        }
    } else if entry_type.is_file() {
        let f_name = entry.path().to_string_lossy();

        // do i need this?
        if ignore_hidden {
            let f_name = entry.file_name().to_string_lossy();
            if f_name.starts_with(".") {
                return Ok((WalkState::Continue, None));
            }
        }
        // Test the various metadata statuses
        let mut meta = "".to_string();
        if access {
            if report_accessed(&entry, days)? {
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
            return Ok((WalkState::Continue, Some( format!("{} ({})", f_name, meta))));
        }
        return Ok((WalkState::Continue, None));
    };

    Ok((WalkState::Continue, None))
}


// was the entry modified within the last `days` # of days
fn report_modified(entry: &DirEntry, days: f32) -> Result<bool, AmbleError> {
    let modified = entry.metadata()?.modified()?;
    Ok(modified.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * days as f64).ceil() as u64))
}

// was the entry accessed iwthint the last `days` # of days
fn report_accessed(entry: &DirEntry, days: f32) -> Result<bool, AmbleError> {
    let accessed = entry.metadata().unwrap().accessed()?;
    Ok(accessed.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * days as f64).ceil() as u64))
}

// was the entry created in the last `days` number of days
fn report_created(entry: &DirEntry, days: f32) -> Result<bool, AmbleError> {
    let created = entry.metadata()?.created()?;
    Ok(created.elapsed()?.as_secs() < ((SECS_PER_DAY as f64 * days as f64).ceil() as u64))
}

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