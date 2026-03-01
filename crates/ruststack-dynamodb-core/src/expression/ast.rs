//! AST types for DynamoDB expression parsing.
//!
//! This module defines the abstract syntax tree for DynamoDB condition, filter,
//! key-condition, update, and projection expressions. The AST is produced by the
//! parser and consumed by the evaluator.

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
    /// The `size(path)` function used as an operand in comparisons.
    Size(AttributePath),
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
    /// Addition: `operand + operand`.
    Plus(Operand, Operand),
    /// Subtraction: `operand - operand`.
    Minus(Operand, Operand),
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
