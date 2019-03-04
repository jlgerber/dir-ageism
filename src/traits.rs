use std::path::Path;
use crate::errors::AmbleError;

pub trait Finder {
    fn find_matching(
        start_dir: &Path,
        days: f32,
        access: bool,
        create: bool,
        modify: bool,
        skip: &Vec<String>, // list of directory names we want to skip
        ignore_hidden: bool,
        threads: Option<u8>,
    ) -> Result<(), AmbleError>;
}