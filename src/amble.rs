use dir_ageism::{ traits::Finder, syncwalk::SyncSearch, asyncwalk::AsyncSearch, errors::AmbleError };

use std::path::PathBuf;
use structopt::StructOpt;

/// Traverse a directory recursively, reporting on files
/// whose access, modification, and/or creation time falls within a
/// certain timeframe.
///
/// If the user does not specify the metadata
/// properties of interest, amble will use access, modify, and create
/// times.
#[derive(StructOpt, Debug)]
#[structopt(name = "amble")]
struct Opt {
    /// Use access time to determine whether a candidate is
    /// of interest to us.
    #[structopt(short = "a", long = "access")]
    access: bool,

    /// Use modification time to determine whether a candidate is
    /// of interest to us.
    #[structopt(short = "m", long = "modification")]
    modify: bool,

    /// Use creation time to determine whether a candidate is
    /// of interest to us.
    #[structopt(short = "c", long = "creation")]
    create: bool,


    /// Ignore Hidden Directories
    #[structopt(short = "i", long = "ignore-hidden")]
    ignore: bool,

    /// The time period in days in which to consider entities, based
    /// on the metadata criteria.
    #[structopt(short = "d", long = "days")]
    days: u8,

    /// Optional list of directory names to skip
    #[structopt(short = "s", long = "skip")]
    skip: Vec<String>,

    /// Files to process
    #[structopt(name = "DIR", parse(from_os_str))]
    dir: PathBuf,

    /// Fallback to using sync processing of directories.
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

    // if the user doesn't specify the metadata of interest, then
    // it is all of interest.
    if !(opt.access || opt.create || opt.modify) {
        opt.access = true;
        opt.create = true;
        opt.modify = true;
    }

    if opt.sync {
        SyncSearch::find_matching(&opt.dir, opt.days, opt.access, opt.create, opt.modify, &opt.skip, opt.ignore)
    } else {
        AsyncSearch::find_matching(&opt.dir, opt.days, opt.access, opt.create, opt.modify, &opt.skip, opt.ignore)
    }
}