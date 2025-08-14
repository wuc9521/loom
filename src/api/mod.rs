//! API module for user-defined datatypes
//! 
//! This module provides a framework for users to define custom datatypes
//! that automatically derive the Syntax trait for integration with the
//! LS-Parser system.

pub mod syntax;

pub use syntax::{Syntax, SyntaxError, Finite};


// Re-export the derive macros
pub use ls_parser_derive::{Syntax, Finite};

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use super::syntax::Syntax;
}
