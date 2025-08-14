//! Syntax trait definition for user-defined datatypes
//!
//! The Syntax trait provides the interface for integrating custom datatypes
//! with the LS-Parser system. All user-defined datatypes must derive this trait.

use std::fmt;

/// Core trait that all user-defined datatypes must implement
///
/// This trait provides the necessary interface for integrating custom
/// datatypes with the LS-Parser system. It includes methods for:
/// - Converting to/from SMT-LIB representation
/// - Type checking and validation
/// - Pretty printing for debugging
pub trait Syntax: fmt::Debug + fmt::Display + Clone {
    /// Get the name of this datatype
    fn name(&self) -> &str;

    /// Convert this value to SMT-LIB format
    fn to_smtlib(&self) -> String;

    /// Parse from SMT-LIB format
    fn from_smtlib(s: &str) -> Result<Self, SyntaxError>
    where
        Self: Sized;

    /// Get the sort/type of this value
    fn sort(&self) -> &str;

    /// Validate that this value is well-formed
    fn validate(&self) -> Result<(), SyntaxError>;

    /// Get the number of constructors (for sum types)
    fn constructor_count(&self) -> usize;

    /// Get the name of the constructor at the given index
    fn constructor_name(&self, index: usize) -> Option<&str>;
}

/// Finite trait for types that can be represented as finite values
/// 
/// This trait is used to ensure that types used in evaluation
/// can be properly handled by the system.
/// All type parameters in a Finite type must also be Finite or Bool.
pub trait Finite: fmt::Debug + fmt::Display + Clone {
    /// Get the cardinality of this finite type
    fn cardinality(&self) -> usize;
    
    /// Check if this value is valid for the finite type
    fn is_valid(&self) -> bool;
    
    /// Get a default value for this finite type
    fn default() -> Self
    where
        Self: Sized;
}

/// Error type for syntax operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum SyntaxError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}
