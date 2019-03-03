//! dir-ageism
//!
//!
use std::path::Path;
use walkdir::{WalkDir};
pub mod errors;

const SECS_PER_DAY: u64 = 86400;

use crate::errors::AmbleError;

pub fn find_matching(
        start_dir: &Path,
        days: u8,
        access: bool,
        create: bool,
        modify: bool,
    ) -> Result<(), AmbleError> {
        if (access || create || modify) == false {
            println!("No search criteria specified. Must use access, create, or modify");
            return Ok(());
        }

    //println!("{:?}", start_dir);
    for entry in WalkDir::new(start_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {

        if !entry.file_type().is_file() { continue; };

        //let mut report = false;
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

// was the entry modified within the last days
fn report_modified(entry: &walkdir::DirEntry, days: u64) -> Result<bool, AmbleError> {
    let modified = entry.metadata()?.modified()?;
    Ok(modified.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}

fn report_accessed(entry: &walkdir::DirEntry, days: u64) -> Result<bool, AmbleError> {
    let accessed = entry.metadata()?.accessed()?;
    Ok(accessed.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}

fn report_created(entry: &walkdir::DirEntry, days: u64) -> Result<bool, AmbleError> {
    let created = entry.metadata()?.created()?;
    Ok(created.elapsed()?.as_secs() < (SECS_PER_DAY * days as u64))
}