//! amble.rs
//!
//! Implements the `amble` command, which is used to find files
//! within a supplied directory which match supplied metadata criteria.
//!
//! Specifically, we are looking for files whose create, modify, and/or
//! update dates fall within a certain number of days, supplied by the
//! user.
use dir_ageism::{
    asyncwalk::AsyncSearch,
    constants::MIN_DAYS,
    errors::AmbleError,
    syncwalk::SyncSearch,
    traits::Finder,
};

use std::path::PathBuf;
use structopt::StructOpt;

/// Traverse a directory recursively, reporting on files
/// whose access, modification, and/or creation time falls within a
/// certain timeframe.
///
/// If the user does not specify the metadata
/// properties of interest, amble will use access, modify, and create
/// times
#[derive(StructOpt, Debug)]
#[structopt(name = "amble")]
struct Opt {
    /// Use access time to determine whether a candidate is
    /// of interest to Amble
    #[structopt(short = "a", long = "access")]
    access: bool,

    /// Use modification time to determine whether a candidate is
    /// of interest to Amble
    #[structopt(short = "m", long = "modify")]
    modify: bool,

    /// Use creation time to determine whether a candidate is
    /// of interest to Amble. (NOT AVAILABLE ON LINUX)
    #[structopt(short = "c", long = "create")]
    create: bool,

    /// Ignore Hidden Files (that start with ".")
    #[structopt(short = "i", long = "ignore-hidden")]
    ignore: bool,

    /// The time period in days in which to consider entities, based
    /// on the metadata criteria
    #[structopt(short = "d", long = "days")]
    days: f32,

    /// Optional list of directory names to skip
    #[structopt(short = "s", long = "skip")]
    skip: Vec<String>,

    /// Optionally specify how many threads to spawn when using async
    #[structopt(short = "t", long = "threads")]
    threads: Option<u8>,

    /// Root directory to process. Amble will recursively descend through
    /// the supplied directory, identifying files which meet the provided
    /// criteria, and report them to stdout, along with an indication
    /// of the matching criteria
    #[structopt(name = "DIR", parse(from_os_str))]
    dir: PathBuf,

    /// Use single threaded directory traversal. The default behavior is
    /// to process directories using as many threads as cores.
    /// However, there is also a syncronous mode, which may be turned
    /// on for reference
    #[structopt(long = "sync")]
    sync: bool,
}

fn main() -> Result<(), AmbleError>{
    let mut opt = Opt::from_args();
    if !opt.dir.exists() {
        println!("Warning: '{}' does not exist. Exiting.",
                opt.dir
                    .into_os_string()
                    .into_string()
                    .unwrap());
        return Ok(());
    }

    if !(opt.days > MIN_DAYS) {
        println!("Warning: days must be greater than 0: {}.", opt.days);
        return Ok(());
    }

    // If the user doesn't specify the metadata of interest, then
    // it is all of interest.
    if !(opt.access || opt.create || opt.modify) {
        opt.access = true;
        #[cfg(target_os = "macos")]
        {
            opt.create = true;
        }
        #[cfg(target_os = "linux")]
        {
            opt.create = false;
        }
        opt.modify = true;
    }

    if opt.sync {
        SyncSearch::new(&opt.dir).days(opt.days)
                                 .access(opt.access)
                                 .create(opt.create)
                                 .modify(opt.modify)
                                 .skip(opt.skip)
                                 .ignore_hidden(opt.ignore)
                                 .find_matching()
    } else {
        AsyncSearch::new(&opt.dir).days(opt.days)
                                  .access(opt.access)
                                  .create(opt.create)
                                  .modify(opt.modify)
                                  .skip(opt.skip)
                                  .ignore_hidden(opt.ignore)
                                  .threads(opt.threads)
                                  .find_matching()
    }
}