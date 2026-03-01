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
            // DynamoDB behavior: missing attributes are never equal to anything
            // (not even to another missing attribute). For `<>` (Ne), a missing
            // attribute is always "not equal" to anything. For all other
            // operators, a missing attribute yields false.
            return Ok(matches!(op, CompareOp::Ne));
        };

        // For ordering operators, validate that query-constant operands have
        // types that support ordering. If a constant has an unsupported type
        // (e.g., list), this is a ValidationException. If the unsupported type
        // comes from an item attribute, comparison simply returns false.
        if is_ordering_op(op) {
            let op_name = op.to_string();
            validate_ordering_operand_type(left, lv, &op_name)?;
            validate_ordering_operand_type(right, rv, &op_name)?;
        }

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

        // BETWEEN uses ordering comparisons, so validate query-constant types.
        validate_ordering_operand_type(value, v, "BETWEEN")?;
        validate_ordering_operand_type(low, lo, "BETWEEN")?;
        validate_ordering_operand_type(high, hi, "BETWEEN")?;

        // If both bounds come from query constants, validate their types match
        // and the bounds are in the correct order (low <= high).
        if is_query_constant(low) && is_query_constant(high) {
            if std::mem::discriminant(lo) != std::mem::discriminant(hi) {
                return Err(ExpressionError::TypeMismatch {
                    message: "BETWEEN bounds must have the same type when both \
                              are expression attribute values"
                        .to_owned(),
                });
            }
            // Check ordering: if low > high, it is a ValidationException.
            if compare_values(lo, hi, CompareOp::Gt)? {
                return Err(ExpressionError::TypeMismatch {
                    message: "BETWEEN bounds are in wrong order; \
                              low bound must be less than or equal to high bound"
                        .to_owned(),
                });
            }
        }

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
                // The second argument must be an expression attribute value (`:name`),
                // not an item attribute path.
                let type_val = match &args[1] {
                    Operand::Value(_) => self.resolve_operand(&args[1])?,
                    _ => {
                        return Err(ExpressionError::InvalidOperand {
                            operation: "attribute_type".to_owned(),
                            message: "second argument must be an expression attribute value"
                                .to_owned(),
                        });
                    }
                };
                let Some(AttributeValue::S(expected_type)) = type_val else {
                    return Err(ExpressionError::TypeMismatch {
                        message: "attribute_type second argument must be a string".to_owned(),
                    });
                };

                // Validate the type string against known DynamoDB type descriptors.
                if !is_valid_type_descriptor(&expected_type) {
                    return Err(ExpressionError::TypeMismatch {
                        message: format!(
                            "Invalid type: {expected_type}. \
                             Valid types: S, SS, N, NS, B, BS, BOOL, NULL, L, M"
                        ),
                    });
                }

                // The first argument can be a path OR an expression attribute value.
                let attr_val = self.resolve_operand(&args[0])?;
                match attr_val {
                    Some(val) => Ok(val.type_descriptor() == expected_type),
                    None => Ok(false),
                }
            }
            FunctionName::BeginsWith => {
                let lval = self.resolve_operand(&args[0])?;
                let rval = self.resolve_operand(&args[1])?;

                // First pass: check query constants for type validation errors.
                // Query constants with unsupported types always produce
                // ValidationException, regardless of item attribute types.
                for (operand, resolved) in [(&args[0], &lval), (&args[1], &rval)] {
                    if let Some(v) = resolved {
                        if !matches!(v, AttributeValue::S(_) | AttributeValue::B(_))
                            && is_query_constant(operand)
                        {
                            return Err(ExpressionError::InvalidOperand {
                                operation: "begins_with".to_owned(),
                                message: format!(
                                    "Incorrect operand type for operator or function; \
                                     operator or function: begins_with, operand type: {td}",
                                    td = v.type_descriptor()
                                ),
                            });
                        }
                    }
                }

                let Some(attr) = lval else {
                    return Ok(false);
                };
                match (&attr, &rval) {
                    (AttributeValue::S(s), Some(AttributeValue::S(prefix))) => {
                        Ok(s.starts_with(prefix.as_str()))
                    }
                    (AttributeValue::B(b), Some(AttributeValue::B(prefix))) => {
                        Ok(b.starts_with(prefix.as_ref()))
                    }
                    _ => Ok(false),
                }
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
            (AttributeValue::B(b), AttributeValue::B(sub)) => {
                // Binary contains: check if b contains sub as a contiguous subsequence.
                Ok(b.windows(sub.len()).any(|window| window == sub.as_ref()))
            }
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
            Operand::Size(inner) => {
                let val = self.resolve_operand(inner)?;
                match val {
                    Some(v) => {
                        // size() is not defined for number, boolean, or null types.
                        // If the value comes from a query constant (expression attribute
                        // value), this is a ValidationException. If it comes from an
                        // item attribute, it simply returns None (failed condition).
                        let type_desc = v.type_descriptor();
                        if matches!(
                            &v,
                            AttributeValue::N(_)
                                | AttributeValue::Bool(_)
                                | AttributeValue::Null(_)
                        ) {
                            // Check if the inner operand is a query-provided value
                            // or a nested size() (which always produces a number).
                            if is_query_constant(inner) {
                                return Err(ExpressionError::InvalidOperand {
                                    operation: "size".to_owned(),
                                    message: format!(
                                        "Incorrect operand type for operator or function; \
                                         operator or function: size, operand type: {type_desc}"
                                    ),
                                });
                            }
                            // From item attribute: size() simply fails (evaluates to no match).
                            return Ok(None);
                        }
                        let size = attribute_size(&v);
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
                let av = self.resolve_set_value(a)?;
                let bv = self.resolve_set_value(b)?;
                numeric_arithmetic(&av, &bv, true)
            }
            SetValue::Minus(a, b) => {
                let av = self.resolve_set_value(a)?;
                let bv = self.resolve_set_value(b)?;
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

        // First, validate the operand type: ADD only supports numbers and sets.
        let operand_type = add_val.type_descriptor();
        if !matches!(
            &add_val,
            AttributeValue::N(_)
                | AttributeValue::Ss(_)
                | AttributeValue::Ns(_)
                | AttributeValue::Bs(_)
        ) {
            return Err(ExpressionError::InvalidOperand {
                operation: "ADD".to_owned(),
                message: format!(
                    "Incorrect operand type for operator or function; \
                     operator: ADD, operand type: {operand_type}"
                ),
            });
        }

        // For nested paths, resolve and modify in place.
        let existing = resolve_path_in_item(item, &action.path, self.names);
        let result = compute_add_result(&add_val, existing.as_ref())?;
        set_path_value(item, &action.path, result, self.names);

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

        // Validate that the operand is a set type.
        let operand_type = del_val.type_descriptor();
        if !matches!(
            &del_val,
            AttributeValue::Ss(_) | AttributeValue::Ns(_) | AttributeValue::Bs(_)
        ) {
            return Err(ExpressionError::InvalidOperand {
                operation: "DELETE".to_owned(),
                message: format!(
                    "Incorrect operand type for operator or function; \
                     operator: DELETE, operand type: {operand_type}"
                ),
            });
        }

        let attr_name = resolve_top_level_name(&action.path, self.names)?;
        let Some(existing) = item.get(&attr_name) else {
            // Deleting from a non-existent attribute is silently ignored.
            return Ok(());
        };

        match (&del_val, existing) {
            (AttributeValue::Ss(to_remove), AttributeValue::Ss(existing_set)) => {
                let filtered: Vec<String> = existing_set
                    .iter()
                    .filter(|s| !to_remove.contains(s))
                    .cloned()
                    .collect();
                if filtered.is_empty() {
                    item.remove(&attr_name);
                } else {
                    item.insert(attr_name, AttributeValue::Ss(filtered));
                }
            }
            (AttributeValue::Ns(to_remove), AttributeValue::Ns(existing_set)) => {
                let filtered: Vec<String> = existing_set
                    .iter()
                    .filter(|n| !to_remove.contains(n))
                    .cloned()
                    .collect();
                if filtered.is_empty() {
                    item.remove(&attr_name);
                } else {
                    item.insert(attr_name, AttributeValue::Ns(filtered));
                }
            }
            (AttributeValue::Bs(to_remove), AttributeValue::Bs(existing_set)) => {
                let filtered: Vec<bytes::Bytes> = existing_set
                    .iter()
                    .filter(|b| !to_remove.contains(b))
                    .cloned()
                    .collect();
                if filtered.is_empty() {
                    item.remove(&attr_name);
                } else {
                    item.insert(attr_name, AttributeValue::Bs(filtered));
                }
            }
            _ => {
                // Set type mismatch (e.g., deleting SS from NS).
                return Err(ExpressionError::InvalidOperand {
                    operation: "DELETE".to_owned(),
                    message: "An operand in the update expression has an incorrect data type"
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
    /// requested attributes. For nested paths, reconstructs the intermediate structure
    /// so that only the projected leaf values are included.
    #[must_use]
    pub fn apply_projection(&self, paths: &[AttributePath]) -> HashMap<String, AttributeValue> {
        let mut result = HashMap::new();
        for path in paths {
            if let Some(val) = self.resolve_path(path) {
                if let Some(top_name) = resolve_top_level_name_opt(path, self.names) {
                    if path.elements.len() == 1 {
                        result.insert(top_name, val.clone());
                    } else {
                        // For nested paths, reconstruct the nested structure.
                        deep_insert(&mut result, path, val.clone(), self.names);
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

/// Returns `true` if the operand originates from a query constant (expression
/// attribute value like `:val`) or a nested `size()` call (which always produces
/// a number), rather than from an item attribute.
fn is_query_constant(operand: &Operand) -> bool {
    match operand {
        Operand::Value(_) | Operand::Size(_) => true,
        Operand::Path(_) => false,
    }
}

/// Returns `true` if the given attribute value type does not support ordering
/// comparisons (`<`, `<=`, `>`, `>=`).
fn is_ordering_unsupported_type(val: &AttributeValue) -> bool {
    matches!(
        val,
        AttributeValue::L(_)
            | AttributeValue::M(_)
            | AttributeValue::Ss(_)
            | AttributeValue::Ns(_)
            | AttributeValue::Bs(_)
            | AttributeValue::Null(_)
    )
}

/// Returns `true` if the comparison operator is an ordering comparison.
fn is_ordering_op(op: CompareOp) -> bool {
    matches!(
        op,
        CompareOp::Lt | CompareOp::Le | CompareOp::Gt | CompareOp::Ge
    )
}

/// Validate that a query-constant operand's type is compatible with ordering
/// operators. Returns an error if the operand is a query constant with a type
/// that does not support ordering.
fn validate_ordering_operand_type(
    operand: &Operand,
    resolved_value: &AttributeValue,
    operator_name: &str,
) -> Result<(), ExpressionError> {
    if is_query_constant(operand) && is_ordering_unsupported_type(resolved_value) {
        return Err(ExpressionError::InvalidOperand {
            operation: "operator".to_owned(),
            message: format!(
                "Incorrect operand type for operator or function; \
                 operator: {operator_name}, operand type: {type_desc}",
                type_desc = resolved_value.type_descriptor()
            ),
        });
    }
    Ok(())
}

/// Compare two `AttributeValue`s using the given comparison operator.
///
/// For equality (`=`/`<>`) all DynamoDB types are supported, including
/// sets (order-independent), lists, and maps (deep equality).
/// For ordering (`<`, `<=`, `>`, `>=`) only string, number, and binary
/// types are comparable.
fn compare_values(
    left: &AttributeValue,
    right: &AttributeValue,
    op: CompareOp,
) -> Result<bool, ExpressionError> {
    match (left, right) {
        // Ordered types: support all six comparison operators.
        (AttributeValue::S(a), AttributeValue::S(b)) => Ok(compare_ord(a, b, op)),
        (AttributeValue::N(a), AttributeValue::N(b)) => {
            let fa = parse_number(a)?;
            let fb = parse_number(b)?;
            Ok(compare_f64(fa, fb, op))
        }
        (AttributeValue::B(a), AttributeValue::B(b)) => Ok(compare_ord(a, b, op)),

        // Bool: only equality/inequality (ordering between true/false is allowed by DynamoDB).
        (AttributeValue::Bool(a), AttributeValue::Bool(b)) => Ok(compare_ord(a, b, op)),

        // Null: only equality/inequality.
        (AttributeValue::Null(true), AttributeValue::Null(true)) => {
            Ok(matches!(op, CompareOp::Eq | CompareOp::Le | CompareOp::Ge))
        }

        // Sets: equality only, order-independent comparison.
        (AttributeValue::Ss(a), AttributeValue::Ss(b))
        | (AttributeValue::Ns(a), AttributeValue::Ns(b)) => {
            let eq = sets_equal(a, b);
            Ok(apply_equality_op(eq, op))
        }
        (AttributeValue::Bs(a), AttributeValue::Bs(b)) => {
            let eq = a.len() == b.len() && a.iter().all(|item| b.contains(item));
            Ok(apply_equality_op(eq, op))
        }

        // List: deep equality, order matters.
        (AttributeValue::L(a), AttributeValue::L(b)) => {
            let eq = deep_equal_lists(a, b);
            Ok(apply_equality_op(eq, op))
        }

        // Map: deep equality, key order does not matter.
        (AttributeValue::M(a), AttributeValue::M(b)) => {
            let eq = deep_equal_maps(a, b);
            Ok(apply_equality_op(eq, op))
        }

        // Different types: not comparable. Equality is false, Ne is true.
        _ => Ok(matches!(op, CompareOp::Ne)),
    }
}

/// For types that only support equality comparison, apply the given operator
/// against the equality result.
fn apply_equality_op(eq: bool, op: CompareOp) -> bool {
    match op {
        CompareOp::Eq => eq,
        CompareOp::Ne => !eq,
        // Sets, lists, and maps do not support ordering; treat as false.
        CompareOp::Lt | CompareOp::Le | CompareOp::Gt | CompareOp::Ge => {
            if eq {
                // Equal values satisfy `<=` and `>=`.
                matches!(op, CompareOp::Le | CompareOp::Ge)
            } else {
                false
            }
        }
    }
}

/// Order-independent equality for string/number sets.
fn sets_equal<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len() && a.iter().all(|item| b.contains(item))
}

/// Deep equality comparison for lists (order matters).
fn deep_equal_lists(a: &[AttributeValue], b: &[AttributeValue]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(av, bv)| deep_equal_values(av, bv))
}

/// Deep equality comparison for maps (key order does not matter).
fn deep_equal_maps(
    a: &HashMap<String, AttributeValue>,
    b: &HashMap<String, AttributeValue>,
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .all(|(k, v)| b.get(k).is_some_and(|bv| deep_equal_values(v, bv)))
}

/// Deep recursive equality for any two `AttributeValue` instances.
///
/// This handles sets with order-independent comparison, which is
/// critical for DynamoDB condition expression evaluation.
fn deep_equal_values(a: &AttributeValue, b: &AttributeValue) -> bool {
    match (a, b) {
        (AttributeValue::S(a), AttributeValue::S(b)) => a == b,
        (AttributeValue::N(a), AttributeValue::N(b)) => {
            // Compare numerically to handle "1" == "1.0" etc.
            let Ok(fa) = a.parse::<f64>() else {
                return a == b;
            };
            let Ok(fb) = b.parse::<f64>() else {
                return a == b;
            };
            (fa - fb).abs() < f64::EPSILON
        }
        (AttributeValue::B(a), AttributeValue::B(b)) => a == b,
        (AttributeValue::Bool(a), AttributeValue::Bool(b))
        | (AttributeValue::Null(a), AttributeValue::Null(b)) => a == b,
        (AttributeValue::Ss(a), AttributeValue::Ss(b))
        | (AttributeValue::Ns(a), AttributeValue::Ns(b)) => sets_equal(a, b),
        (AttributeValue::Bs(a), AttributeValue::Bs(b)) => {
            a.len() == b.len() && a.iter().all(|item| b.contains(item))
        }
        (AttributeValue::L(a), AttributeValue::L(b)) => deep_equal_lists(a, b),
        (AttributeValue::M(a), AttributeValue::M(b)) => deep_equal_maps(a, b),
        _ => false,
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

/// Returns `true` if the given string is a valid DynamoDB type descriptor.
fn is_valid_type_descriptor(s: &str) -> bool {
    matches!(
        s,
        "S" | "SS" | "N" | "NS" | "B" | "BS" | "BOOL" | "NULL" | "L" | "M"
    )
}

/// Insert a value at a nested path within a result map, creating intermediate structures.
fn deep_insert(
    result: &mut HashMap<String, AttributeValue>,
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
        result.insert(top_name, value);
        return;
    }

    // For nested paths, build the intermediate structure.
    let entry = result
        .entry(top_name)
        .or_insert_with(|| AttributeValue::M(HashMap::new()));
    if let AttributeValue::M(map) = entry {
        let sub_path = AttributePath {
            elements: path.elements[1..].to_vec(),
        };
        deep_insert(map, &sub_path, value, names);
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

/// Resolve a name reference, handling `#name` substitution.
fn resolve_name_ref(name: &str, names: &HashMap<String, String>) -> String {
    if name.starts_with('#') {
        names.get(name).cloned().unwrap_or_else(|| name.to_owned())
    } else {
        name.to_owned()
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

    let PathElement::Attribute(top) = &path.elements[0] else {
        return;
    };
    let top_name = resolve_name_ref(top, names);

    if path.elements.len() == 1 {
        item.insert(top_name, value);
        return;
    }

    // For nested paths, traverse into the value recursively.
    set_value_at_path(item.entry(top_name), &path.elements[1..], value, names);
}

/// Set a value at a subpath starting from an entry in a map.
fn set_value_at_path(
    entry: std::collections::hash_map::Entry<'_, String, AttributeValue>,
    remaining: &[PathElement],
    value: AttributeValue,
    names: &HashMap<String, String>,
) {
    let container = entry.or_insert_with(|| AttributeValue::M(HashMap::new()));
    set_value_in_container(container, remaining, value, names);
}

/// Set a value at a subpath starting from an `AttributeValue` container (map or list).
fn set_value_in_container(
    container: &mut AttributeValue,
    remaining: &[PathElement],
    value: AttributeValue,
    names: &HashMap<String, String>,
) {
    if remaining.is_empty() {
        return;
    }

    match &remaining[0] {
        PathElement::Attribute(name) => {
            let resolved = resolve_name_ref(name, names);
            let AttributeValue::M(map) = container else {
                return;
            };
            if remaining.len() == 1 {
                map.insert(resolved, value);
            } else {
                let child = map
                    .entry(resolved)
                    .or_insert_with(|| AttributeValue::M(HashMap::new()));
                set_value_in_container(child, &remaining[1..], value, names);
            }
        }
        PathElement::Index(idx) => {
            let AttributeValue::L(list) = container else {
                return;
            };
            // DynamoDB behavior: extend list with NULLs if index is beyond bounds.
            while list.len() <= *idx {
                list.push(AttributeValue::Null(true));
            }
            if remaining.len() == 1 {
                list[*idx] = value;
            } else {
                set_value_in_container(&mut list[*idx], &remaining[1..], value, names);
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

    let PathElement::Attribute(top) = &path.elements[0] else {
        return;
    };
    let top_name = resolve_name_ref(top, names);

    if path.elements.len() == 1 {
        item.remove(&top_name);
        return;
    }

    // For nested paths, traverse to the parent container and remove.
    if let Some(container) = item.get_mut(&top_name) {
        remove_at_path(container, &path.elements[1..], names);
    }
}

/// Remove a value at a subpath within a container (map or list).
fn remove_at_path(
    container: &mut AttributeValue,
    remaining: &[PathElement],
    names: &HashMap<String, String>,
) {
    if remaining.is_empty() {
        return;
    }

    // Base case: we're at the last path element, perform the actual removal.
    if remaining.len() == 1 {
        match &remaining[0] {
            PathElement::Attribute(name) => {
                let resolved = resolve_name_ref(name, names);
                if let AttributeValue::M(map) = container {
                    map.remove(&resolved);
                }
            }
            PathElement::Index(idx) => {
                if let AttributeValue::L(list) = container {
                    if *idx < list.len() {
                        list.remove(*idx);
                    }
                }
            }
        }
        return;
    }

    // Recursive case: traverse deeper.
    match &remaining[0] {
        PathElement::Attribute(name) => {
            let resolved = resolve_name_ref(name, names);
            if let AttributeValue::M(map) = container {
                if let Some(child) = map.get_mut(&resolved) {
                    remove_at_path(child, &remaining[1..], names);
                }
            }
        }
        PathElement::Index(idx) => {
            if let AttributeValue::L(list) = container {
                if let Some(child) = list.get_mut(*idx) {
                    remove_at_path(child, &remaining[1..], names);
                }
            }
        }
    }
}

/// Resolve a path within an item, returning a clone of the value at that path.
///
/// This is similar to `EvalContext::resolve_path` but operates on a mutable item
/// reference without requiring an `EvalContext`.
fn resolve_path_in_item(
    item: &HashMap<String, AttributeValue>,
    path: &AttributePath,
    names: &HashMap<String, String>,
) -> Option<AttributeValue> {
    let mut current: Option<&AttributeValue> = None;

    for (i, element) in path.elements.iter().enumerate() {
        match element {
            PathElement::Attribute(name) => {
                let resolved_name = if name.starts_with('#') {
                    names.get(name.as_str())?
                } else {
                    name
                };
                if i == 0 {
                    current = item.get(resolved_name);
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

    current.cloned()
}

/// Compute the result of an ADD operation given the add value and existing value.
fn compute_add_result(
    add_val: &AttributeValue,
    existing: Option<&AttributeValue>,
) -> Result<AttributeValue, ExpressionError> {
    match (add_val, existing) {
        // ADD to a number: increment
        (AttributeValue::N(_), Some(existing_val @ AttributeValue::N(_))) => {
            numeric_arithmetic(existing_val, add_val, true)
        }
        // ADD to a string set
        (AttributeValue::Ss(new_items), Some(AttributeValue::Ss(existing_set))) => {
            let mut merged = existing_set.clone();
            for s in new_items {
                if !merged.contains(s) {
                    merged.push(s.clone());
                }
            }
            Ok(AttributeValue::Ss(merged))
        }
        // ADD to a number set
        (AttributeValue::Ns(new_items), Some(AttributeValue::Ns(existing_set))) => {
            let mut merged = existing_set.clone();
            for n in new_items {
                if !merged.contains(n) {
                    merged.push(n.clone());
                }
            }
            Ok(AttributeValue::Ns(merged))
        }
        // ADD to a binary set
        (AttributeValue::Bs(new_items), Some(AttributeValue::Bs(existing_set))) => {
            let mut merged = existing_set.clone();
            for b in new_items {
                if !merged.contains(b) {
                    merged.push(b.clone());
                }
            }
            Ok(AttributeValue::Bs(merged))
        }
        // ADD a number or set to non-existing attribute: set it directly
        (_, None) => Ok(add_val.clone()),
        // ADD a number/set to a mismatched existing attribute type
        (_, Some(_)) => Err(ExpressionError::InvalidOperand {
            operation: "ADD".to_owned(),
            message: "An operand in the update expression has an incorrect data type".to_owned(),
        }),
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

    // -----------------------------------------------------------------------
    // 3B: begins_with binary support
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_evaluate_begins_with_binary() {
        let item = make_item(&[(
            "data",
            AttributeValue::B(bytes::Bytes::from_static(b"\x01\x02\x03\x04")),
        )]);
        let names = empty_names();
        let values = make_values(&[(
            ":prefix",
            AttributeValue::B(bytes::Bytes::from_static(b"\x01\x02")),
        )]);

        let expr = parse_condition("begins_with(data, :prefix)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_begins_with_binary_no_match() {
        let item = make_item(&[(
            "data",
            AttributeValue::B(bytes::Bytes::from_static(b"\x01\x02\x03")),
        )]);
        let names = empty_names();
        let values = make_values(&[(
            ":prefix",
            AttributeValue::B(bytes::Bytes::from_static(b"\x02\x03")),
        )]);

        let expr = parse_condition("begins_with(data, :prefix)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(!ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_begins_with_type_mismatch_returns_false() {
        // String attribute with binary prefix should return false.
        let item = make_item(&[("name", AttributeValue::S("Alice".to_owned()))]);
        let names = empty_names();
        let values = make_values(&[(
            ":prefix",
            AttributeValue::B(bytes::Bytes::from_static(b"\x01")),
        )]);

        let expr = parse_condition("begins_with(name, :prefix)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(!ctx.evaluate(&expr).unwrap());
    }

    // -----------------------------------------------------------------------
    // 3C: contains binary support
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_evaluate_contains_binary() {
        let item = make_item(&[(
            "data",
            AttributeValue::B(bytes::Bytes::from_static(b"\x01\x02\x03\x04")),
        )]);
        let names = empty_names();
        let values = make_values(&[(
            ":sub",
            AttributeValue::B(bytes::Bytes::from_static(b"\x02\x03")),
        )]);

        let expr = parse_condition("contains(data, :sub)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(ctx.evaluate(&expr).unwrap());
    }

    #[test]
    fn test_should_evaluate_contains_binary_no_match() {
        let item = make_item(&[(
            "data",
            AttributeValue::B(bytes::Bytes::from_static(b"\x01\x02\x03")),
        )]);
        let names = empty_names();
        let values = make_values(&[(
            ":sub",
            AttributeValue::B(bytes::Bytes::from_static(b"\x04\x05")),
        )]);

        let expr = parse_condition("contains(data, :sub)").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        assert!(!ctx.evaluate(&expr).unwrap());
    }

    // -----------------------------------------------------------------------
    // 3E: DELETE on sets should remove empty sets
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_remove_attribute_when_delete_empties_string_set() {
        let item = make_item(&[("tags", AttributeValue::Ss(vec!["only".to_owned()]))]);
        let names = empty_names();
        let values = make_values(&[(":removeTags", AttributeValue::Ss(vec!["only".to_owned()]))]);

        let update = parse_update("DELETE tags :removeTags").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        // The attribute should be removed entirely, not left as an empty set.
        assert!(!result.contains_key("tags"));
    }

    #[test]
    fn test_should_remove_attribute_when_delete_empties_number_set() {
        let item = make_item(&[(
            "nums",
            AttributeValue::Ns(vec!["1".to_owned(), "2".to_owned()]),
        )]);
        let names = empty_names();
        let values = make_values(&[(
            ":removeNums",
            AttributeValue::Ns(vec!["1".to_owned(), "2".to_owned()]),
        )]);

        let update = parse_update("DELETE nums :removeNums").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert!(!result.contains_key("nums"));
    }

    #[test]
    fn test_should_remove_attribute_when_delete_empties_binary_set() {
        let item = make_item(&[(
            "bins",
            AttributeValue::Bs(vec![bytes::Bytes::from_static(b"\x01")]),
        )]);
        let names = empty_names();
        let values = make_values(&[(
            ":removeBins",
            AttributeValue::Bs(vec![bytes::Bytes::from_static(b"\x01")]),
        )]);

        let update = parse_update("DELETE bins :removeBins").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert!(!result.contains_key("bins"));
    }

    // -----------------------------------------------------------------------
    // 3F: ADD on missing number attributes
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_add_number_to_missing_attribute() {
        let item = HashMap::new();
        let names = empty_names();
        let values = make_values(&[(":inc", AttributeValue::N("5".to_owned()))]);

        let update = parse_update("ADD counter :inc").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert_eq!(
            result.get("counter"),
            Some(&AttributeValue::N("5".to_owned()))
        );
    }

    // -----------------------------------------------------------------------
    // 3A: Nested path SET improvements (list index beyond bounds)
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_extend_list_with_nulls_for_out_of_bounds_set() {
        let item = make_item(&[(
            "items",
            AttributeValue::L(vec![AttributeValue::S("a".to_owned())]),
        )]);
        let names = empty_names();
        let values = make_values(&[(":val", AttributeValue::S("c".to_owned()))]);

        let update = parse_update("SET items[3] = :val").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        match result.get("items") {
            Some(AttributeValue::L(list)) => {
                assert_eq!(list.len(), 4);
                assert_eq!(list[0], AttributeValue::S("a".to_owned()));
                assert_eq!(list[1], AttributeValue::Null(true));
                assert_eq!(list[2], AttributeValue::Null(true));
                assert_eq!(list[3], AttributeValue::S("c".to_owned()));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // 3D: Projection expression for nested paths
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_project_nested_path_with_only_requested_fields() {
        let mut info = HashMap::new();
        info.insert("rating".to_owned(), AttributeValue::N("5".to_owned()));
        info.insert("genre".to_owned(), AttributeValue::S("action".to_owned()));
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
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("id"));
        // The info map should only contain the "rating" field, not "genre".
        match result.get("info") {
            Some(AttributeValue::M(map)) => {
                assert!(map.contains_key("rating"));
                assert!(!map.contains_key("genre"));
            }
            other => panic!("expected M with only rating, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // 3B (parser): if_not_exists + arithmetic compound expression
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_evaluate_if_not_exists_plus_value() {
        let item = HashMap::new();
        let names = HashMap::from([("#c".to_owned(), "counter".to_owned())]);
        let values = make_values(&[
            (":zero", AttributeValue::N("0".to_owned())),
            (":one", AttributeValue::N("1".to_owned())),
        ]);

        let update = parse_update("SET #c = if_not_exists(#c, :zero) + :one").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert_eq!(
            result.get("counter"),
            Some(&AttributeValue::N("1".to_owned()))
        );
    }

    #[test]
    fn test_should_evaluate_if_not_exists_plus_on_existing_value() {
        let item = make_item(&[("counter", AttributeValue::N("10".to_owned()))]);
        let names = HashMap::from([("#c".to_owned(), "counter".to_owned())]);
        let values = make_values(&[
            (":zero", AttributeValue::N("0".to_owned())),
            (":one", AttributeValue::N("1".to_owned())),
        ]);

        let update = parse_update("SET #c = if_not_exists(#c, :zero) + :one").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        assert_eq!(
            result.get("counter"),
            Some(&AttributeValue::N("11".to_owned()))
        );
    }
}
