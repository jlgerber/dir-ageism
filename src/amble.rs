use dir_ageism::{ traits::Finder, syncwalk::SyncSearch, asyncwalk::AsyncSearch, errors::AmbleError };

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
    /// of interest to Amble
    #[structopt(short = "c", long = "create")]
    create: bool,

    /// Ignore Hidden Files (that start with ".")
    #[structopt(short = "i", long = "ignore-hidden")]
    ignore: bool,

    /// The time period in days in which to consider entities, based
    /// on the metadata criteria
    #[structopt(short = "d", long = "days")]
    days: u8,

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

    // if the user doesn't specify the metadata of interest, then
    // it is all of interest.
    if !(opt.access || opt.create || opt.modify) {
        opt.access = true;
        opt.create = true;
        opt.modify = true;
    }

    if opt.sync {
        SyncSearch::find_matching(&opt.dir, opt.days, opt.access, opt.create, opt.modify, &opt.skip, opt.ignore, None)
    } else {
        AsyncSearch::find_matching(&opt.dir, opt.days, opt.access, opt.create, opt.modify, &opt.skip, opt.ignore, opt.threads)
    }
}