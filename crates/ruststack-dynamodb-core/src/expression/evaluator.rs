//! Expression evaluator for DynamoDB condition, update, and projection expressions.
//!
//! The evaluator resolves expression attribute names and values against an item,
//! then evaluates condition expressions to booleans, applies update mutations, or
//! projects attributes.

use std::collections::HashMap;

use ruststack_dynamodb_model::AttributeValue;

use super::ast::{
    AddAction, AttributePath, CompareOp, DeleteAction, Expr, FunctionName, LogicalOp, Operand,
    PathElement, SetAction, SetValue, UpdateExpr,
};
use super::parser::ExpressionError;

// ---------------------------------------------------------------------------
// Evaluation context
// ---------------------------------------------------------------------------

/// Evaluation context binding an item to its expression attribute name/value mappings.
#[derive(Debug)]
pub struct EvalContext<'a> {
    /// The DynamoDB item being evaluated.
    pub item: &'a HashMap<String, AttributeValue>,
    /// Expression attribute name substitutions (`#name` -> actual attribute name).
    pub names: &'a HashMap<String, String>,
    /// Expression attribute value substitutions (`:val` -> `AttributeValue`).
    pub values: &'a HashMap<String, AttributeValue>,
}

// ---------------------------------------------------------------------------
// Condition evaluation
// ---------------------------------------------------------------------------

impl EvalContext<'_> {
    /// Evaluate a condition expression against the item, returning `true` or `false`.
    ///
    /// # Errors
    ///
    /// Returns `ExpressionError` if attribute references cannot be resolved or
    /// types are incompatible.
    pub fn evaluate(&self, expr: &Expr) -> Result<bool, ExpressionError> {
        match expr {
            Expr::Compare { left, op, right } => self.eval_compare(left, *op, right),
            Expr::Between { value, low, high } => self.eval_between(value, low, high),
            Expr::In { value, list } => self.eval_in(value, list),
            Expr::Logical { op, left, right } => self.eval_logical(*op, left, right),
            Expr::Not(inner) => self.evaluate(inner).map(|v| !v),
            Expr::Function { name, args } => self.eval_function(*name, args),
        }
    }

    fn eval_compare(
        &self,
        left: &Operand,
        op: CompareOp,
        right: &Operand,
    ) -> Result<bool, ExpressionError> {
        let lval = self.resolve_operand(left)?;
        let rval = self.resolve_operand(right)?;

        let (Some(lv), Some(rv)) = (&lval, &rval) else {
            // If either side is missing, comparison is false (DynamoDB behavior).
            return Ok(false);
        };

        compare_values(lv, rv, op)
    }

    fn eval_between(
        &self,
        value: &Operand,
        low: &Operand,
        high: &Operand,
    ) -> Result<bool, ExpressionError> {
        let v = self.resolve_operand(value)?;
        let lo = self.resolve_operand(low)?;
        let hi = self.resolve_operand(high)?;

        let (Some(v), Some(lo), Some(hi)) = (&v, &lo, &hi) else {
            return Ok(false);
        };

        let ge_low = compare_values(v, lo, CompareOp::Ge)?;
        let le_high = compare_values(v, hi, CompareOp::Le)?;
        Ok(ge_low && le_high)
    }

    fn eval_in(&self, value: &Operand, list: &[Operand]) -> Result<bool, ExpressionError> {
        let v = self.resolve_operand(value)?;
        let Some(v) = &v else {
            return Ok(false);
        };
        for item in list {
            let item_val = self.resolve_operand(item)?;
            if let Some(iv) = &item_val {
                if compare_values(v, iv, CompareOp::Eq)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn eval_logical(
        &self,
        op: LogicalOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<bool, ExpressionError> {
        match op {
            LogicalOp::And => {
                if !self.evaluate(left)? {
                    return Ok(false);
                }
                self.evaluate(right)
            }
            LogicalOp::Or => {
                if self.evaluate(left)? {
                    return Ok(true);
                }
                self.evaluate(right)
            }
        }
    }

    fn eval_function(&self, name: FunctionName, args: &[Operand]) -> Result<bool, ExpressionError> {
        match name {
            FunctionName::AttributeExists => {
                let path = operand_as_path(&args[0], "attribute_exists")?;
                Ok(self.resolve_path(path).is_some())
            }
            FunctionName::AttributeNotExists => {
                let path = operand_as_path(&args[0], "attribute_not_exists")?;
                Ok(self.resolve_path(path).is_none())
            }
            FunctionName::AttributeType => {
                let path = operand_as_path(&args[0], "attribute_type")?;
                let type_val = self.resolve_operand(&args[1])?;
                let Some(AttributeValue::S(expected_type)) = type_val else {
                    return Err(ExpressionError::TypeMismatch {
                        message: "attribute_type second argument must be a string".to_owned(),
                    });
                };
                match self.resolve_path(path) {
                    Some(val) => Ok(val.type_descriptor() == expected_type),
                    None => Ok(false),
                }
            }
            FunctionName::BeginsWith => {
                let path = operand_as_path(&args[0], "begins_with")?;
                let prefix_val = self.resolve_operand(&args[1])?;
                let Some(attr) = self.resolve_path(path) else {
                    return Ok(false);
                };
                let Some(s) = attr.as_s() else {
                    return Ok(false);
                };
                let Some(AttributeValue::S(prefix)) = prefix_val else {
                    return Err(ExpressionError::TypeMismatch {
                        message: "begins_with prefix must be a string".to_owned(),
                    });
                };
                Ok(s.starts_with(&prefix))
            }
            FunctionName::Contains => self.eval_contains(args),
            FunctionName::Size => Err(ExpressionError::InvalidOperand {
                operation: "condition".to_owned(),
                message: "size() cannot be used as a standalone condition; use it in a comparison"
                    .to_owned(),
            }),
        }
    }

    fn eval_contains(&self, args: &[Operand]) -> Result<bool, ExpressionError> {
        let path = operand_as_path(&args[0], "contains")?;
        let search_val = self.resolve_operand(&args[1])?;
        let Some(attr) = self.resolve_path(path) else {
            return Ok(false);
        };
        let Some(search) = search_val else {
            return Ok(false);
        };
        match (attr, &search) {
            (AttributeValue::S(s), AttributeValue::S(sub)) => Ok(s.contains(sub.as_str())),
            (AttributeValue::Ss(set), AttributeValue::S(val))
            | (AttributeValue::Ns(set), AttributeValue::N(val)) => Ok(set.contains(val)),
            (AttributeValue::Bs(set), AttributeValue::B(val)) => Ok(set.contains(val)),
            (AttributeValue::L(list), _) => Ok(list.contains(&search)),
            _ => Ok(false),
        }
    }
}

// ---------------------------------------------------------------------------
// Operand resolution
// ---------------------------------------------------------------------------

impl EvalContext<'_> {
    /// Resolve an operand to its concrete `AttributeValue`, if present.
    ///
    /// # Errors
    ///
    /// Returns `ExpressionError` if a value reference (`:name`) cannot be found
    /// in the values map.
    pub fn resolve_operand(
        &self,
        operand: &Operand,
    ) -> Result<Option<AttributeValue>, ExpressionError> {
        match operand {
            Operand::Path(path) => Ok(self.resolve_path(path).cloned()),
            Operand::Value(name) => {
                let key = format!(":{name}");
                self.values.get(&key).cloned().map_or_else(
                    || Err(ExpressionError::UnresolvedValue { name: key.clone() }),
                    |v| Ok(Some(v)),
                )
            }
            Operand::Size(path) => {
                let val = self.resolve_path(path);
                match val {
                    Some(v) => {
                        let size = attribute_size(v);
                        Ok(Some(AttributeValue::N(size.to_string())))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    /// Walk an attribute path against the item, resolving `#name` placeholders
    /// through the names map.
    #[must_use]
    pub fn resolve_path(&self, path: &AttributePath) -> Option<&AttributeValue> {
        let mut current: Option<&AttributeValue> = None;

        for (i, element) in path.elements.iter().enumerate() {
            match element {
                PathElement::Attribute(name) => {
                    let resolved_name = if name.starts_with('#') {
                        self.names.get(name.as_str())?
                    } else {
                        name
                    };
                    if i == 0 {
                        current = self.item.get(resolved_name);
                    } else {
                        let map = current?.as_m()?;
                        current = map.get(resolved_name);
                    }
                }
                PathElement::Index(idx) => {
                    let list = current?.as_l()?;
                    current = list.get(*idx);
                }
            }
        }

        current
    }
}

// ---------------------------------------------------------------------------
// Update application
// ---------------------------------------------------------------------------

impl EvalContext<'_> {
    /// Apply an update expression to the item, returning the modified item.
    ///
    /// The original item is cloned; the returned map contains the result of all
    /// SET, REMOVE, ADD, and DELETE actions.
    ///
    /// # Errors
    ///
    /// Returns `ExpressionError` if operands cannot be resolved or types are
    /// incompatible for the requested operation.
    pub fn apply_update(
        &self,
        update: &UpdateExpr,
    ) -> Result<HashMap<String, AttributeValue>, ExpressionError> {
        let mut result = self.item.clone();

        for action in &update.set_actions {
            self.apply_set_action(&mut result, action)?;
        }
        for path in &update.remove_paths {
            apply_remove(&mut result, path, self.names);
        }
        for action in &update.add_actions {
            self.apply_add_action(&mut result, action)?;
        }
        for action in &update.delete_actions {
            self.apply_delete_action(&mut result, action)?;
        }

        Ok(result)
    }

    fn apply_set_action(
        &self,
        item: &mut HashMap<String, AttributeValue>,
        action: &SetAction,
    ) -> Result<(), ExpressionError> {
        let value = self.resolve_set_value(&action.value)?;
        set_path_value(item, &action.path, value, self.names);
        Ok(())
    }

    fn resolve_set_value(&self, set_value: &SetValue) -> Result<AttributeValue, ExpressionError> {
        match set_value {
            SetValue::Operand(op) => {
                self.resolve_operand(op)?
                    .ok_or_else(|| ExpressionError::InvalidOperand {
                        operation: "SET".to_owned(),
                        message: "operand resolved to None".to_owned(),
                    })
            }
            SetValue::Plus(a, b) => {
                let av =
                    self.resolve_operand(a)?
                        .ok_or_else(|| ExpressionError::InvalidOperand {
                            operation: "SET addition".to_owned(),
                            message: "left operand resolved to None".to_owned(),
                        })?;
                let bv =
                    self.resolve_operand(b)?
                        .ok_or_else(|| ExpressionError::InvalidOperand {
                            operation: "SET addition".to_owned(),
                            message: "right operand resolved to None".to_owned(),
                        })?;
                numeric_arithmetic(&av, &bv, true)
            }
            SetValue::Minus(a, b) => {
                let av =
                    self.resolve_operand(a)?
                        .ok_or_else(|| ExpressionError::InvalidOperand {
                            operation: "SET subtraction".to_owned(),
                            message: "left operand resolved to None".to_owned(),
                        })?;
                let bv =
                    self.resolve_operand(b)?
                        .ok_or_else(|| ExpressionError::InvalidOperand {
                            operation: "SET subtraction".to_owned(),
                            message: "right operand resolved to None".to_owned(),
                        })?;
                numeric_arithmetic(&av, &bv, false)
            }
            SetValue::IfNotExists(path, default) => {
                if let Some(existing) = self.resolve_path(path) {
                    Ok(existing.clone())
                } else {
                    self.resolve_operand(default)?
                        .ok_or_else(|| ExpressionError::InvalidOperand {
                            operation: "if_not_exists".to_owned(),
                            message: "default operand resolved to None".to_owned(),
                        })
                }
            }
            SetValue::ListAppend(a, b) => {
                let av =
                    self.resolve_operand(a)?
                        .ok_or_else(|| ExpressionError::InvalidOperand {
                            operation: "list_append".to_owned(),
                            message: "first operand resolved to None".to_owned(),
                        })?;
                let bv =
                    self.resolve_operand(b)?
                        .ok_or_else(|| ExpressionError::InvalidOperand {
                            operation: "list_append".to_owned(),
                            message: "second operand resolved to None".to_owned(),
                        })?;
                match (av, bv) {
                    (AttributeValue::L(mut list_a), AttributeValue::L(list_b)) => {
                        list_a.extend(list_b);
                        Ok(AttributeValue::L(list_a))
                    }
                    _ => Err(ExpressionError::TypeMismatch {
                        message: "list_append requires two list operands".to_owned(),
                    }),
                }
            }
        }
    }

    fn apply_add_action(
        &self,
        item: &mut HashMap<String, AttributeValue>,
        action: &AddAction,
    ) -> Result<(), ExpressionError> {
        let add_val = self.resolve_operand(&action.value)?.ok_or_else(|| {
            ExpressionError::InvalidOperand {
                operation: "ADD".to_owned(),
                message: "value operand resolved to None".to_owned(),
            }
        })?;

        let attr_name = resolve_top_level_name(&action.path, self.names)?;
        let existing = item.get(&attr_name);

        match (&add_val, existing) {
            // ADD to a number: increment
            (AttributeValue::N(_), Some(existing_val)) => {
                let result = numeric_arithmetic(existing_val, &add_val, true)?;
                item.insert(attr_name, result);
            }
            // ADD to a string set
            (AttributeValue::Ss(new_items), Some(AttributeValue::Ss(existing_set))) => {
                let mut merged = existing_set.clone();
                for s in new_items {
                    if !merged.contains(s) {
                        merged.push(s.clone());
                    }
                }
                item.insert(attr_name, AttributeValue::Ss(merged));
            }
            // ADD to a number set
            (AttributeValue::Ns(new_items), Some(AttributeValue::Ns(existing_set))) => {
                let mut merged = existing_set.clone();
                for n in new_items {
                    if !merged.contains(n) {
                        merged.push(n.clone());
                    }
                }
                item.insert(attr_name, AttributeValue::Ns(merged));
            }
            // ADD to a binary set
            (AttributeValue::Bs(new_items), Some(AttributeValue::Bs(existing_set))) => {
                let mut merged = existing_set.clone();
                for b in new_items {
                    if !merged.contains(b) {
                        merged.push(b.clone());
                    }
                }
                item.insert(attr_name, AttributeValue::Bs(merged));
            }
            // ADD a number or set to non-existing attribute: set it directly
            (
                AttributeValue::N(_)
                | AttributeValue::Ss(_)
                | AttributeValue::Ns(_)
                | AttributeValue::Bs(_),
                None,
            ) => {
                item.insert(attr_name, add_val);
            }
            _ => {
                return Err(ExpressionError::TypeMismatch {
                    message: "ADD requires a number or set value".to_owned(),
                });
            }
        }

        Ok(())
    }

    fn apply_delete_action(
        &self,
        item: &mut HashMap<String, AttributeValue>,
        action: &DeleteAction,
    ) -> Result<(), ExpressionError> {
        let del_val = self.resolve_operand(&action.value)?.ok_or_else(|| {
            ExpressionError::InvalidOperand {
                operation: "DELETE".to_owned(),
                message: "value operand resolved to None".to_owned(),
            }
        })?;

        let attr_name = resolve_top_level_name(&action.path, self.names)?;
        let Some(existing) = item.get(&attr_name) else {
            return Ok(());
        };

        match (&del_val, existing) {
            (AttributeValue::Ss(to_remove), AttributeValue::Ss(existing_set)) => {
                let filtered: Vec<String> = existing_set
                    .iter()
                    .filter(|s| !to_remove.contains(s))
                    .cloned()
                    .collect();
                item.insert(attr_name, AttributeValue::Ss(filtered));
            }
            (AttributeValue::Ns(to_remove), AttributeValue::Ns(existing_set)) => {
                let filtered: Vec<String> = existing_set
                    .iter()
                    .filter(|n| !to_remove.contains(n))
                    .cloned()
                    .collect();
                item.insert(attr_name, AttributeValue::Ns(filtered));
            }
            (AttributeValue::Bs(to_remove), AttributeValue::Bs(existing_set)) => {
                let filtered: Vec<bytes::Bytes> = existing_set
                    .iter()
                    .filter(|b| !to_remove.contains(b))
                    .cloned()
                    .collect();
                item.insert(attr_name, AttributeValue::Bs(filtered));
            }
            _ => {
                return Err(ExpressionError::TypeMismatch {
                    message: "DELETE requires a set value matching the existing attribute type"
                        .to_owned(),
                });
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Projection
// ---------------------------------------------------------------------------

impl EvalContext<'_> {
    /// Apply a projection expression, returning a new item containing only the
    /// requested attributes.
    #[must_use]
    pub fn apply_projection(&self, paths: &[AttributePath]) -> HashMap<String, AttributeValue> {
        let mut result = HashMap::new();
        for path in paths {
            if let Some(val) = self.resolve_path(path) {
                if let Some(top_name) = resolve_top_level_name_opt(path, self.names) {
                    if path.elements.len() == 1 {
                        result.insert(top_name, val.clone());
                    } else {
                        // For nested paths, insert the top-level attribute with the nested value.
                        // A full implementation would reconstruct the nested structure; here we
                        // insert the full top-level attribute for simplicity, which is the common
                        // DynamoDB projection behavior for document paths.
                        if let Some(top_val) = self.item.get(&top_name) {
                            result.insert(top_name, top_val.clone());
                        }
                    }
                }
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Compare two `AttributeValue`s using the given comparison operator.
fn compare_values(
    left: &AttributeValue,
    right: &AttributeValue,
    op: CompareOp,
) -> Result<bool, ExpressionError> {
    match (left, right) {
        (AttributeValue::S(a), AttributeValue::S(b)) => Ok(compare_ord(a, b, op)),
        (AttributeValue::N(a), AttributeValue::N(b)) => {
            let fa = parse_number(a)?;
            let fb = parse_number(b)?;
            Ok(compare_f64(fa, fb, op))
        }
        (AttributeValue::B(a), AttributeValue::B(b)) => Ok(compare_ord(a, b, op)),
        (AttributeValue::Bool(a), AttributeValue::Bool(b)) => Ok(compare_ord(a, b, op)),
        (AttributeValue::Null(true), AttributeValue::Null(true)) => {
            Ok(matches!(op, CompareOp::Eq | CompareOp::Le | CompareOp::Ge))
        }
        _ => {
            // Different types are not comparable; equality is false, others are also false.
            Ok(matches!(op, CompareOp::Ne))
        }
    }
}

/// Generic ordered comparison.
fn compare_ord<T: Ord>(a: &T, b: &T, op: CompareOp) -> bool {
    match op {
        CompareOp::Eq => a == b,
        CompareOp::Ne => a != b,
        CompareOp::Lt => a < b,
        CompareOp::Le => a <= b,
        CompareOp::Gt => a > b,
        CompareOp::Ge => a >= b,
    }
}

/// Floating-point comparison for DynamoDB number values.
fn compare_f64(a: f64, b: f64, op: CompareOp) -> bool {
    match op {
        CompareOp::Eq => (a - b).abs() < f64::EPSILON,
        CompareOp::Ne => (a - b).abs() >= f64::EPSILON,
        CompareOp::Lt => a < b,
        CompareOp::Le => a <= b,
        CompareOp::Gt => a > b,
        CompareOp::Ge => a >= b,
    }
}

/// Parse a DynamoDB number string to f64.
fn parse_number(s: &str) -> Result<f64, ExpressionError> {
    s.parse::<f64>().map_err(|_| ExpressionError::TypeMismatch {
        message: format!("'{s}' is not a valid number"),
    })
}

/// Perform numeric addition or subtraction.
fn numeric_arithmetic(
    a: &AttributeValue,
    b: &AttributeValue,
    is_add: bool,
) -> Result<AttributeValue, ExpressionError> {
    let (AttributeValue::N(na), AttributeValue::N(nb)) = (a, b) else {
        return Err(ExpressionError::TypeMismatch {
            message: "arithmetic requires number operands".to_owned(),
        });
    };
    let fa = parse_number(na)?;
    let fb = parse_number(nb)?;
    let result = if is_add { fa + fb } else { fa - fb };
    Ok(AttributeValue::N(format_number(result)))
}

/// Format a number, preferring integer representation when the value is integral.
fn format_number(v: f64) -> String {
    // Safe to truncate: we already verified the value is integral and within i64 range.
    #[allow(clippy::float_cmp, clippy::cast_possible_truncation)]
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        v.to_string()
    }
}

/// Compute the size of an `AttributeValue`.
#[must_use]
fn attribute_size(val: &AttributeValue) -> usize {
    match val {
        AttributeValue::S(s) => s.len(),
        AttributeValue::N(n) => n.len(),
        AttributeValue::B(b) => b.len(),
        AttributeValue::Ss(v) | AttributeValue::Ns(v) => v.len(),
        AttributeValue::Bs(v) => v.len(),
        AttributeValue::Bool(_) | AttributeValue::Null(_) => 1,
        AttributeValue::L(v) => v.len(),
        AttributeValue::M(m) => m.len(),
    }
}

/// Extract the path argument from an operand, returning an error if it is not a path.
fn operand_as_path<'o>(
    operand: &'o Operand,
    function_name: &str,
) -> Result<&'o AttributePath, ExpressionError> {
    match operand {
        Operand::Path(path) => Ok(path),
        _ => Err(ExpressionError::InvalidOperand {
            operation: function_name.to_owned(),
            message: "argument must be an attribute path".to_owned(),
        }),
    }
}

/// Resolve the top-level attribute name from a path (handling `#name` substitution).
fn resolve_top_level_name(
    path: &AttributePath,
    names: &HashMap<String, String>,
) -> Result<String, ExpressionError> {
    let Some(PathElement::Attribute(name)) = path.elements.first() else {
        return Err(ExpressionError::InvalidOperand {
            operation: "path resolution".to_owned(),
            message: "path must start with an attribute name".to_owned(),
        });
    };
    if name.starts_with('#') {
        names
            .get(name.as_str())
            .cloned()
            .ok_or_else(|| ExpressionError::UnresolvedName { name: name.clone() })
    } else {
        Ok(name.clone())
    }
}

/// Same as `resolve_top_level_name` but returns `Option` instead of `Result`.
fn resolve_top_level_name_opt(
    path: &AttributePath,
    names: &HashMap<String, String>,
) -> Option<String> {
    let PathElement::Attribute(name) = path.elements.first()? else {
        return None;
    };
    if name.starts_with('#') {
        names.get(name.as_str()).cloned()
    } else {
        Some(name.clone())
    }
}

/// Set a value at the given path in an item. For top-level paths, this inserts
/// directly into the map. For nested paths, it traverses/creates intermediate maps.
fn set_path_value(
    item: &mut HashMap<String, AttributeValue>,
    path: &AttributePath,
    value: AttributeValue,
    names: &HashMap<String, String>,
) {
    if path.elements.is_empty() {
        return;
    }

    let top_name = match &path.elements[0] {
        PathElement::Attribute(name) => {
            if name.starts_with('#') {
                names
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            }
        }
        PathElement::Index(_) => return,
    };

    if path.elements.len() == 1 {
        item.insert(top_name, value);
        return;
    }

    // For nested paths, traverse/create intermediate containers.
    set_nested_value(item, &top_name, &path.elements[1..], value, names);
}

/// Recursively set a nested value. Creates intermediate maps as needed.
fn set_nested_value(
    item: &mut HashMap<String, AttributeValue>,
    key: &str,
    remaining: &[PathElement],
    value: AttributeValue,
    names: &HashMap<String, String>,
) {
    if remaining.is_empty() {
        item.insert(key.to_owned(), value);
        return;
    }

    let entry = item
        .entry(key.to_owned())
        .or_insert_with(|| AttributeValue::M(HashMap::new()));

    match &remaining[0] {
        PathElement::Attribute(name) => {
            let resolved = if let Some(stripped) = name.strip_prefix('#') {
                names.get(stripped).cloned().unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            };
            if let AttributeValue::M(map) = entry {
                set_nested_value(map, &resolved, &remaining[1..], value, names);
            }
        }
        PathElement::Index(idx) => {
            if let AttributeValue::L(list) = entry {
                if *idx < list.len() {
                    if remaining.len() == 1 {
                        list[*idx] = value;
                    }
                    // Deeper indexing into lists of maps would require more complex logic;
                    // for the common case, we handle single-level list index assignment.
                }
            }
        }
    }
}

/// Remove an attribute at the given path from an item.
fn apply_remove(
    item: &mut HashMap<String, AttributeValue>,
    path: &AttributePath,
    names: &HashMap<String, String>,
) {
    if path.elements.is_empty() {
        return;
    }

    let top_name = match &path.elements[0] {
        PathElement::Attribute(name) => {
            if name.starts_with('#') {
                names
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            }
        }
        PathElement::Index(_) => return,
    };

    if path.elements.len() == 1 {
        item.remove(&top_name);
        return;
    }

    // For nested paths, traverse to the parent and remove.
    if let Some(AttributeValue::M(map)) = item.get_mut(&top_name) {
        let sub_path = AttributePath {
            elements: path.elements[1..].to_vec(),
        };
        apply_remove(map, &sub_path, names);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::parser::{parse_condition, parse_projection, parse_update};

    /// Helper to build a simple item with string attributes.
    fn make_item(pairs: &[(&str, AttributeValue)]) -> HashMap<String, AttributeValue> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect()
    }

    fn empty_names() -> HashMap<String, String> {
        HashMap::new()
    }

    fn make_values(pairs: &[(&str, AttributeValue)]) -> HashMap<String, AttributeValue> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect()
    }

    #[test]
    fn test_should_evaluate_equality() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = HashMap::from([("#n".to_owned(), "name".to_owned())]);
        let values = make_values(&[(":val", AttributeValue::S("Alice".to_owned()))]);

        let expr = parse_condition("#n = :val").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_inequality() {
        let item = make_item(&[("age", AttributeValue::N("30".to_owned()))]);
        let values = make_values(&[(":val", AttributeValue::N("25".to_owned()))]);
        let names = empty_names();

        let expr = parse_condition("age > :val").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_and() {
        let item = make_item(&[
            ("name", AttributeValue::S("Alice".to_owned())),
            ("age", AttributeValue::N("30".to_owned())),
        ]);
        let names = HashMap::from([
            ("#n".to_owned(), "name".to_owned()),
            ("#a".to_owned(), "age".to_owned()),
        ]);
        let values = make_values(&[
            (":name", AttributeValue::S("Alice".to_owned())),
            (":age", AttributeValue::N("30".to_owned())),
        ]);

        let expr = parse_condition("#n = :name AND #a = :age").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_or_short_circuit() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = HashMap::from([("#n".to_owned(), "name".to_owned())]);
        let values = make_values(&[
            (":v1", AttributeValue::S("Alice".to_owned())),
            (":v2", AttributeValue::S("Bob".to_owned())),
        ]);

        // First clause is true, so OR short-circuits.
        let expr = parse_condition("#n = :v1 OR #n = :v2").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_attribute_exists() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = HashMap::from([("#n".to_owned(), "name".to_owned())]);
        let values = HashMap::new();

        let expr = parse_condition("attribute_exists(#n)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());

        let expr = parse_condition("attribute_not_exists(#n)").unwrap();
        assert!(!ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_attribute_not_exists_for_missing() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = HashMap::from([("#x".to_owned(), "missing".to_owned())]);
        let values = HashMap::new();

        let expr = parse_condition("attribute_not_exists(#x)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_begins_with() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = HashMap::from([("#n".to_owned(), "name".to_owned())]);
        let values = make_values(&[(":prefix", AttributeValue::S("Ali".to_owned()))]);

        let expr = parse_condition("begins_with(#n, :prefix)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_contains_string() {
        let item = make_item(&[("bio", AttributeValue::S("Hello World".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[(":sub", AttributeValue::S("World".to_owned()))]);

        let expr = parse_condition("contains(bio, :sub)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_contains_set() {
        let item = make_item(&[(
            "tags",
            AttributeValue::Ss(vec!["rust".to_owned(), "aws".to_owned()]),
        )]);
        let names = empty_names();
        let values = make_values(&[(":tag", AttributeValue::S("rust".to_owned()))]);

        let expr = parse_condition("contains(tags, :tag)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_between() {
        let item = make_item(&[("age", AttributeValue::N("25".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[
            (":low", AttributeValue::N("20".to_owned())),
            (":high", AttributeValue::N("30".to_owned())),
        ]);

        let expr = parse_condition("age BETWEEN :low AND :high").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_in() {
        let item = make_item(&[("status", AttributeValue::S("active".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[
            (":v1", AttributeValue::S("active".to_owned())),
            (":v2", AttributeValue::S("pending".to_owned())),
        ]);

        let expr = parse_condition("status IN (:v1, :v2)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_not() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[(":val", AttributeValue::S("Bob".to_owned()))]);

        let expr = parse_condition("NOT name = :val").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_size_comparison() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[(":len", AttributeValue::N("3".to_owned()))]);

        let expr = parse_condition("size(name) > :len").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap()); // "Alice" has length 5 > 3
    }

    #[test]
    fn test_should_apply_set_update() {
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = HashMap::from([("#n".to_owned(), "name".to_owned())]);
        let values = make_values(&[(":val", AttributeValue::S("Bob".to_owned()))]);

        let update = parse_update("SET #n = :val").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert_eq!(
            result.get("name"),
            Some(&AttributeValue::S("Bob".to_owned()))
        );
    }

    #[test]
    fn test_should_apply_set_arithmetic() {
        let item = make_item(&[("count", AttributeValue::N("10".to_owned()))]);
        let names = HashMap::from([("#c".to_owned(), "count".to_owned())]);
        let values = make_values(&[(":inc", AttributeValue::N("5".to_owned()))]);

        let update = parse_update("SET #c = #c + :inc").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert_eq!(
            result.get("count"),
            Some(&AttributeValue::N("15".to_owned()))
        );
    }

    #[test]
    fn test_should_apply_set_if_not_exists() {
        let item = HashMap::new();
        let names = HashMap::from([("#n".to_owned(), "name".to_owned())]);
        let values = make_values(&[(":default", AttributeValue::S("Unknown".to_owned()))]);

        let update = parse_update("SET #n = if_not_exists(#n, :default)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert_eq!(
            result.get("name"),
            Some(&AttributeValue::S("Unknown".to_owned()))
        );
    }

    #[test]
    fn test_should_apply_set_list_append() {
        let item = make_item(&[(
            "items",
            AttributeValue::L(vec![AttributeValue::S("a".to_owned())]),
        )]);
        let names = empty_names();
        let values = make_values(&[(
            ":newItems",
            AttributeValue::L(vec![AttributeValue::S("b".to_owned())]),
        )]);

        let update = parse_update("SET items = list_append(items, :newItems)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        match result.get("items") {
            Some(AttributeValue::L(list)) => assert_eq!(list.len(), 2),
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn test_should_apply_remove_update() {
        let item = make_item(&[
            ("name", AttributeValue::S("Alice".to_owned())),
            ("age", AttributeValue::N("30".to_owned())),
        ]);
        let names = HashMap::from([("#a".to_owned(), "age".to_owned())]);
        let values = HashMap::new();

        let update = parse_update("REMOVE #a").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert!(!result.contains_key("age"));
        assert!(result.contains_key("name"));
    }

    #[test]
    fn test_should_apply_add_to_number() {
        let item = make_item(&[("count", AttributeValue::N("10".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[(":inc", AttributeValue::N("5".to_owned()))]);

        let update = parse_update("ADD count :inc").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert_eq!(
            result.get("count"),
            Some(&AttributeValue::N("15".to_owned()))
        );
    }

    #[test]
    fn test_should_apply_add_to_set() {
        let item = make_item(&[("tags", AttributeValue::Ss(vec!["rust".to_owned()]))]);
        let names = empty_names();
        let values = make_values(&[(":newTags", AttributeValue::Ss(vec!["aws".to_owned()]))]);

        let update = parse_update("ADD tags :newTags").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        match result.get("tags") {
            Some(AttributeValue::Ss(set)) => {
                assert!(set.contains(&"rust".to_owned()));
                assert!(set.contains(&"aws".to_owned()));
            }
            other => panic!("expected SS, got {other:?}"),
        }
    }

    #[test]
    fn test_should_apply_delete_from_set() {
        let item = make_item(&[(
            "tags",
            AttributeValue::Ss(vec!["rust".to_owned(), "aws".to_owned(), "go".to_owned()]),
        )]);
        let names = empty_names();
        let values = make_values(&[(":removeTags", AttributeValue::Ss(vec!["go".to_owned()]))]);

        let update = parse_update("DELETE tags :removeTags").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        match result.get("tags") {
            Some(AttributeValue::Ss(set)) => {
                assert!(set.contains(&"rust".to_owned()));
                assert!(set.contains(&"aws".to_owned()));
                assert!(!set.contains(&"go".to_owned()));
            }
            other => panic!("expected SS, got {other:?}"),
        }
    }

    #[test]
    fn test_should_apply_projection() {
        let item = make_item(&[
            ("id", AttributeValue::S("123".to_owned())),
            ("name", AttributeValue::S("Alice".to_owned())),
            ("age", AttributeValue::N("30".to_owned())),
        ]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("id, name").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("id"));
        assert!(result.contains_key("name"));
        assert!(!result.contains_key("age"));
    }

    #[test]
    fn test_should_apply_projection_with_nested_path() {
        let mut info = HashMap::new();
        info.insert("rating".to_owned(), AttributeValue::N("5".to_owned()));
        let item = make_item(&[
            ("id", AttributeValue::S("123".to_owned())),
            ("info", AttributeValue::M(info)),
        ]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("id, info.rating").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        assert!(result.contains_key("id"));
        assert!(result.contains_key("info"));
    }

    #[test]
    fn test_should_resolve_nested_path() {
        let mut info = HashMap::new();
        info.insert("rating".to_owned(), AttributeValue::N("5".to_owned()));
        let item = make_item(&[("info", AttributeValue::M(info))]);
        let names = empty_names();
        let values = HashMap::new();

        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };

        let path = AttributePath {
            elements: vec![
                PathElement::Attribute("info".to_owned()),
                PathElement::Attribute("rating".to_owned()),
            ],
        };
        let val = ctx.resolve_path(&path);
        assert_eq!(val, Some(&AttributeValue::N("5".to_owned())));
    }

    #[test]
    fn test_should_resolve_list_index_path() {
        let item = make_item(&[(
            "items",
            AttributeValue::L(vec![
                AttributeValue::S("first".to_owned()),
                AttributeValue::S("second".to_owned()),
            ]),
        )]);
        let names = empty_names();
        let values = HashMap::new();

        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };

        let path = AttributePath {
            elements: vec![
                PathElement::Attribute("items".to_owned()),
                PathElement::Index(1),
            ],
        };
        let val = ctx.resolve_path(&path);
        assert_eq!(val, Some(&AttributeValue::S("second".to_owned())));
    }

    #[test]
    fn test_should_return_false_for_missing_attribute_comparison() {
        let item = HashMap::new();
        let names = empty_names();
        let values = make_values(&[(":val", AttributeValue::S("test".to_owned()))]);

        let expr = parse_condition("name = :val").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(!ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_resolve_names_with_hash_prefix_keys() {
        // AWS SDK sends expression attribute names map with keys like "#n",
        // and the parser stores path elements as "#n". Verify that the
        // evaluator correctly looks up "#n" in the names map.
        let item = make_item(&[("myAttr", AttributeValue::S("hello".to_owned()))]);
        let names = HashMap::from([("#n".to_owned(), "myAttr".to_owned())]);
        let values = make_values(&[(":v", AttributeValue::S("hello".to_owned()))]);

        let expr = parse_condition("#n = :v").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_resolve_operand_value_with_colon_prefix() {
        // The parser stores Operand::Value("val") without the ":" prefix.
        // The evaluator's resolve_operand prepends ":" before looking up in
        // the values map, so values map keys must include ":".
        let item = make_item(&[("age", AttributeValue::N("25".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[(":threshold", AttributeValue::N("20".to_owned()))]);

        let expr = parse_condition("age > :threshold").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }
}
