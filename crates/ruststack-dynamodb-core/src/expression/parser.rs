//! Lexer and recursive-descent parser for DynamoDB expressions.
//!
//! Supports condition/filter/key-condition expressions, update expressions, and
//! projection expressions. Keywords and function names are matched
//! case-insensitively per DynamoDB specification.

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use super::ast::{
    AddAction, AttributePath, CompareOp, DeleteAction, Expr, FunctionName, LogicalOp, Operand,
    PathElement, SetAction, SetValue, UpdateExpr,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced during expression parsing or evaluation.
#[derive(Debug, thiserror::Error)]
pub enum ExpressionError {
    /// An unexpected token was encountered.
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        /// What was expected.
        expected: String,
        /// What was found.
        found: String,
    },
    /// The expression ended prematurely.
    #[error("Unexpected end of expression")]
    UnexpectedEof,
    /// An expression attribute name placeholder could not be resolved.
    #[error("Unresolved expression attribute name: {name}")]
    UnresolvedName {
        /// The unresolved name reference.
        name: String,
    },
    /// An expression attribute value placeholder could not be resolved.
    #[error("Unresolved expression attribute value: {name}")]
    UnresolvedValue {
        /// The unresolved value reference.
        name: String,
    },
    /// An operand is invalid for the given operation.
    #[error("Invalid operand for {operation}: {message}")]
    InvalidOperand {
        /// The operation that failed.
        operation: String,
        /// Explanation.
        message: String,
    },
    /// A type mismatch occurred during evaluation.
    #[error("Type mismatch: {message}")]
    TypeMismatch {
        /// Explanation.
        message: String,
    },
    /// A validation error in projection or other expression.
    #[error("{message}")]
    Validation {
        /// Explanation.
        message: String,
    },
}

// ---------------------------------------------------------------------------
// Token type
// ---------------------------------------------------------------------------

/// Lexer token for DynamoDB expressions.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// A plain identifier (attribute name).
    Identifier(String),
    /// An expression attribute name reference (`#name`).
    ExprAttrName(String),
    /// An expression attribute value reference (`:value`).
    ExprAttrValue(String),
    /// `=`
    Eq,
    /// `<>`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `.`
    Dot,
    /// `,`
    Comma,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    // Keywords
    /// `AND`
    And,
    /// `OR`
    Or,
    /// `NOT`
    Not,
    /// `BETWEEN`
    Between,
    /// `IN`
    In,
    /// `SET`
    Set,
    /// `REMOVE`
    Remove,
    /// `ADD`
    Add,
    /// `DELETE`
    Delete,
    // Function names
    /// `attribute_exists`
    AttributeExists,
    /// `attribute_not_exists`
    AttributeNotExists,
    /// `attribute_type`
    AttributeType,
    /// `begins_with`
    BeginsWith,
    /// `contains`
    Contains,
    /// `size`
    Size,
    /// `if_not_exists`
    IfNotExists,
    /// `list_append`
    ListAppend,
    /// A non-negative integer (used for list indices).
    Number(usize),
    /// End of input.
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(s) => write!(f, "identifier '{s}'"),
            Self::ExprAttrName(s) => write!(f, "#{s}"),
            Self::ExprAttrValue(s) => write!(f, ":{s}"),
            Self::Eq => write!(f, "'='"),
            Self::Ne => write!(f, "'<>'"),
            Self::Lt => write!(f, "'<'"),
            Self::Le => write!(f, "'<='"),
            Self::Gt => write!(f, "'>'"),
            Self::Ge => write!(f, "'>='"),
            Self::Plus => write!(f, "'+'"),
            Self::Minus => write!(f, "'-'"),
            Self::Dot => write!(f, "'.'"),
            Self::Comma => write!(f, "','"),
            Self::LParen => write!(f, "'('"),
            Self::RParen => write!(f, "')'"),
            Self::LBracket => write!(f, "'['"),
            Self::RBracket => write!(f, "']'"),
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
            Self::Not => write!(f, "NOT"),
            Self::Between => write!(f, "BETWEEN"),
            Self::In => write!(f, "IN"),
            Self::Set => write!(f, "SET"),
            Self::Remove => write!(f, "REMOVE"),
            Self::Add => write!(f, "ADD"),
            Self::Delete => write!(f, "DELETE"),
            Self::AttributeExists => write!(f, "attribute_exists"),
            Self::AttributeNotExists => write!(f, "attribute_not_exists"),
            Self::AttributeType => write!(f, "attribute_type"),
            Self::BeginsWith => write!(f, "begins_with"),
            Self::Contains => write!(f, "contains"),
            Self::Size => write!(f, "size"),
            Self::IfNotExists => write!(f, "if_not_exists"),
            Self::ListAppend => write!(f, "list_append"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Eof => write!(f, "EOF"),
        }
    }
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

/// Tokenizer for DynamoDB expression strings.
struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    /// Tokenize the entire input into a vector of tokens.
    fn tokenize(&mut self) -> Result<Vec<Token>, ExpressionError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            if tok == Token::Eof {
                tokens.push(Token::Eof);
                break;
            }
            tokens.push(tok);
        }
        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while self.chars.peek().is_some_and(char::is_ascii_whitespace) {
            self.chars.next();
        }
    }

    fn next_token(&mut self) -> Result<Token, ExpressionError> {
        self.skip_whitespace();

        let Some(&ch) = self.chars.peek() else {
            return Ok(Token::Eof);
        };

        match ch {
            '#' => self.read_expr_attr_name(),
            ':' => self.read_expr_attr_value(),
            '=' => {
                self.chars.next();
                Ok(Token::Eq)
            }
            '<' => Ok(self.read_lt_family()),
            '>' => Ok(self.read_gt_family()),
            '+' => {
                self.chars.next();
                Ok(Token::Plus)
            }
            '-' => {
                self.chars.next();
                Ok(Token::Minus)
            }
            '.' => {
                self.chars.next();
                Ok(Token::Dot)
            }
            ',' => {
                self.chars.next();
                Ok(Token::Comma)
            }
            '(' => {
                self.chars.next();
                Ok(Token::LParen)
            }
            ')' => {
                self.chars.next();
                Ok(Token::RParen)
            }
            '[' => {
                self.chars.next();
                Ok(Token::LBracket)
            }
            ']' => {
                self.chars.next();
                Ok(Token::RBracket)
            }
            c if c.is_ascii_digit() => self.read_number(),
            c if is_ident_start(c) => Ok(self.read_identifier_or_keyword()),
            _ => Err(ExpressionError::UnexpectedToken {
                expected: "valid token".to_owned(),
                found: format!("'{ch}'; syntax error near '{ch}'"),
            }),
        }
    }

    fn read_expr_attr_name(&mut self) -> Result<Token, ExpressionError> {
        self.chars.next(); // consume '#'
        // After '#', the reference name can start with any alphanumeric or underscore
        // character (unlike bare identifiers which must start with a letter).
        let name = self.read_ref_chars();
        if name.is_empty() {
            return Err(ExpressionError::UnexpectedToken {
                expected: "attribute name after '#'".to_owned(),
                found: "empty".to_owned(),
            });
        }
        Ok(Token::ExprAttrName(name))
    }

    fn read_expr_attr_value(&mut self) -> Result<Token, ExpressionError> {
        self.chars.next(); // consume ':'
        // After ':', the reference name can start with any alphanumeric or underscore
        // character (unlike bare identifiers which must start with a letter).
        let name = self.read_ref_chars();
        if name.is_empty() {
            return Err(ExpressionError::UnexpectedToken {
                expected: "value name after ':'".to_owned(),
                found: "empty".to_owned(),
            });
        }
        Ok(Token::ExprAttrValue(name))
    }

    fn read_lt_family(&mut self) -> Token {
        self.chars.next(); // consume '<'
        if self.chars.peek() == Some(&'=') {
            self.chars.next();
            Token::Le
        } else if self.chars.peek() == Some(&'>') {
            self.chars.next();
            Token::Ne
        } else {
            Token::Lt
        }
    }

    fn read_gt_family(&mut self) -> Token {
        self.chars.next(); // consume '>'
        if self.chars.peek() == Some(&'=') {
            self.chars.next();
            Token::Ge
        } else {
            Token::Gt
        }
    }

    fn read_number(&mut self) -> Result<Token, ExpressionError> {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        // DynamoDB rejects list indices that overflow a reasonable integer
        // range with "List index is not within the allowable range".
        let n: usize = s.parse().map_err(|_| ExpressionError::Validation {
            message: format!("List index is not within the allowable range; index: [{s}]"),
        })?;
        // Even if it fits in usize on 64-bit, DynamoDB limits indices to
        // roughly i32 range.
        if n > i32::MAX as usize {
            return Err(ExpressionError::Validation {
                message: format!("List index is not within the allowable range; index: [{s}]"),
            });
        }
        Ok(Token::Number(n))
    }

    fn read_ident_chars(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if is_ident_continue(c) {
                s.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        s
    }

    /// Read characters for a reference name (after `#` or `:`).
    ///
    /// Unlike bare identifiers, references allow any alphanumeric or underscore
    /// character in any position (including leading digits and underscores).
    fn read_ref_chars(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        s
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let ident = self.read_ident_chars();
        let lower = ident.to_ascii_lowercase();

        // Keywords (case-insensitive): SET, REMOVE, ADD, DELETE, AND, OR, NOT,
        // BETWEEN, IN.
        match lower.as_str() {
            "and" => return Token::And,
            "or" => return Token::Or,
            "not" => return Token::Not,
            "between" => return Token::Between,
            "in" => return Token::In,
            "set" => return Token::Set,
            "remove" => return Token::Remove,
            "add" => return Token::Add,
            "delete" => return Token::Delete,
            _ => {}
        }

        // Function names (case-sensitive per DynamoDB specification).
        match ident.as_str() {
            "attribute_exists" => Token::AttributeExists,
            "attribute_not_exists" => Token::AttributeNotExists,
            "attribute_type" => Token::AttributeType,
            "begins_with" => Token::BeginsWith,
            "contains" => Token::Contains,
            "size" => Token::Size,
            "if_not_exists" => Token::IfNotExists,
            "list_append" => Token::ListAppend,
            _ => Token::Identifier(ident),
        }
    }
}

/// Returns `true` if `c` can start an identifier.
///
/// DynamoDB identifiers must start with an alphabetic character. Underscores
/// and digits are NOT valid as the first character of an attribute name token.
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

/// Returns `true` if `c` can continue an identifier.
fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Recursive-descent parser for DynamoDB expressions.
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<Token, ExpressionError> {
        let tok = self.advance();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            Ok(tok)
        } else {
            Err(ExpressionError::UnexpectedToken {
                expected: expected.to_string(),
                found: tok.to_string(),
            })
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    /// Check if the current Identifier token is followed by `(`, indicating
    /// a function call.
    fn is_function_call_ahead(&self) -> bool {
        matches!(self.tokens.get(self.pos + 1), Some(Token::LParen))
    }
}

// ---------------------------------------------------------------------------
// Condition expression parsing (precedence climbing)
// ---------------------------------------------------------------------------

impl Parser {
    /// Parse a full condition expression (OR is lowest precedence).
    fn parse_or_expr(&mut self) -> Result<Expr, ExpressionError> {
        let mut left = self.parse_and_expr()?;
        while matches!(self.peek(), Token::Or) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::Logical {
                op: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// Parse AND expressions.
    fn parse_and_expr(&mut self) -> Result<Expr, ExpressionError> {
        let mut left = self.parse_not_expr()?;
        while matches!(self.peek(), Token::And) {
            self.advance();
            let right = self.parse_not_expr()?;
            left = Expr::Logical {
                op: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// Parse NOT expressions.
    fn parse_not_expr(&mut self) -> Result<Expr, ExpressionError> {
        if matches!(self.peek(), Token::Not) {
            self.advance();
            let expr = self.parse_not_expr()?;
            return Ok(Expr::Not(Box::new(expr)));
        }
        self.parse_primary_expr()
    }

    /// Parse primary expressions: comparisons, BETWEEN, IN, functions, and parenthesized groups.
    fn parse_primary_expr(&mut self) -> Result<Expr, ExpressionError> {
        // Parenthesized group
        if matches!(self.peek(), Token::LParen) {
            self.advance();
            let expr = self.parse_or_expr()?;
            self.expect(&Token::RParen)?;
            return Ok(expr);
        }

        // Reject update-only functions (if_not_exists, list_append) used in
        // condition expressions.
        if matches!(self.peek(), Token::IfNotExists | Token::ListAppend) {
            let name = if matches!(self.peek(), Token::IfNotExists) {
                "if_not_exists"
            } else {
                "list_append"
            };
            // Consume the function name and its arguments to give a clean error.
            self.advance();
            if matches!(self.peek(), Token::LParen) {
                self.advance();
                let mut depth = 1u32;
                while depth > 0 && !matches!(self.peek(), Token::Eof) {
                    match self.advance() {
                        Token::LParen => depth += 1,
                        Token::RParen => depth -= 1,
                        _ => {}
                    }
                }
            }
            // Check if followed by a comparison operator (used as operand in comparison).
            return Err(ExpressionError::InvalidOperand {
                operation: name.to_owned(),
                message: format!(
                    "The function is not allowed to be used this way in an expression; \
                     function: {name}"
                ),
            });
        }

        // Function call as expression (attribute_exists, attribute_not_exists, etc.)
        if let Some(func_name) = self.peek_function_name() {
            return self.parse_function_expr(func_name);
        }

        // Operand-initiated expressions (comparison, BETWEEN, IN)
        let operand = self.parse_operand()?;
        self.parse_postfix_expr(operand)
    }

    /// Check if current token is a condition function name, return the `FunctionName` if so.
    fn peek_function_name(&self) -> Option<FunctionName> {
        match self.peek() {
            Token::AttributeExists => Some(FunctionName::AttributeExists),
            Token::AttributeNotExists => Some(FunctionName::AttributeNotExists),
            Token::AttributeType => Some(FunctionName::AttributeType),
            Token::BeginsWith => Some(FunctionName::BeginsWith),
            Token::Contains => Some(FunctionName::Contains),
            _ => None,
        }
    }

    /// Parse a function expression like `attribute_exists(#name)`.
    fn parse_function_expr(&mut self, name: FunctionName) -> Result<Expr, ExpressionError> {
        self.advance(); // consume function name
        self.expect(&Token::LParen)?;
        let mut args = vec![self.parse_operand()?];
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            args.push(self.parse_operand()?);
        }
        self.expect(&Token::RParen)?;
        Ok(Expr::Function { name, args })
    }

    /// After parsing a left operand, parse comparison, BETWEEN, or IN.
    fn parse_postfix_expr(&mut self, left: Operand) -> Result<Expr, ExpressionError> {
        match self.peek() {
            Token::Eq | Token::Ne | Token::Lt | Token::Le | Token::Gt | Token::Ge => {
                let op = self.parse_compare_op()?;
                let right = self.parse_operand()?;
                Ok(Expr::Compare {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }
            Token::Between => {
                self.advance();
                let low = self.parse_operand()?;
                self.expect(&Token::And)?;
                let high = self.parse_operand()?;
                Ok(Expr::Between {
                    value: Box::new(left),
                    low: Box::new(low),
                    high: Box::new(high),
                })
            }
            Token::In => {
                self.advance();
                self.expect(&Token::LParen)?;
                // Empty IN list is a syntax error.
                if matches!(self.peek(), Token::RParen) {
                    return Err(ExpressionError::UnexpectedToken {
                        expected: "operand".to_owned(),
                        found: "Syntax error; IN list must have at least one element".to_owned(),
                    });
                }
                let mut list = vec![self.parse_operand()?];
                while matches!(self.peek(), Token::Comma) {
                    self.advance();
                    list.push(self.parse_operand()?);
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::In {
                    value: Box::new(left),
                    list,
                })
            }
            Token::LParen => {
                // The left operand looks like a function call (e.g., `dog(...)` or
                // `BEGINS_WITH(...)`), but the identifier was not recognized as a
                // valid function name.
                let func_name = match &left {
                    Operand::Path(path) => path.to_string(),
                    other => format!("{other:?}"),
                };
                Err(ExpressionError::UnexpectedToken {
                    expected: "valid function name".to_owned(),
                    found: format!(
                        "'{func_name}' is not a valid function; valid functions are: \
                        attribute_exists, attribute_not_exists, attribute_type, begins_with, \
                        contains, size"
                    ),
                })
            }
            _ => Err(ExpressionError::UnexpectedToken {
                expected: "comparison operator, BETWEEN, or IN after operand".to_owned(),
                found: format!(
                    "Syntax error; a standalone value or path is not a valid condition; \
                     found: {}",
                    self.peek()
                ),
            }),
        }
    }

    fn parse_compare_op(&mut self) -> Result<CompareOp, ExpressionError> {
        let tok = self.advance();
        match tok {
            Token::Eq => Ok(CompareOp::Eq),
            Token::Ne => Ok(CompareOp::Ne),
            Token::Lt => Ok(CompareOp::Lt),
            Token::Le => Ok(CompareOp::Le),
            Token::Gt => Ok(CompareOp::Gt),
            Token::Ge => Ok(CompareOp::Ge),
            _ => Err(ExpressionError::UnexpectedToken {
                expected: "comparison operator".to_owned(),
                found: tok.to_string(),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Operand & path parsing
// ---------------------------------------------------------------------------

impl Parser {
    /// Parse an operand: a value reference, a `size(path)` call, or an attribute path.
    fn parse_operand(&mut self) -> Result<Operand, ExpressionError> {
        match self.peek() {
            Token::ExprAttrValue(_) => {
                let Token::ExprAttrValue(name) = self.advance() else {
                    return Err(ExpressionError::UnexpectedEof);
                };
                Ok(Operand::Value(name))
            }
            Token::Size => {
                self.advance();
                self.expect(&Token::LParen)?;
                let inner = self.parse_operand()?;
                // Check for extra arguments: size() takes exactly 1 argument.
                if matches!(self.peek(), Token::Comma) {
                    // Count extra args.
                    let mut count = 1;
                    while matches!(self.peek(), Token::Comma) {
                        self.advance();
                        let _ = self.parse_operand()?;
                        count += 1;
                    }
                    let _ = self.expect(&Token::RParen);
                    return Err(ExpressionError::InvalidOperand {
                        operation: "size".to_owned(),
                        message: format!(
                            "Incorrect number of operands for operator or function; \
                             operator or function: size, number of operands: {count}"
                        ),
                    });
                }
                self.expect(&Token::RParen)?;
                Ok(Operand::Size(Box::new(inner)))
            }
            _ => {
                let path = self.parse_attribute_path()?;
                Ok(Operand::Path(path))
            }
        }
    }

    /// Parse an attribute path like `info.rating`, `#name`, `myList[0].value`.
    fn parse_attribute_path(&mut self) -> Result<AttributePath, ExpressionError> {
        let first = self.parse_path_head()?;
        let mut elements = vec![first];

        loop {
            match self.peek() {
                Token::Dot => {
                    self.advance();
                    let elem = self.parse_path_head()?;
                    elements.push(elem);
                }
                Token::LBracket => {
                    self.advance();
                    let tok = self.advance();
                    // Reject negative indices (minus token inside brackets).
                    if matches!(tok, Token::Minus) {
                        return Err(ExpressionError::UnexpectedToken {
                            expected: "non-negative index".to_owned(),
                            found: "Syntax error; negative index is not allowed".to_owned(),
                        });
                    }
                    let Token::Number(idx) = tok else {
                        return Err(ExpressionError::UnexpectedToken {
                            expected: "number".to_owned(),
                            found: "Syntax error; expected a non-negative integer index".to_owned(),
                        });
                    };
                    self.expect(&Token::RBracket)?;
                    elements.push(PathElement::Index(idx));
                }
                _ => break,
            }
        }

        Ok(AttributePath { elements })
    }

    /// Parse the first element of a path segment (identifier or `#name`).
    fn parse_path_head(&mut self) -> Result<PathElement, ExpressionError> {
        match self.peek() {
            Token::Identifier(_) => {
                let Token::Identifier(name) = self.advance() else {
                    return Err(ExpressionError::UnexpectedEof);
                };
                Ok(PathElement::Attribute(name))
            }
            Token::ExprAttrName(_) => {
                let Token::ExprAttrName(name) = self.advance() else {
                    return Err(ExpressionError::UnexpectedEof);
                };
                Ok(PathElement::Attribute(format!("#{name}")))
            }
            _ => Err(ExpressionError::UnexpectedToken {
                expected: "attribute name or #name".to_owned(),
                found: self.peek().to_string(),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Update expression parsing
// ---------------------------------------------------------------------------

impl Parser {
    /// Parse a complete update expression (SET, REMOVE, ADD, DELETE clauses).
    ///
    /// Each clause type (SET, REMOVE, ADD, DELETE) can only appear once.
    /// Within a clause, multiple actions are separated by commas.
    fn parse_update_expr(&mut self) -> Result<UpdateExpr, ExpressionError> {
        let mut update = UpdateExpr {
            set_actions: Vec::new(),
            remove_paths: Vec::new(),
            add_actions: Vec::new(),
            delete_actions: Vec::new(),
        };

        let mut seen_set = false;
        let mut seen_remove = false;
        let mut seen_add = false;
        let mut seen_delete = false;

        while !self.at_end() {
            match self.peek() {
                Token::Set => {
                    if seen_set {
                        return Err(ExpressionError::InvalidOperand {
                            operation: "UpdateExpression".to_owned(),
                            message:
                                "The \"SET\" section can only be used once in an update expression"
                                    .to_owned(),
                        });
                    }
                    seen_set = true;
                    self.advance();
                    self.parse_set_clause(&mut update.set_actions)?;
                }
                Token::Remove => {
                    if seen_remove {
                        return Err(ExpressionError::InvalidOperand {
                            operation: "UpdateExpression".to_owned(),
                            message: "The \"REMOVE\" section can only be used once in an update expression".to_owned(),
                        });
                    }
                    seen_remove = true;
                    self.advance();
                    self.parse_remove_clause(&mut update.remove_paths)?;
                }
                Token::Add => {
                    if seen_add {
                        return Err(ExpressionError::InvalidOperand {
                            operation: "UpdateExpression".to_owned(),
                            message:
                                "The \"ADD\" section can only be used once in an update expression"
                                    .to_owned(),
                        });
                    }
                    seen_add = true;
                    self.advance();
                    self.parse_add_clause(&mut update.add_actions)?;
                }
                Token::Delete => {
                    if seen_delete {
                        return Err(ExpressionError::InvalidOperand {
                            operation: "UpdateExpression".to_owned(),
                            message: "The \"DELETE\" section can only be used once in an update expression".to_owned(),
                        });
                    }
                    seen_delete = true;
                    self.advance();
                    self.parse_delete_clause(&mut update.delete_actions)?;
                }
                _ => {
                    return Err(ExpressionError::UnexpectedToken {
                        expected: "SET, REMOVE, ADD, or DELETE".to_owned(),
                        found: self.peek().to_string(),
                    });
                }
            }
        }

        Ok(update)
    }

    fn parse_set_clause(&mut self, actions: &mut Vec<SetAction>) -> Result<(), ExpressionError> {
        actions.push(self.parse_set_action()?);
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            actions.push(self.parse_set_action()?);
        }
        Ok(())
    }

    fn parse_set_action(&mut self) -> Result<SetAction, ExpressionError> {
        let path = self.parse_attribute_path()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_set_value()?;
        Ok(SetAction { path, value })
    }

    /// Parse the right-hand side of a SET action, which may be a simple operand,
    /// arithmetic (`a + b`, `a - b`), `if_not_exists(path, val)`, or `list_append(a, b)`.
    /// Supports compound expressions like `if_not_exists(path, :zero) + :one`.
    fn parse_set_value(&mut self) -> Result<SetValue, ExpressionError> {
        // Parse primary value (operand, if_not_exists, list_append, or unknown function)
        let primary = self.parse_set_value_primary()?;

        // Check for trailing + or -
        match self.peek() {
            Token::Plus => {
                self.advance();
                let right = self.parse_set_value_primary()?;
                Ok(SetValue::Plus(Box::new(primary), Box::new(right)))
            }
            Token::Minus => {
                self.advance();
                let right = self.parse_set_value_primary()?;
                Ok(SetValue::Minus(Box::new(primary), Box::new(right)))
            }
            _ => Ok(primary),
        }
    }

    /// Parse a primary SET value (without checking for trailing arithmetic operators).
    fn parse_set_value_primary(&mut self) -> Result<SetValue, ExpressionError> {
        match self.peek() {
            Token::IfNotExists => self.parse_if_not_exists(),
            Token::ListAppend => self.parse_list_append(),
            // Check if an identifier is followed by `(` - this is a function call.
            // DynamoDB accepts any identifier as a function name in the parser,
            // but rejects unknown function names later.
            Token::Identifier(_) if self.is_function_call_ahead() => {
                let Token::Identifier(name) = self.advance() else {
                    return Err(ExpressionError::UnexpectedEof);
                };
                // Skip over the function arguments.
                self.expect(&Token::LParen)?;
                self.parse_operand()?;
                while matches!(self.peek(), Token::Comma) {
                    self.advance();
                    self.parse_operand()?;
                }
                self.expect(&Token::RParen)?;
                Err(ExpressionError::InvalidOperand {
                    operation: "UpdateExpression".to_owned(),
                    message: format!("Invalid function name; function: {name}"),
                })
            }
            _ => Ok(SetValue::Operand(self.parse_operand()?)),
        }
    }

    fn parse_if_not_exists(&mut self) -> Result<SetValue, ExpressionError> {
        self.advance(); // consume `if_not_exists`
        self.expect(&Token::LParen)?;
        let path = self.parse_attribute_path()?;
        self.expect(&Token::Comma)?;
        let default = self.parse_operand()?;
        self.expect(&Token::RParen)?;
        Ok(SetValue::IfNotExists(path, default))
    }

    fn parse_list_append(&mut self) -> Result<SetValue, ExpressionError> {
        self.advance(); // consume `list_append`
        self.expect(&Token::LParen)?;
        let first = self.parse_operand()?;
        self.expect(&Token::Comma)?;
        let second = self.parse_operand()?;
        self.expect(&Token::RParen)?;
        Ok(SetValue::ListAppend(first, second))
    }

    fn parse_remove_clause(
        &mut self,
        paths: &mut Vec<AttributePath>,
    ) -> Result<(), ExpressionError> {
        paths.push(self.parse_attribute_path()?);
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            paths.push(self.parse_attribute_path()?);
        }
        Ok(())
    }

    fn parse_add_clause(&mut self, actions: &mut Vec<AddAction>) -> Result<(), ExpressionError> {
        actions.push(self.parse_add_action()?);
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            actions.push(self.parse_add_action()?);
        }
        Ok(())
    }

    fn parse_add_action(&mut self) -> Result<AddAction, ExpressionError> {
        let path = self.parse_attribute_path()?;
        let value = self.parse_operand()?;
        Ok(AddAction { path, value })
    }

    fn parse_delete_clause(
        &mut self,
        actions: &mut Vec<DeleteAction>,
    ) -> Result<(), ExpressionError> {
        actions.push(self.parse_delete_action()?);
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            actions.push(self.parse_delete_action()?);
        }
        Ok(())
    }

    fn parse_delete_action(&mut self) -> Result<DeleteAction, ExpressionError> {
        let path = self.parse_attribute_path()?;
        let value = self.parse_operand()?;
        Ok(DeleteAction { path, value })
    }
}

// ---------------------------------------------------------------------------
// Projection expression parsing
// ---------------------------------------------------------------------------

impl Parser {
    /// Parse a projection expression: comma-separated attribute paths.
    fn parse_projection_expr(&mut self) -> Result<Vec<AttributePath>, ExpressionError> {
        let mut paths = vec![self.parse_attribute_path()?];
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            paths.push(self.parse_attribute_path()?);
        }
        Ok(paths)
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a condition, filter, or key-condition expression.
///
/// # Errors
///
/// Returns `ExpressionError` if the expression is syntactically invalid.
pub fn parse_condition(input: &str) -> Result<Expr, ExpressionError> {
    let tokens = Lexer::new(input).tokenize()?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_or_expr()?;
    if !parser.at_end() {
        return Err(ExpressionError::UnexpectedToken {
            expected: "end of expression".to_owned(),
            found: parser.peek().to_string(),
        });
    }
    Ok(expr)
}

/// Parse an update expression (SET / REMOVE / ADD / DELETE).
///
/// # Errors
///
/// Returns `ExpressionError` if the expression is syntactically invalid.
pub fn parse_update(input: &str) -> Result<UpdateExpr, ExpressionError> {
    let tokens = Lexer::new(input).tokenize()?;
    let mut parser = Parser::new(tokens);
    let update = parser.parse_update_expr()?;

    if update.set_actions.is_empty()
        && update.remove_paths.is_empty()
        && update.add_actions.is_empty()
        && update.delete_actions.is_empty()
    {
        return Err(ExpressionError::UnexpectedToken {
            expected: "SET, REMOVE, ADD, or DELETE".to_owned(),
            found: "empty update expression".to_owned(),
        });
    }

    Ok(update)
}

/// Parse a projection expression (comma-separated attribute paths).
///
/// # Errors
///
/// Returns `ExpressionError` if the expression is syntactically invalid,
/// contains overlapping or conflicting paths, exceeds nesting depth, or
/// has duplicate top-level attributes.
pub fn parse_projection(input: &str) -> Result<Vec<AttributePath>, ExpressionError> {
    // Reject empty projection expression.
    if input.trim().is_empty() {
        return Err(ExpressionError::Validation {
            message: "Invalid ProjectionExpression: The expression can not be empty;".to_owned(),
        });
    }

    // Reject leading commas.
    if input.trim_start().starts_with(',') {
        return Err(ExpressionError::Validation {
            message: "Invalid ProjectionExpression: Syntax error; unexpected comma at start"
                .to_owned(),
        });
    }

    // Reject trailing commas.
    if input.trim_end().ends_with(',') {
        return Err(ExpressionError::Validation {
            message: "Invalid ProjectionExpression: Syntax error; unexpected comma at end"
                .to_owned(),
        });
    }

    // Reject empty segments between commas (e.g., "a,,b").
    // Check for consecutive commas with optional whitespace between them.
    {
        let trimmed = input.trim();
        let chars: Vec<char> = trimmed.chars().collect();
        let mut saw_comma = false;
        for &ch in &chars {
            if ch == ',' {
                if saw_comma {
                    return Err(ExpressionError::Validation {
                        message: "Invalid ProjectionExpression: Syntax error; empty segment \
                                  between commas"
                            .to_owned(),
                    });
                }
                saw_comma = true;
            } else if !ch.is_ascii_whitespace() {
                saw_comma = false;
            }
        }
    }

    let tokens = Lexer::new(input).tokenize()?;
    let mut parser = Parser::new(tokens);
    let paths = parser.parse_projection_expr()?;
    if !parser.at_end() {
        return Err(ExpressionError::UnexpectedToken {
            expected: "end of expression".to_owned(),
            found: parser.peek().to_string(),
        });
    }

    // Validate nesting depth: DynamoDB limits to 32 levels.
    for path in &paths {
        if path.elements.len() > 32 {
            return Err(ExpressionError::Validation {
                message: format!(
                    "Invalid ProjectionExpression: The document path has too many nesting \
                     levels; nesting levels: {}",
                    path.elements.len()
                ),
            });
        }
    }

    // Validate duplicate top-level attributes.
    {
        let mut seen: Vec<String> = Vec::new();
        for path in &paths {
            let repr = format!("{path}");
            if seen.contains(&repr) {
                return Err(ExpressionError::Validation {
                    message: format!(
                        "Invalid ProjectionExpression: Two document paths overlap with \
                         each other; must remove or rewrite one of these paths; path one: \
                         [{repr}], path two: [{repr}]"
                    ),
                });
            }
            seen.push(repr);
        }
    }

    // Validate overlapping and conflicting paths.
    validate_projection_paths(&paths)?;

    Ok(paths)
}

/// Represents a resolved element in a projection path for validation purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ResolvedPathElement {
    /// A map attribute access (via dot notation).
    MapKey(String),
    /// A list index access (via bracket notation).
    ListIndex(usize),
}

/// Convert an `AttributePath` to a vector of `ResolvedPathElement` for comparison.
fn path_to_resolved(path: &AttributePath) -> Vec<ResolvedPathElement> {
    path.elements
        .iter()
        .map(|e| match e {
            PathElement::Attribute(name) => ResolvedPathElement::MapKey(name.clone()),
            PathElement::Index(idx) => ResolvedPathElement::ListIndex(*idx),
        })
        .collect()
}

/// Validate that no two projection paths overlap or conflict.
///
/// - **Overlap**: One path is a prefix of another, or two paths are identical.
/// - **Conflict**: At a shared prefix point, one path accesses via dot (map key)
///   and the other via index (list), meaning the same node would need to be both
///   a map and a list simultaneously.
fn validate_projection_paths(paths: &[AttributePath]) -> Result<(), ExpressionError> {
    let resolved: Vec<Vec<ResolvedPathElement>> = paths.iter().map(path_to_resolved).collect();

    for i in 0..resolved.len() {
        for j in (i + 1)..resolved.len() {
            let a = &resolved[i];
            let b = &resolved[j];
            let min_len = a.len().min(b.len());

            // Check the shared prefix for conflicts.
            let mut prefix_matches = true;
            for k in 0..min_len {
                match (&a[k], &b[k]) {
                    (ResolvedPathElement::MapKey(ka), ResolvedPathElement::MapKey(kb)) => {
                        if ka != kb {
                            prefix_matches = false;
                            break;
                        }
                    }
                    (ResolvedPathElement::ListIndex(ia), ResolvedPathElement::ListIndex(ib)) => {
                        if ia != ib {
                            prefix_matches = false;
                            break;
                        }
                    }
                    // One is a map key and the other is a list index at the same depth:
                    // this is a conflict.
                    _ => {
                        return Err(ExpressionError::Validation {
                            message: format!(
                                "Invalid ProjectionExpression: Two document paths conflict \
                                 with each other; must remove or rewrite one of these paths; \
                                 path one: [{}], path two: [{}]",
                                paths[i], paths[j]
                            ),
                        });
                    }
                }
            }

            // If the entire shorter path matches as a prefix of the longer one,
            // they overlap.
            if prefix_matches && (a.len() != b.len()) {
                return Err(ExpressionError::Validation {
                    message: format!(
                        "Invalid ProjectionExpression: Two document paths overlap with \
                         each other; must remove or rewrite one of these paths; \
                         path one: [{}], path two: [{}]",
                        paths[i], paths[j]
                    ),
                });
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_simple_comparison() {
        let expr = parse_condition("#name = :val").unwrap();
        match &expr {
            Expr::Compare { left, op, right } => {
                assert!(matches!(left.as_ref(), Operand::Path(_)));
                assert_eq!(*op, CompareOp::Eq);
                assert!(matches!(right.as_ref(), Operand::Value(v) if v == "val"));
            }
            other => panic!("expected Compare, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_and_condition() {
        let expr = parse_condition("#a = :v1 AND #b = :v2").unwrap();
        match &expr {
            Expr::Logical { op, left, right } => {
                assert_eq!(*op, LogicalOp::And);
                assert!(matches!(left.as_ref(), Expr::Compare { .. }));
                assert!(matches!(right.as_ref(), Expr::Compare { .. }));
            }
            other => panic!("expected Logical AND, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_between() {
        let expr = parse_condition("#age BETWEEN :low AND :high").unwrap();
        match &expr {
            Expr::Between { value, low, high } => {
                assert!(matches!(value.as_ref(), Operand::Path(_)));
                assert!(matches!(low.as_ref(), Operand::Value(v) if v == "low"));
                assert!(matches!(high.as_ref(), Operand::Value(v) if v == "high"));
            }
            other => panic!("expected Between, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_function() {
        let expr = parse_condition("attribute_exists(#name)").unwrap();
        match &expr {
            Expr::Function { name, args } => {
                assert_eq!(*name, FunctionName::AttributeExists);
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_nested_path() {
        let expr = parse_condition("info.rating = :val").unwrap();
        match &expr {
            Expr::Compare { left, .. } => match left.as_ref() {
                Operand::Path(path) => {
                    assert_eq!(path.elements.len(), 2);
                    assert!(matches!(&path.elements[0], PathElement::Attribute(n) if n == "info"));
                    assert!(
                        matches!(&path.elements[1], PathElement::Attribute(n) if n == "rating")
                    );
                }
                other => panic!("expected Path operand, got {other:?}"),
            },
            other => panic!("expected Compare, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_update_set() {
        let update = parse_update("SET #name = :val").unwrap();
        assert_eq!(update.set_actions.len(), 1);
        assert!(update.remove_paths.is_empty());
        assert!(update.add_actions.is_empty());
        assert!(update.delete_actions.is_empty());

        let action = &update.set_actions[0];
        assert!(matches!(&action.value, SetValue::Operand(Operand::Value(v)) if v == "val"));
    }

    #[test]
    fn test_should_parse_update_remove() {
        let update = parse_update("REMOVE #attr").unwrap();
        assert!(update.set_actions.is_empty());
        assert_eq!(update.remove_paths.len(), 1);
    }

    #[test]
    fn test_should_parse_projection() {
        let paths = parse_projection("id, name, info.rating").unwrap();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].elements.len(), 1);
        assert_eq!(paths[1].elements.len(), 1);
        assert_eq!(paths[2].elements.len(), 2);
    }

    #[test]
    fn test_should_parse_or_expression() {
        let expr = parse_condition("#a = :v1 OR #b = :v2").unwrap();
        assert!(matches!(
            &expr,
            Expr::Logical {
                op: LogicalOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn test_should_parse_not_expression() {
        let expr = parse_condition("NOT #a = :v1").unwrap();
        assert!(matches!(&expr, Expr::Not(_)));
    }

    #[test]
    fn test_should_parse_in_expression() {
        let expr = parse_condition("#status IN (:v1, :v2, :v3)").unwrap();
        match &expr {
            Expr::In { list, .. } => assert_eq!(list.len(), 3),
            other => panic!("expected In, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_size_operand() {
        let expr = parse_condition("size(#name) > :val").unwrap();
        match &expr {
            Expr::Compare { left, op, .. } => {
                assert!(matches!(left.as_ref(), Operand::Size(_)));
                assert_eq!(*op, CompareOp::Gt);
            }
            other => panic!("expected Compare with Size operand, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_update_set_with_arithmetic() {
        let update = parse_update("SET #count = #count + :inc").unwrap();
        assert_eq!(update.set_actions.len(), 1);
        assert!(matches!(&update.set_actions[0].value, SetValue::Plus(_, _)));
    }

    #[test]
    fn test_should_parse_update_set_if_not_exists() {
        let update = parse_update("SET #name = if_not_exists(#name, :default)").unwrap();
        assert_eq!(update.set_actions.len(), 1);
        assert!(matches!(
            &update.set_actions[0].value,
            SetValue::IfNotExists(_, _)
        ));
    }

    #[test]
    fn test_should_parse_update_set_list_append() {
        let update = parse_update("SET #list = list_append(#list, :vals)").unwrap();
        assert_eq!(update.set_actions.len(), 1);
        assert!(matches!(
            &update.set_actions[0].value,
            SetValue::ListAppend(_, _)
        ));
    }

    #[test]
    fn test_should_parse_update_add_clause() {
        let update = parse_update("ADD #count :inc").unwrap();
        assert_eq!(update.add_actions.len(), 1);
    }

    #[test]
    fn test_should_parse_update_delete_clause() {
        let update = parse_update("DELETE #tags :removeTags").unwrap();
        assert_eq!(update.delete_actions.len(), 1);
    }

    #[test]
    fn test_should_parse_combined_update() {
        let update = parse_update("SET #a = :v1 REMOVE #b ADD #c :v2").unwrap();
        assert_eq!(update.set_actions.len(), 1);
        assert_eq!(update.remove_paths.len(), 1);
        assert_eq!(update.add_actions.len(), 1);
    }

    #[test]
    fn test_should_parse_path_with_index() {
        let expr = parse_condition("myList[0] = :val").unwrap();
        match &expr {
            Expr::Compare { left, .. } => match left.as_ref() {
                Operand::Path(path) => {
                    assert_eq!(path.elements.len(), 2);
                    assert!(
                        matches!(&path.elements[0], PathElement::Attribute(n) if n == "myList")
                    );
                    assert!(matches!(&path.elements[1], PathElement::Index(0)));
                }
                other => panic!("expected Path operand, got {other:?}"),
            },
            other => panic!("expected Compare, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_begins_with_function() {
        let expr = parse_condition("begins_with(#name, :prefix)").unwrap();
        match &expr {
            Expr::Function { name, args } => {
                assert_eq!(*name, FunctionName::BeginsWith);
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_contains_function() {
        let expr = parse_condition("contains(#tags, :tag)").unwrap();
        match &expr {
            Expr::Function { name, args } => {
                assert_eq!(*name, FunctionName::Contains);
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn test_should_error_on_unexpected_token() {
        let result = parse_condition("= :val");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_error_on_empty_input() {
        let result = parse_condition("");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_parse_case_insensitive_keywords() {
        let expr = parse_condition("#a = :v1 and #b = :v2").unwrap();
        assert!(matches!(
            &expr,
            Expr::Logical {
                op: LogicalOp::And,
                ..
            }
        ));

        let expr = parse_condition("#a = :v1 AND #b = :v2").unwrap();
        assert!(matches!(
            &expr,
            Expr::Logical {
                op: LogicalOp::And,
                ..
            }
        ));
    }

    #[test]
    fn test_should_parse_all_comparison_operators() {
        for (input, expected_op) in [
            ("#a = :v", CompareOp::Eq),
            ("#a <> :v", CompareOp::Ne),
            ("#a < :v", CompareOp::Lt),
            ("#a <= :v", CompareOp::Le),
            ("#a > :v", CompareOp::Gt),
            ("#a >= :v", CompareOp::Ge),
        ] {
            let expr = parse_condition(input).unwrap();
            match &expr {
                Expr::Compare { op, .. } => {
                    assert_eq!(*op, expected_op, "failed for input: {input}");
                }
                other => panic!("expected Compare for '{input}', got {other:?}"),
            }
        }
    }

    #[test]
    fn test_should_error_on_empty_update_expression() {
        let result = parse_update("");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_error_on_whitespace_only_update_expression() {
        let result = parse_update("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_parse_if_not_exists_plus_value() {
        let update = parse_update("SET #c = if_not_exists(#c, :zero) + :one").unwrap();
        assert_eq!(update.set_actions.len(), 1);
        assert!(matches!(&update.set_actions[0].value, SetValue::Plus(_, _)));
        // The left side should be IfNotExists, the right side an Operand.
        match &update.set_actions[0].value {
            SetValue::Plus(left, right) => {
                assert!(matches!(left.as_ref(), SetValue::IfNotExists(_, _)));
                assert!(
                    matches!(right.as_ref(), SetValue::Operand(Operand::Value(v)) if v == "one")
                );
            }
            other => panic!("expected Plus, got {other:?}"),
        }
    }

    #[test]
    fn test_should_parse_if_not_exists_minus_value() {
        let update = parse_update("SET #c = if_not_exists(#c, :ten) - :one").unwrap();
        assert_eq!(update.set_actions.len(), 1);
        match &update.set_actions[0].value {
            SetValue::Minus(left, right) => {
                assert!(matches!(left.as_ref(), SetValue::IfNotExists(_, _)));
                assert!(
                    matches!(right.as_ref(), SetValue::Operand(Operand::Value(v)) if v == "one")
                );
            }
            other => panic!("expected Minus, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Projection expression validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_reject_empty_projection_expression() {
        let result = parse_projection("");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty"), "expected 'empty' in: {err}");
    }

    #[test]
    fn test_should_reject_whitespace_only_projection() {
        let result = parse_projection("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_reject_leading_comma_projection() {
        let result = parse_projection(",a");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("comma") || err.contains("Syntax"),
            "expected comma/Syntax error in: {err}"
        );
    }

    #[test]
    fn test_should_reject_trailing_comma_projection() {
        let result = parse_projection("a,");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("comma") || err.contains("Syntax"),
            "expected comma/Syntax error in: {err}"
        );
    }

    #[test]
    fn test_should_reject_empty_segment_between_commas() {
        let result = parse_projection("a,,b");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("empty") || err.contains("comma"),
            "expected empty/comma error in: {err}"
        );
    }

    #[test]
    fn test_should_reject_duplicate_top_level_attributes() {
        let result = parse_projection("a, a");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("overlap"), "expected 'overlap' in: {err}");
    }

    #[test]
    fn test_should_reject_overlapping_paths_prefix() {
        // "a" is a prefix of "a.b" -- they overlap.
        let result = parse_projection("a, a.b");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("overlap"), "expected 'overlap' in: {err}");
    }

    #[test]
    fn test_should_reject_overlapping_paths_reverse() {
        let result = parse_projection("a.b, a");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("overlap"), "expected 'overlap' in: {err}");
    }

    #[test]
    fn test_should_reject_conflicting_paths_map_vs_index() {
        // "a.b" treats a as a map, "a[0]" treats a as a list -- conflict.
        let result = parse_projection("a.b, a[0]");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("conflict"), "expected 'conflict' in: {err}");
    }

    #[test]
    fn test_should_reject_nesting_depth_exceeds_32() {
        // Create a path with 33 elements.
        let path = (0..33)
            .map(|i| format!("a{i}"))
            .collect::<Vec<_>>()
            .join(".");
        let result = parse_projection(&path);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("nesting levels"),
            "expected 'nesting levels' in: {err}"
        );
    }

    #[test]
    fn test_should_accept_nesting_depth_at_32() {
        // 32 elements is the maximum allowed.
        let path = (0..32)
            .map(|i| format!("a{i}"))
            .collect::<Vec<_>>()
            .join(".");
        let result = parse_projection(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_reject_negative_index() {
        let result = parse_condition("a[-1] = :v");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Syntax error"),
            "expected 'Syntax error' in: {err}"
        );
    }

    #[test]
    fn test_should_accept_non_overlapping_sibling_paths() {
        // "a.b" and "a.c" share prefix "a" but are siblings, not overlapping.
        let result = parse_projection("a.b, a.c");
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_accept_different_indices() {
        // "a[0]" and "a[1]" share prefix "a" but are different indices.
        let result = parse_projection("a[0], a[1]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_reject_identical_nested_paths() {
        let result = parse_projection("a.b.c, a.b.c");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("overlap"), "expected 'overlap' in: {err}");
    }
}
