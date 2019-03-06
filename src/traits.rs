//! traits.rs
//!
//! Defines the Finder trait, used by syncwalk and asyncwalk
//! to find the files which match supplied stat metadata
//use std::path::Path;
use crate::errors::AmbleError;

/// Finder trait provies the `find_matching` method, which should be used
/// to find files matching supplied criteria (presumably on the struct or
/// enum implementing Finder)
pub trait Finder {
    type ReturnType;

    fn find_matching( &self ) -> Result<Self::ReturnType, AmbleError>;
}