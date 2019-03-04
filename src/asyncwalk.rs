use ignore::{WalkBuilder,DirEntry, WalkState};
use crossbeam_channel as channel;
use std::thread;

use crate::traits::Finder;
use std::path::Path;
use crate::errors::AmbleError;

const SECS_PER_DAY: u64 = 86400;

pub struct AsyncSearch {}

impl Finder for AsyncSearch {
    fn find_matching(
        start_dir: &Path,
        days: u8,
        access: bool,
        create: bool,
        modify: bool,
        skip: &Vec<String>,
        ignore_hidden: bool,// list of directory names we want to skip
    ) -> Result<(), AmbleError> {
        if (access || create || modify) == false {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }
        let (tx, rx) = channel::bounded::<String>(100);

        let stdout_thread = thread::spawn(move || {
            for dent in rx {
                println!("{}", dent)
            }
        });

        let walker = WalkBuilder::new(start_dir)
            .hidden(ignore_hidden)
            .threads(0)
            .build_parallel();

        walker.run(|| {
            let tx = tx.clone();
            let myskip = skip.clone();
            Box::new(move |result| {
                match process_entry(result, days, access, create, modify, &myskip, ignore_hidden ) {
                    Ok((state,Some(meta))) => {
                        tx.send(meta).unwrap();
                        state
                    },
                    Err(e) => {
                        tx.send(e.to_string()).unwrap();
                        WalkState::Continue
                    },
                    Ok((state, None))=>{
                        state
                    }
                }
            })
        });

        drop(tx);
        stdout_thread.join().unwrap();

        Ok(())
    }
}

// process a single entry to determine whether or not it matches criteria.
// If it matches, we return an Ok wrapping a tuple of WalkState, Some(path).
// If we want to skip an entry, we return Ok wrapping a tuple of WalkState, None.
// If there is an error, we return an Err wrrapping AmbleError.
fn process_entry(result: std::result::Result<ignore::DirEntry, ignore::Error>,
   days:u8, access: bool, create: bool, modify: bool, skip: &Vec<String>, ignore_hidden: bool,
)
-> Result<(WalkState,Option< String>),AmbleError> {
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
            return Ok((WalkState::Continue,Some( format!("{} ({})", f_name, meta))));
        }
        return Ok((WalkState::Continue, None));
    };

    Ok((WalkState::Continue, None))
}


// was the entry modified within the last `days` # of days
fn report_modified(entry: &DirEntry, days: u64) -> Result<bool, AmbleError> {
    let modified = entry.metadata()?.modified()?;
    Ok(modified.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}

// was the entry accessed iwthint the last `days` # of days
fn report_accessed(entry: &DirEntry, days: u64) -> Result<bool, AmbleError> {
    let accessed = entry.metadata().unwrap().accessed()?;
    Ok(accessed.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}

// was the entry created in the last `days` number of days
fn report_created(entry: &DirEntry, days: u64) -> Result<bool, AmbleError> {
    let created = entry.metadata()?.created()?;
    Ok(created.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
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