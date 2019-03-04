use std::path::Path;
use crate::errors::AmbleError;

pub trait Finder {
    fn find_matching(
        start_dir: &Path,
        days: u8,
        access: bool,
        create: bool,
        modify: bool,
        skip: &Vec<String>, // list of directory names we want to skip
        ignore_hidden: bool,

    ) -> Result<(), AmbleError>;
}