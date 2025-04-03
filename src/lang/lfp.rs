use super::expr::Expr;

pub struct LFP {
    /// The name of the function
    pub name: String,
    /// The arguments of the function``
    pub args: Vec<String>,
    /// The body of the function
    pub body: Expr,
}
