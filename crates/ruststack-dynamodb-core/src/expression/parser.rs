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
                found: format!("'{ch}'"),
            }),
        }
    }

    fn read_expr_attr_name(&mut self) -> Result<Token, ExpressionError> {
        self.chars.next(); // consume '#'
        let name = self.read_ident_chars();
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
        let name = self.read_ident_chars();
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
        let n: usize = s.parse().map_err(|_| ExpressionError::InvalidOperand {
            operation: "index parse".to_owned(),
            message: format!("'{s}' is not a valid index"),
        })?;
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

    fn read_identifier_or_keyword(&mut self) -> Token {
        let ident = self.read_ident_chars();
        let lower = ident.to_ascii_lowercase();
        match lower.as_str() {
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "between" => Token::Between,
            "in" => Token::In,
            "set" => Token::Set,
            "remove" => Token::Remove,
            "add" => Token::Add,
            "delete" => Token::Delete,
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
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
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
            _ => Err(ExpressionError::UnexpectedToken {
                expected: "comparison operator, BETWEEN, or IN".to_owned(),
                found: self.peek().to_string(),
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
                let path = self.parse_attribute_path()?;
                self.expect(&Token::RParen)?;
                Ok(Operand::Size(path))
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
                    let Token::Number(idx) = self.advance() else {
                        return Err(ExpressionError::UnexpectedToken {
                            expected: "number".to_owned(),
                            found: "non-number".to_owned(),
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
    fn parse_update_expr(&mut self) -> Result<UpdateExpr, ExpressionError> {
        let mut update = UpdateExpr {
            set_actions: Vec::new(),
            remove_paths: Vec::new(),
            add_actions: Vec::new(),
            delete_actions: Vec::new(),
        };

        while !self.at_end() {
            match self.peek() {
                Token::Set => {
                    self.advance();
                    self.parse_set_clause(&mut update.set_actions)?;
                }
                Token::Remove => {
                    self.advance();
                    self.parse_remove_clause(&mut update.remove_paths)?;
                }
                Token::Add => {
                    self.advance();
                    self.parse_add_clause(&mut update.add_actions)?;
                }
                Token::Delete => {
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
    fn parse_set_value(&mut self) -> Result<SetValue, ExpressionError> {
        // Check for if_not_exists and list_append first
        match self.peek() {
            Token::IfNotExists => return self.parse_if_not_exists(),
            Token::ListAppend => return self.parse_list_append(),
            _ => {}
        }

        let first = self.parse_operand()?;

        match self.peek() {
            Token::Plus => {
                self.advance();
                let second = self.parse_operand()?;
                Ok(SetValue::Plus(first, second))
            }
            Token::Minus => {
                self.advance();
                let second = self.parse_operand()?;
                Ok(SetValue::Minus(first, second))
            }
            _ => Ok(SetValue::Operand(first)),
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
/// Returns `ExpressionError` if the expression is syntactically invalid.
pub fn parse_projection(input: &str) -> Result<Vec<AttributePath>, ExpressionError> {
    let tokens = Lexer::new(input).tokenize()?;
    let mut parser = Parser::new(tokens);
    let paths = parser.parse_projection_expr()?;
    if !parser.at_end() {
        return Err(ExpressionError::UnexpectedToken {
            expected: "end of expression".to_owned(),
            found: parser.peek().to_string(),
        });
    }
    Ok(paths)
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
}
