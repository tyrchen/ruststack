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

        // Sort SET actions by their target list index so that out-of-bounds
        // appends happen in index order, matching DynamoDB behavior.
        let mut sorted_set_actions: Vec<&SetAction> = update.set_actions.iter().collect();
        sorted_set_actions.sort_by_key(|action| extract_last_index(&action.path));

        for action in &sorted_set_actions {
            self.apply_set_action(&mut result, action)?;
        }
        for path in &update.remove_paths {
            apply_remove(&mut result, path, self.names)?;
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
        set_path_value(item, &action.path, value, self.names)?;
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
        set_path_value(item, &action.path, result, self.names)?;

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
                let del_type = del_val.type_descriptor();
                let existing_type = existing.type_descriptor();
                return Err(ExpressionError::InvalidOperand {
                    operation: "DELETE".to_owned(),
                    message: format!(
                        "Type mismatch for DELETE; operator type: {del_type}, \
                         existing type: {existing_type}"
                    ),
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
    let result = precise_arithmetic(na, nb, is_add)?;
    Ok(AttributeValue::N(result))
}

/// Parsed number representation: sign, significant digits, and exponent.
/// The number represents: sign * digits * 10^exponent
/// e.g. "123.45" → (false, "12345", -2), "-5e3" → (true, "5", 3)
struct ParsedNumber {
    negative: bool,
    /// Significant digits as a string (no leading zeros, no trailing zeros).
    digits: String,
    /// Exponent after shifting all digits to integers.
    exponent: i64,
}

/// Parse a number string into its components.
fn parse_number_components(s: &str) -> Result<ParsedNumber, ExpressionError> {
    let trimmed = s.trim();
    let (negative, rest) = if let Some(r) = trimmed.strip_prefix('-') {
        (true, r)
    } else if let Some(r) = trimmed.strip_prefix('+') {
        (false, r)
    } else {
        (false, trimmed)
    };

    let (mantissa, explicit_exp) = if let Some(pos) = rest.find(['e', 'E']) {
        let exp: i64 = rest[pos + 1..]
            .parse()
            .map_err(|_| ExpressionError::TypeMismatch {
                message: "invalid number format".to_owned(),
            })?;
        (&rest[..pos], exp)
    } else {
        (rest, 0i64)
    };

    // Remove the decimal point and adjust exponent.
    let (all_digits, frac_len) = if let Some(dot_pos) = mantissa.find('.') {
        let integer_part = &mantissa[..dot_pos];
        let frac_part = &mantissa[dot_pos + 1..];
        let all = format!("{integer_part}{frac_part}");
        #[allow(clippy::cast_possible_wrap)]
        let frac = frac_part.len() as i64;
        (all, frac)
    } else {
        (mantissa.to_owned(), 0i64)
    };

    // Remove leading zeros.
    let trimmed_digits = all_digits.trim_start_matches('0').to_owned();
    if trimmed_digits.is_empty() {
        return Ok(ParsedNumber {
            negative: false,
            digits: "0".to_owned(),
            exponent: 0,
        });
    }

    // Remove trailing zeros and adjust exponent.
    let trailing_zeros = trimmed_digits.len() - trimmed_digits.trim_end_matches('0').len();
    let final_digits = trimmed_digits.trim_end_matches('0').to_owned();
    #[allow(clippy::cast_possible_wrap)]
    let exponent = explicit_exp - frac_len + trailing_zeros as i64;

    Ok(ParsedNumber {
        negative,
        digits: final_digits,
        exponent,
    })
}

/// Perform precise arithmetic on two DynamoDB number strings.
/// Returns the result as a canonical DynamoDB number string.
fn precise_arithmetic(a: &str, b: &str, is_add: bool) -> Result<String, ExpressionError> {
    let mut pa = parse_number_components(a)?;
    let mut pb = parse_number_components(b)?;

    // For subtraction, flip the sign of b.
    if !is_add {
        pb.negative = !pb.negative;
    }

    // If either is zero, return the other.
    if pa.digits == "0" {
        return Ok(format_parsed_number(&pb));
    }
    if pb.digits == "0" {
        return Ok(format_parsed_number(&pa));
    }

    // Align exponents by padding the higher-exponent number with trailing zeros.
    let min_exp = pa.exponent.min(pb.exponent);
    // Shifts are always non-negative since min_exp <= both exponents.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let a_shift = (pa.exponent - min_exp) as usize;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let b_shift = (pb.exponent - min_exp) as usize;
    pa.digits.push_str(&"0".repeat(a_shift));
    pb.digits.push_str(&"0".repeat(b_shift));
    pa.exponent = min_exp;
    pb.exponent = min_exp;

    // Check total digit count — if alignment requires too many digits, precision is lost.
    let max_digits = pa.digits.len().max(pb.digits.len());
    if max_digits > 38 + 2 {
        // Precision would be lost in the result.
        return Err(ExpressionError::Validation {
            message: "Number overflow. Attempting to store a number with magnitude larger than supported range".to_owned(),
        });
    }

    // Convert to i128 for arithmetic (supports up to 38 digits).
    let a_val: i128 = pa
        .digits
        .parse()
        .map_err(|_| ExpressionError::TypeMismatch {
            message: "number too large for arithmetic".to_owned(),
        })?;
    let b_val: i128 = pb
        .digits
        .parse()
        .map_err(|_| ExpressionError::TypeMismatch {
            message: "number too large for arithmetic".to_owned(),
        })?;

    let a_signed = if pa.negative { -a_val } else { a_val };
    let b_signed = if pb.negative { -b_val } else { b_val };
    let result = a_signed + b_signed;

    let result_negative = result < 0;
    let result_abs = result.unsigned_abs();
    let result_str = result_abs.to_string();

    // Remove trailing zeros and adjust exponent.
    let trimmed = result_str.trim_end_matches('0');
    let trailing = result_str.len() - trimmed.len();
    #[allow(clippy::cast_possible_wrap)]
    let final_exp = min_exp + trailing as i64;

    if trimmed.is_empty() || trimmed == "0" {
        return Ok("0".to_owned());
    }

    // Validate result precision and magnitude.
    if trimmed.len() > 38 {
        return Err(ExpressionError::Validation {
            message: "Number overflow. Attempting to store a number with magnitude larger than supported range".to_owned(),
        });
    }

    #[allow(clippy::cast_possible_wrap)]
    let magnitude = final_exp + trimmed.len() as i64 - 1;
    if magnitude > 125 {
        return Err(ExpressionError::Validation {
            message: "Number overflow. Attempting to store a number with magnitude larger than supported range".to_owned(),
        });
    }
    if magnitude < -130 {
        return Err(ExpressionError::Validation {
            message: "Number underflow. Attempting to store a number with magnitude smaller than supported range".to_owned(),
        });
    }

    let parsed = ParsedNumber {
        negative: result_negative,
        digits: trimmed.to_owned(),
        exponent: final_exp,
    };
    Ok(format_parsed_number(&parsed))
}

/// Format a `ParsedNumber` into a canonical DynamoDB number string.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn format_parsed_number(p: &ParsedNumber) -> String {
    if p.digits == "0" {
        return "0".to_owned();
    }

    let sign = if p.negative { "-" } else { "" };
    let digits = &p.digits;
    let exp = p.exponent;

    // The number is: digits * 10^exp
    // We want to produce the most natural representation.
    let total_integer_digits = digits.len() as i64 + exp;

    if exp >= 0 && total_integer_digits <= 20 {
        // Pure integer: append zeros.
        return format!("{sign}{digits}{}", "0".repeat(exp as usize));
    }

    if total_integer_digits > 0 && total_integer_digits <= 20 {
        // Mixed: some integer digits, some fractional.
        let int_part = &digits[..total_integer_digits as usize];
        let frac_part = &digits[total_integer_digits as usize..];
        return format!("{sign}{int_part}.{frac_part}");
    }

    if ((-6)..=0).contains(&total_integer_digits) {
        // Small decimal: 0.000...digits
        let leading_zeros = (-total_integer_digits) as usize;
        return format!("{sign}0.{}{digits}", "0".repeat(leading_zeros));
    }

    // Scientific notation for very large or very small numbers.
    if digits.len() == 1 {
        let sci_exp = exp + digits.len() as i64 - 1;
        format!("{sign}{digits}E{sci_exp}")
    } else {
        let int_part = &digits[..1];
        let frac_part = &digits[1..];
        let sci_exp = exp + digits.len() as i64 - 1;
        format!("{sign}{int_part}.{frac_part}E{sci_exp}")
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
/// Resolved path element for deep insertion.
enum ResolvedElement {
    /// A map key access.
    MapKey(String),
    /// A list index access (value not needed since projection appends densely).
    ListIndex,
}

/// Insert a value at the given path into a result map, creating intermediate
/// maps and lists as needed. Handles nested map and list structures, merging
/// values with shared prefixes. For list indices, creates sparse lists (filled
/// with `Null(true)` placeholders) large enough to hold the target index.
fn deep_insert(
    result: &mut HashMap<String, AttributeValue>,
    path: &AttributePath,
    value: AttributeValue,
    names: &HashMap<String, String>,
) {
    if path.elements.is_empty() {
        return;
    }

    // Build a resolved elements list (replacing `#name` references).
    let resolved: Vec<ResolvedElement> = path
        .elements
        .iter()
        .map(|e| match e {
            PathElement::Attribute(name) => {
                let resolved_name = if name.starts_with('#') {
                    names
                        .get(name.as_str())
                        .cloned()
                        .unwrap_or_else(|| name.clone())
                } else {
                    name.clone()
                };
                ResolvedElement::MapKey(resolved_name)
            }
            PathElement::Index(_) => ResolvedElement::ListIndex,
        })
        .collect();

    // The first element must be a map key (top-level attribute name).
    let ResolvedElement::MapKey(ref top_name) = resolved[0] else {
        return;
    };

    if resolved.len() == 1 {
        result.insert(top_name.clone(), value);
        return;
    }

    // For deeper paths, navigate/create intermediate containers.
    let entry = result
        .entry(top_name.clone())
        .or_insert_with(|| match &resolved[1] {
            ResolvedElement::MapKey(_) => AttributeValue::M(HashMap::new()),
            ResolvedElement::ListIndex => AttributeValue::L(Vec::new()),
        });

    deep_insert_into_av(entry, &resolved[1..], value);
}

/// Recursively insert a value into an `AttributeValue` at the given resolved path.
fn deep_insert_into_av(
    target: &mut AttributeValue,
    resolved: &[ResolvedElement],
    value: AttributeValue,
) {
    if resolved.is_empty() {
        return;
    }

    if resolved.len() == 1 {
        // Base case: set the value.
        match &resolved[0] {
            ResolvedElement::MapKey(key) => {
                if let AttributeValue::M(map) = target {
                    map.insert(key.clone(), value);
                }
            }
            ResolvedElement::ListIndex => {
                if let AttributeValue::L(list) = target {
                    // For projection, we append values without null-padding.
                    // Each projected index just gets appended to the result list.
                    list.push(value);
                }
            }
        }
        return;
    }

    // Recursive case: navigate into the next level.
    match &resolved[0] {
        ResolvedElement::MapKey(key) => {
            if let AttributeValue::M(map) = target {
                // Determine what the next element expects.
                let child = map
                    .entry(key.clone())
                    .or_insert_with(|| match &resolved[1] {
                        ResolvedElement::MapKey(_) => AttributeValue::M(HashMap::new()),
                        ResolvedElement::ListIndex => AttributeValue::L(Vec::new()),
                    });
                deep_insert_into_av(child, &resolved[1..], value);
            }
        }
        ResolvedElement::ListIndex => {
            if let AttributeValue::L(list) = target {
                // For projection, find or create a child entry.
                // We append a new entry if the list is empty or create one.
                if list.is_empty() {
                    let child = match &resolved[1] {
                        ResolvedElement::MapKey(_) => AttributeValue::M(HashMap::new()),
                        ResolvedElement::ListIndex => AttributeValue::L(Vec::new()),
                    };
                    list.push(child);
                }
                let child = list.last_mut().expect("just pushed");
                deep_insert_into_av(child, &resolved[1..], value);
            }
        }
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

/// Extract the last `PathElement::Index` value from a path for sorting.
///
/// Returns `usize::MAX` for paths that do not end with an index, so they
/// sort after all indexed paths.
fn extract_last_index(path: &AttributePath) -> usize {
    match path.elements.last() {
        Some(PathElement::Index(idx)) => *idx,
        _ => usize::MAX,
    }
}

/// Set a value at the given path in an item. For top-level paths, this inserts
/// directly into the map. For nested paths, it traverses/creates intermediate maps.
///
/// # Errors
///
/// Returns `ExpressionError` if a nested path encounters a type that does not
/// support the path traversal (e.g., `.field` on a non-map, `[idx]` on a non-list).
fn set_path_value(
    item: &mut HashMap<String, AttributeValue>,
    path: &AttributePath,
    value: AttributeValue,
    names: &HashMap<String, String>,
) -> Result<(), ExpressionError> {
    if path.elements.is_empty() {
        return Ok(());
    }

    let PathElement::Attribute(top) = &path.elements[0] else {
        return Ok(());
    };
    let top_name = resolve_name_ref(top, names);

    if path.elements.len() == 1 {
        item.insert(top_name, value);
        return Ok(());
    }

    // For nested paths, validate that the existing value supports traversal
    // before creating any intermediate structures.
    validate_nested_path_for_set(item, &path.elements, names)?;

    // For nested paths, traverse into the value recursively.
    set_value_at_path(item.entry(top_name), &path.elements[1..], value, names);
    Ok(())
}

/// Validate that a nested SET path is compatible with the existing item structure.
///
/// DynamoDB requires that intermediate containers in a nested path already exist
/// and have the correct type. For `.field`, the parent must be a map. For `[idx]`,
/// the parent must be a list. If the parent attribute does not exist at all, it is
/// also an error.
fn validate_nested_path_for_set(
    item: &HashMap<String, AttributeValue>,
    elements: &[PathElement],
    names: &HashMap<String, String>,
) -> Result<(), ExpressionError> {
    if elements.len() <= 1 {
        return Ok(());
    }

    // Resolve the top-level attribute name.
    let PathElement::Attribute(top) = &elements[0] else {
        return Ok(());
    };
    let top_name = resolve_name_ref(top, names);

    let Some(current) = item.get(&top_name) else {
        // The top-level attribute doesn't exist. For nested paths, this is an error
        // because DynamoDB requires existing intermediate containers.
        return Err(ExpressionError::InvalidOperand {
            operation: "SET".to_owned(),
            message: "The document path provided in the update expression \
                      is invalid for update"
                .to_owned(),
        });
    };

    validate_path_type(current, &elements[1..], names)
}

/// Recursively validate that a path matches the container types.
fn validate_path_type(
    current: &AttributeValue,
    remaining: &[PathElement],
    names: &HashMap<String, String>,
) -> Result<(), ExpressionError> {
    if remaining.is_empty() {
        return Ok(());
    }

    match &remaining[0] {
        PathElement::Attribute(name) => {
            let resolved = resolve_name_ref(name, names);
            match current {
                AttributeValue::M(map) => {
                    if remaining.len() > 1 {
                        if let Some(child) = map.get(&resolved) {
                            return validate_path_type(child, &remaining[1..], names);
                        }
                    }
                    Ok(())
                }
                _ => Err(ExpressionError::InvalidOperand {
                    operation: "SET".to_owned(),
                    message: "The document path provided in the update expression \
                              is invalid for update"
                        .to_owned(),
                }),
            }
        }
        PathElement::Index(idx) => match current {
            AttributeValue::L(list) => {
                if remaining.len() > 1 {
                    if let Some(child) = list.get(*idx) {
                        return validate_path_type(child, &remaining[1..], names);
                    }
                }
                Ok(())
            }
            _ => Err(ExpressionError::InvalidOperand {
                operation: "SET".to_owned(),
                message: "The document path provided in the update expression \
                          is invalid for update"
                    .to_owned(),
            }),
        },
    }
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
            if *idx >= list.len() {
                // DynamoDB behavior: if the index is beyond the list length,
                // append the value at the end of the list (no NULL padding).
                if remaining.len() == 1 {
                    list.push(value);
                }
                // For deeper nesting with out-of-bounds index, silently ignore
                // since there is no element to traverse into.
            } else if remaining.len() == 1 {
                list[*idx] = value;
            } else {
                set_value_in_container(&mut list[*idx], &remaining[1..], value, names);
            }
        }
    }
}

/// Remove an attribute at the given path from an item.
///
/// # Errors
///
/// Returns `ExpressionError` if a nested path encounters a type mismatch
/// (e.g., `.field` on a non-map when the item exists, `[idx]` on a non-list).
fn apply_remove(
    item: &mut HashMap<String, AttributeValue>,
    path: &AttributePath,
    names: &HashMap<String, String>,
) -> Result<(), ExpressionError> {
    if path.elements.is_empty() {
        return Ok(());
    }

    let PathElement::Attribute(top) = &path.elements[0] else {
        return Ok(());
    };
    let top_name = resolve_name_ref(top, names);

    if path.elements.len() == 1 {
        item.remove(&top_name);
        return Ok(());
    }

    // For nested paths, validate the path first.
    if let Some(existing) = item.get(&top_name) {
        validate_path_type(existing, &path.elements[1..], names)?;
    } else {
        // Path root doesn't exist - this is a validation error for nested paths.
        return Err(ExpressionError::InvalidOperand {
            operation: "REMOVE".to_owned(),
            message: "The document path provided in the update expression \
                      is invalid for update"
                .to_owned(),
        });
    }

    // For nested paths, traverse to the parent container and remove.
    if let Some(container) = item.get_mut(&top_name) {
        remove_at_path(container, &path.elements[1..], names);
    }
    Ok(())
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
        (_, Some(existing_val)) => {
            let add_type = add_val.type_descriptor();
            let existing_type = existing_val.type_descriptor();
            Err(ExpressionError::InvalidOperand {
                operation: "ADD".to_owned(),
                message: format!(
                    "Type mismatch for ADD; operator type: {add_type}, \
                     existing type: {existing_type}"
                ),
            })
        }
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
    fn test_should_append_to_list_for_out_of_bounds_set() {
        let item = make_item(&[(
            "items",
            AttributeValue::L(vec![AttributeValue::S("a".to_owned())]),
        )]);
        let names = empty_names();
        let values = make_values(&[(":val", AttributeValue::S("c".to_owned()))]);

        // DynamoDB behavior: SET with an index beyond the list length appends
        // the value at the end, without padding with NULLs.
        let update = parse_update("SET items[3] = :val").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        match result.get("items") {
            Some(AttributeValue::L(list)) => {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0], AttributeValue::S("a".to_owned()));
                assert_eq!(list[1], AttributeValue::S("c".to_owned()));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn test_should_append_multiple_out_of_bounds_sorted_by_index() {
        let item = make_item(&[(
            "a",
            AttributeValue::L(vec![
                AttributeValue::S("one".to_owned()),
                AttributeValue::S("two".to_owned()),
                AttributeValue::S("three".to_owned()),
            ]),
        )]);
        let names = empty_names();
        let values = make_values(&[
            (":val1", AttributeValue::S("a1".to_owned())),
            (":val2", AttributeValue::S("a2".to_owned())),
            (":val3", AttributeValue::S("a3".to_owned())),
            (":val4", AttributeValue::S("a4".to_owned())),
        ]);

        // DynamoDB sorts out-of-bounds SET actions by their index.
        let update =
            parse_update("SET a[84] = :val1, a[37] = :val2, a[17] = :val3, a[50] = :val4").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_update(&update).unwrap();
        match result.get("a") {
            Some(AttributeValue::L(list)) => {
                assert_eq!(list.len(), 7);
                assert_eq!(list[0], AttributeValue::S("one".to_owned()));
                assert_eq!(list[1], AttributeValue::S("two".to_owned()));
                assert_eq!(list[2], AttributeValue::S("three".to_owned()));
                // Sorted by index: 17(a3), 37(a2), 50(a4), 84(a1)
                assert_eq!(list[3], AttributeValue::S("a3".to_owned()));
                assert_eq!(list[4], AttributeValue::S("a2".to_owned()));
                assert_eq!(list[5], AttributeValue::S("a4".to_owned()));
                assert_eq!(list[6], AttributeValue::S("a1".to_owned()));
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

    // -----------------------------------------------------------------------
    // Projection: nested paths with list indices and shared prefixes
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_project_nested_list_index() {
        // Projecting "a[0]" on {a: [2, 4, 6]} should return {a: [2]}.
        let item = make_item(&[(
            "a",
            AttributeValue::L(vec![
                AttributeValue::N("2".to_owned()),
                AttributeValue::N("4".to_owned()),
                AttributeValue::N("6".to_owned()),
            ]),
        )]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("a[0]").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        match result.get("a") {
            Some(AttributeValue::L(list)) => {
                assert_eq!(list.len(), 1);
                assert_eq!(list[0], AttributeValue::N("2".to_owned()));
            }
            other => panic!("expected L([2]), got {other:?}"),
        }
    }

    #[test]
    fn test_should_project_shared_prefix_nested_map() {
        // Projecting "a.b[0], a.c" on {a: {b: [2, 4, 6], c: 5, d: 9}}
        // should return {a: {b: [2], c: 5}}.
        let mut inner = HashMap::new();
        inner.insert(
            "b".to_owned(),
            AttributeValue::L(vec![
                AttributeValue::N("2".to_owned()),
                AttributeValue::N("4".to_owned()),
                AttributeValue::N("6".to_owned()),
            ]),
        );
        inner.insert("c".to_owned(), AttributeValue::N("5".to_owned()));
        inner.insert("d".to_owned(), AttributeValue::N("9".to_owned()));
        let item = make_item(&[("a", AttributeValue::M(inner))]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("a.b[0], a.c").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        assert_eq!(result.len(), 1);
        match result.get("a") {
            Some(AttributeValue::M(map)) => {
                assert!(map.contains_key("b"));
                assert!(map.contains_key("c"));
                assert!(!map.contains_key("d"));
                match map.get("b") {
                    Some(AttributeValue::L(list)) => {
                        assert_eq!(list.len(), 1);
                        assert_eq!(list[0], AttributeValue::N("2".to_owned()));
                    }
                    other => panic!("expected L([2]), got {other:?}"),
                }
                assert_eq!(map.get("c"), Some(&AttributeValue::N("5".to_owned())));
            }
            other => panic!("expected M, got {other:?}"),
        }
    }

    #[test]
    fn test_should_return_empty_for_out_of_bounds_list_index() {
        // Projecting "a[10]" on {a: [1, 2]} should return {} (value not found).
        let item = make_item(&[(
            "a",
            AttributeValue::L(vec![
                AttributeValue::N("1".to_owned()),
                AttributeValue::N("2".to_owned()),
            ]),
        )]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("a[10]").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        // The item "a" should not appear because the index is out of bounds.
        assert!(result.is_empty());
    }

    #[test]
    fn test_should_project_deeply_nested_path() {
        // Projecting "a.b.c" on {a: {b: {c: 1, d: 2}}} should return {a: {b: {c: 1}}}.
        let mut c_map = HashMap::new();
        c_map.insert("c".to_owned(), AttributeValue::N("1".to_owned()));
        c_map.insert("d".to_owned(), AttributeValue::N("2".to_owned()));
        let mut b_map = HashMap::new();
        b_map.insert("b".to_owned(), AttributeValue::M(c_map));
        let item = make_item(&[("a", AttributeValue::M(b_map))]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("a.b.c").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        match result.get("a") {
            Some(AttributeValue::M(a_map)) => match a_map.get("b") {
                Some(AttributeValue::M(b_map)) => {
                    assert_eq!(b_map.get("c"), Some(&AttributeValue::N("1".to_owned())));
                    assert!(!b_map.contains_key("d"));
                }
                other => panic!("expected M for b, got {other:?}"),
            },
            other => panic!("expected M for a, got {other:?}"),
        }
    }

    #[test]
    fn test_should_return_empty_for_missing_nested_path() {
        // Projecting "a.b.c" on {a: {x: 1}} should return {} (path not found).
        let mut inner = HashMap::new();
        inner.insert("x".to_owned(), AttributeValue::N("1".to_owned()));
        let item = make_item(&[("a", AttributeValue::M(inner))]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("a.b.c").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        assert!(result.is_empty());
    }

    #[test]
    fn test_should_project_list_then_map_access() {
        // Projecting "a[0].name" on {a: [{name: "Alice", age: 30}]}
        // should return {a: [{name: "Alice"}]}.
        let mut person = HashMap::new();
        person.insert("name".to_owned(), AttributeValue::S("Alice".to_owned()));
        person.insert("age".to_owned(), AttributeValue::N("30".to_owned()));
        let item = make_item(&[("a", AttributeValue::L(vec![AttributeValue::M(person)]))]);
        let names = empty_names();
        let values = HashMap::new();

        let paths = parse_projection("a[0].name").unwrap();
        let ctx = EvalContext {
            item: &item,
            names: &names,
            values: &values,
        };
        let result = ctx.apply_projection(&paths);
        match result.get("a") {
            Some(AttributeValue::L(list)) => {
                assert_eq!(list.len(), 1);
                match &list[0] {
                    AttributeValue::M(map) => {
                        assert_eq!(
                            map.get("name"),
                            Some(&AttributeValue::S("Alice".to_owned()))
                        );
                        assert!(!map.contains_key("age"));
                    }
                    other => panic!("expected M inside list, got {other:?}"),
                }
            }
            other => panic!("expected L, got {other:?}"),
        }
    }
}
