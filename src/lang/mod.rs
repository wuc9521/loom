pub mod expr;
pub mod interpreter;
pub mod model;
pub mod pattern;
pub mod runtime;
pub mod theory;
pub mod types;
pub mod builtin;    
pub mod magic;

// Export specific items to avoid conflicts
pub use interpreter::TheoryInterpreter;
pub use magic::*;