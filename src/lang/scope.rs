use crate::lang::runtime::{Scope, Value};
use crate::lang::interpreter::TheoryInterpreter;
use log::debug;

#[derive(Debug)]
pub struct ScopeManager {
    pub stack: Vec<Scope>,
}

impl ScopeManager {
    pub fn new() -> Self {
        let mut manager = Self { stack: Vec::new() };
        manager.stack.push(Scope::new()); // initialize global scope
        manager
    }

    /// get global scope as mutable reference
    pub fn global_scope_mut(&mut self) -> &mut Scope {
        self.stack.first_mut().expect("Stack should never be empty")
    }

    /// get global scope as immutable reference
    pub fn global_scope(&self) -> &Scope {
        self.stack.first().expect("Stack should never be empty")
    }

    /// get current scope as immutable reference
    pub fn current_scope(&self) -> &Scope {
        self.stack.last().expect("Stack should never be empty")
    }

    /// get current scope as mutable reference
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.stack.last_mut().expect("Stack should never be empty")
    }

    /// lookup a symbol in the scope stack
    pub fn lookup_symbol(&self, name: &str) -> Option<Value> {
        // lookup in the scope stack (from top to bottom, i.e. from recent to distant)
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.lookup(name) {
                debug!("find symbol: {} = {}", name, value);
                return Some(value);
            }
        }
        None
    }

    /// define a symbol in the current scope
    pub fn define_symbol(&mut self, name: String, value: Value) {
        self.current_scope_mut().define(name, value);
    }

    /// enter a new scope
    pub fn enter_scope(&mut self) {
        let new_scope = Scope::new();
        self.stack.push(new_scope);
    }

    /// exit the current scope
    pub fn exit_scope(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
            debug!("exit scope, current stack depth: {}", self.stack.len());
        } else {
            debug!("try to exit root scope");
        }
    }

    /// lookup a symbol in the scope stack with theory support
    pub fn lookup_symbol_with_theories(
        &self,
        name: &str,
        theories: &std::collections::HashMap<String, TheoryInterpreter>,
    ) -> Option<Value> {
        // first try to find in current scopes
        if let Some(value) = self.lookup_symbol(name) {
            return Some(value);
        }

        // if not found in current scopes, try to find in theories
        for theory in theories.values() {
            if let Some(value) = theory.interpreter.lookup_symbol(name) {
                return Some(value);
            }
        }

        None
    }
}
