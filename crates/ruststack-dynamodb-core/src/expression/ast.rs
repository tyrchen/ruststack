//! AST types for DynamoDB expression parsing.
//!
//! This module defines the abstract syntax tree for DynamoDB condition, filter,
//! key-condition, update, and projection expressions. The AST is produced by the
//! parser and consumed by the evaluator.
//!
//! The module also provides utility functions to collect all expression attribute
//! name and value references from parsed ASTs, used for validating that all
//! provided names/values are actually used in expressions.

use std::collections::HashSet;
use std::fmt;

/// Expression AST node for condition, filter, and key-condition expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Comparison expression: `left op right`.
    Compare {
        /// Left-hand operand.
        left: Box<Operand>,
        /// Comparison operator.
        op: CompareOp,
        /// Right-hand operand.
        right: Box<Operand>,
    },
    /// Between expression: `value BETWEEN low AND high`.
    Between {
        /// Value to test.
        value: Box<Operand>,
        /// Lower bound (inclusive).
        low: Box<Operand>,
        /// Upper bound (inclusive).
        high: Box<Operand>,
    },
    /// In expression: `value IN (list...)`.
    In {
        /// Value to search for.
        value: Box<Operand>,
        /// List of candidate values.
        list: Vec<Operand>,
    },
    /// Logical combination: `left AND right` or `left OR right`.
    Logical {
        /// Logical operator.
        op: LogicalOp,
        /// Left-hand expression.
        left: Box<Expr>,
        /// Right-hand expression.
        right: Box<Expr>,
    },
    /// Logical negation: `NOT expr`.
    Not(Box<Expr>),
    /// Function call: `function_name(args...)`.
    Function {
        /// Function name.
        name: FunctionName,
        /// Function arguments.
        args: Vec<Operand>,
    },
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Equal (`=`).
    Eq,
    /// Not equal (`<>`).
    Ne,
    /// Less than (`<`).
    Lt,
    /// Less than or equal (`<=`).
    Le,
    /// Greater than (`>`).
    Gt,
    /// Greater than or equal (`>=`).
    Ge,
}

impl fmt::Display for CompareOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq => write!(f, "="),
            Self::Ne => write!(f, "<>"),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
        }
    }
}

/// Logical operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
        }
    }
}

/// Built-in DynamoDB expression function names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionName {
    /// `attribute_exists(path)` - true if the attribute exists.
    AttributeExists,
    /// `attribute_not_exists(path)` - true if the attribute does not exist.
    AttributeNotExists,
    /// `attribute_type(path, type)` - true if the attribute is of the given type.
    AttributeType,
    /// `begins_with(path, substr)` - true if the string begins with the prefix.
    BeginsWith,
    /// `contains(path, operand)` - true if string contains substring or set contains element.
    Contains,
    /// `size(path)` - returns the size of the attribute.
    Size,
}

impl fmt::Display for FunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttributeExists => write!(f, "attribute_exists"),
            Self::AttributeNotExists => write!(f, "attribute_not_exists"),
            Self::AttributeType => write!(f, "attribute_type"),
            Self::BeginsWith => write!(f, "begins_with"),
            Self::Contains => write!(f, "contains"),
            Self::Size => write!(f, "size"),
        }
    }
}

/// An operand in an expression (a value producer).
#[derive(Debug, Clone)]
pub enum Operand {
    /// A document path reference (e.g., `info.rating`, `#name`, `myList[0]`).
    Path(AttributePath),
    /// An expression attribute value reference (e.g., `:val`).
    Value(String),
    /// The `size(operand)` function used as an operand in comparisons.
    ///
    /// The inner operand can be a path, a value reference, or even a nested `size()`.
    Size(Box<Operand>),
}

/// A document path consisting of one or more elements.
#[derive(Debug, Clone)]
pub struct AttributePath {
    /// The path elements in order.
    pub elements: Vec<PathElement>,
}

impl fmt::Display for AttributePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, elem) in self.elements.iter().enumerate() {
            match elem {
                PathElement::Attribute(name) => {
                    if i > 0 {
                        write!(f, ".{name}")?;
                    } else {
                        write!(f, "{name}")?;
                    }
                }
                PathElement::Index(idx) => write!(f, "[{idx}]")?,
            }
        }
        Ok(())
    }
}

/// A single element in an attribute path.
#[derive(Debug, Clone)]
pub enum PathElement {
    /// A named attribute (or `#placeholder` reference).
    Attribute(String),
    /// A list index dereference (e.g., `[0]`).
    Index(usize),
}

/// Update expression AST containing all four clause types.
#[derive(Debug, Clone)]
pub struct UpdateExpr {
    /// SET actions: assign values to attributes.
    pub set_actions: Vec<SetAction>,
    /// REMOVE actions: remove attributes.
    pub remove_paths: Vec<AttributePath>,
    /// ADD actions: add to numbers or sets.
    pub add_actions: Vec<AddAction>,
    /// DELETE actions: remove elements from sets.
    pub delete_actions: Vec<DeleteAction>,
}

/// A single SET action: `path = value`.
#[derive(Debug, Clone)]
pub struct SetAction {
    /// Target attribute path.
    pub path: AttributePath,
    /// Value to assign.
    pub value: SetValue,
}

/// The right-hand side of a SET action.
#[derive(Debug, Clone)]
pub enum SetValue {
    /// Simple operand assignment.
    Operand(Operand),
    /// Addition: `left + right` (supports compound expressions like `if_not_exists(...) + :val`).
    Plus(Box<SetValue>, Box<SetValue>),
    /// Subtraction: `left - right` (supports compound expressions).
    Minus(Box<SetValue>, Box<SetValue>),
    /// `if_not_exists(path, operand)` - use default if path does not exist.
    IfNotExists(AttributePath, Operand),
    /// `list_append(operand, operand)` - concatenate two lists.
    ListAppend(Operand, Operand),
}

/// A single ADD action: `path value`.
#[derive(Debug, Clone)]
pub struct AddAction {
    /// Target attribute path.
    pub path: AttributePath,
    /// Value to add.
    pub value: Operand,
}

/// A single DELETE action: `path value`.
#[derive(Debug, Clone)]
pub struct DeleteAction {
    /// Target attribute path.
    pub path: AttributePath,
    /// Value (set) to remove.
    pub value: Operand,
}

// ---------------------------------------------------------------------------
// Collecting used expression attribute names and values from ASTs
// ---------------------------------------------------------------------------

/// Collect all expression attribute name references (`#name`) used in a
/// condition/filter expression AST. Returns names with the `#` prefix.
#[allow(clippy::implicit_hasher)]
pub fn collect_names_from_expr(expr: &Expr, names: &mut HashSet<String>) {
    match expr {
        Expr::Compare { left, right, .. } => {
            collect_names_from_operand(left, names);
            collect_names_from_operand(right, names);
        }
        Expr::Between { value, low, high } => {
            collect_names_from_operand(value, names);
            collect_names_from_operand(low, names);
            collect_names_from_operand(high, names);
        }
        Expr::In { value, list } => {
            collect_names_from_operand(value, names);
            for item in list {
                collect_names_from_operand(item, names);
            }
        }
        Expr::Logical { left, right, .. } => {
            collect_names_from_expr(left, names);
            collect_names_from_expr(right, names);
        }
        Expr::Not(inner) => collect_names_from_expr(inner, names),
        Expr::Function { args, .. } => {
            for arg in args {
                collect_names_from_operand(arg, names);
            }
        }
    }
}

/// Collect all expression attribute value references (`:name`) used in a
/// condition/filter expression AST. Returns names with the `:` prefix.
#[allow(clippy::implicit_hasher)]
pub fn collect_values_from_expr(expr: &Expr, values: &mut HashSet<String>) {
    match expr {
        Expr::Compare { left, right, .. } => {
            collect_values_from_operand(left, values);
            collect_values_from_operand(right, values);
        }
        Expr::Between { value, low, high } => {
            collect_values_from_operand(value, values);
            collect_values_from_operand(low, values);
            collect_values_from_operand(high, values);
        }
        Expr::In { value, list } => {
            collect_values_from_operand(value, values);
            for item in list {
                collect_values_from_operand(item, values);
            }
        }
        Expr::Logical { left, right, .. } => {
            collect_values_from_expr(left, values);
            collect_values_from_expr(right, values);
        }
        Expr::Not(inner) => collect_values_from_expr(inner, values),
        Expr::Function { args, .. } => {
            for arg in args {
                collect_values_from_operand(arg, values);
            }
        }
    }
}

/// Collect all expression attribute name references from an update expression.
#[allow(clippy::implicit_hasher)]
pub fn collect_names_from_update(update: &UpdateExpr, names: &mut HashSet<String>) {
    for action in &update.set_actions {
        collect_names_from_path(&action.path, names);
        collect_names_from_set_value(&action.value, names);
    }
    for path in &update.remove_paths {
        collect_names_from_path(path, names);
    }
    for action in &update.add_actions {
        collect_names_from_path(&action.path, names);
        collect_names_from_operand(&action.value, names);
    }
    for action in &update.delete_actions {
        collect_names_from_path(&action.path, names);
        collect_names_from_operand(&action.value, names);
    }
}

/// Collect all expression attribute value references from an update expression.
#[allow(clippy::implicit_hasher)]
pub fn collect_values_from_update(update: &UpdateExpr, values: &mut HashSet<String>) {
    for action in &update.set_actions {
        collect_values_from_set_value(&action.value, values);
    }
    for action in &update.add_actions {
        collect_values_from_operand(&action.value, values);
    }
    for action in &update.delete_actions {
        collect_values_from_operand(&action.value, values);
    }
}

/// Collect all expression attribute name references from projection paths.
#[allow(clippy::implicit_hasher)]
pub fn collect_names_from_projection(paths: &[AttributePath], names: &mut HashSet<String>) {
    for path in paths {
        collect_names_from_path(path, names);
    }
}

/// Collect all top-level attribute path names used in a condition/filter
/// expression AST. Returns the raw first element of each `Operand::Path`,
/// which may be a bare name like `myAttr` or a `#placeholder` reference.
/// This is used to validate that FilterExpression does not reference key
/// attributes.
#[allow(clippy::implicit_hasher)]
pub fn collect_paths_from_expr(expr: &Expr, paths: &mut HashSet<String>) {
    match expr {
        Expr::Compare { left, right, .. } => {
            collect_paths_from_operand(left, paths);
            collect_paths_from_operand(right, paths);
        }
        Expr::Between { value, low, high } => {
            collect_paths_from_operand(value, paths);
            collect_paths_from_operand(low, paths);
            collect_paths_from_operand(high, paths);
        }
        Expr::In { value, list } => {
            collect_paths_from_operand(value, paths);
            for item in list {
                collect_paths_from_operand(item, paths);
            }
        }
        Expr::Logical { left, right, .. } => {
            collect_paths_from_expr(left, paths);
            collect_paths_from_expr(right, paths);
        }
        Expr::Not(inner) => collect_paths_from_expr(inner, paths),
        Expr::Function { args, .. } => {
            for arg in args {
                collect_paths_from_operand(arg, paths);
            }
        }
    }
}

fn collect_paths_from_operand(operand: &Operand, paths: &mut HashSet<String>) {
    match operand {
        Operand::Path(path) => {
            if let Some(PathElement::Attribute(name)) = path.elements.first() {
                paths.insert(name.clone());
            }
        }
        Operand::Value(_) => {}
        Operand::Size(inner) => collect_paths_from_operand(inner, paths),
    }
}

fn collect_names_from_operand(operand: &Operand, names: &mut HashSet<String>) {
    match operand {
        Operand::Path(path) => collect_names_from_path(path, names),
        Operand::Value(_) => {}
        Operand::Size(inner) => collect_names_from_operand(inner, names),
    }
}

fn collect_values_from_operand(operand: &Operand, values: &mut HashSet<String>) {
    match operand {
        Operand::Value(name) => {
            values.insert(format!(":{name}"));
        }
        Operand::Path(_) => {}
        Operand::Size(inner) => collect_values_from_operand(inner, values),
    }
}

fn collect_names_from_path(path: &AttributePath, names: &mut HashSet<String>) {
    for element in &path.elements {
        if let PathElement::Attribute(name) = element {
            if name.starts_with('#') {
                names.insert(name.clone());
            }
        }
    }
}

fn collect_names_from_set_value(set_value: &SetValue, names: &mut HashSet<String>) {
    match set_value {
        SetValue::Operand(op) => collect_names_from_operand(op, names),
        SetValue::Plus(a, b) | SetValue::Minus(a, b) => {
            collect_names_from_set_value(a, names);
            collect_names_from_set_value(b, names);
        }
        SetValue::IfNotExists(path, op) => {
            collect_names_from_path(path, names);
            collect_names_from_operand(op, names);
        }
        SetValue::ListAppend(a, b) => {
            collect_names_from_operand(a, names);
            collect_names_from_operand(b, names);
        }
    }
}

fn collect_values_from_set_value(set_value: &SetValue, values: &mut HashSet<String>) {
    match set_value {
        SetValue::Operand(op) => collect_values_from_operand(op, values),
        SetValue::Plus(a, b) | SetValue::Minus(a, b) => {
            collect_values_from_set_value(a, values);
            collect_values_from_set_value(b, values);
        }
        SetValue::IfNotExists(_, op) => {
            collect_values_from_operand(op, values);
        }
        SetValue::ListAppend(a, b) => {
            collect_values_from_operand(a, values);
            collect_values_from_operand(b, values);
        }
    }
}
