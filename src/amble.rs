use dir_ageism::find_matching;


use std::path::PathBuf;
use structopt::StructOpt;

/// Traverse a directory recursively, reporting on entities which
/// whose access, modification, and/or creation time falls within a
/// certain timeframe.
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

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    /// The time period in days in which to consider entities, based
    /// on the metadata criteria. (ie access time, create time, modify time)
    #[structopt(short = "d", long = "days")]
    days: u8,

    /// Files to process
    #[structopt(name = "DIR", parse(from_os_str))]
    dir: PathBuf,
}


fn main() {
    let opt = Opt::from_args();
    if !opt.dir.exists() {
        println!("Warning: '{}' does not exist. Exiting.",
                opt.dir
                    .into_os_string()
                    .into_string()
                    .unwrap());
        return;
    }

    //println!("{:?}", opt);
    find_matching(&opt.dir, opt.days, opt.access, opt.create, opt.modify);
}