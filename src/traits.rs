//! traits.rs
//!
//! Defines the Finder trait, used by syncwalk and asyncwalk
//! to find the files which match supplied stat metadata
//use std::path::Path;
use crate::errors::AmbleError;

/// Finder trait provies the find_matching method, which should be used
/// to find matching
pub trait Finder {
    type ReturnType;

    fn find_matching(&self
        // start_dir: &Path,
        // days: f32,
        // access: bool,
        // create: bool,
        // modify: bool,
        // skip: &Vec<String>, // list of directory names we want to skip
        // ignore_hidden: bool,
        // threads: Option<u8>,
    ) -> Result<Self::ReturnType, AmbleError>;
}