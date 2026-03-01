//! DynamoDB provider implementing all 12 MVP operations.

use std::collections::HashMap;
use std::sync::Arc;

use ruststack_dynamodb_model::AttributeValue;
use ruststack_dynamodb_model::error::DynamoDBError;
use ruststack_dynamodb_model::input::{
    BatchGetItemInput, BatchWriteItemInput, CreateTableInput, DeleteItemInput, DeleteTableInput,
    DescribeTableInput, GetItemInput, ListTablesInput, PutItemInput, QueryInput, ScanInput,
    UpdateItemInput,
};
use ruststack_dynamodb_model::output::{
    BatchGetItemOutput, BatchWriteItemOutput, CreateTableOutput, DeleteItemOutput,
    DeleteTableOutput, DescribeTableOutput, GetItemOutput, ListTablesOutput, PutItemOutput,
    QueryOutput, ScanOutput, UpdateItemOutput,
};
use ruststack_dynamodb_model::types::{
    AttributeDefinition, BillingMode, KeyType, ScalarAttributeType, TableStatus,
};

use crate::config::DynamoDBConfig;
use crate::error::{expression_error_to_dynamodb, storage_error_to_dynamodb};
use crate::expression::{EvalContext, parse_condition, parse_projection, parse_update};
use crate::state::{DynamoDBServiceState, DynamoDBTable};
use crate::storage::{
    KeyAttribute, KeySchema, SortKeyCondition, SortableAttributeValue, TableStorage,
    calculate_item_size, extract_primary_key,
};

/// Maximum item size in bytes (400 KB).
const MAX_ITEM_SIZE_BYTES: u64 = 400 * 1024;

/// Main DynamoDB provider implementing all operations.
#[derive(Debug)]
pub struct RustStackDynamoDB {
    /// Service state owning all tables.
    pub state: Arc<DynamoDBServiceState>,
    /// Configuration.
    pub config: Arc<DynamoDBConfig>,
}

impl RustStackDynamoDB {
    /// Create a new DynamoDB provider.
    #[must_use]
    pub fn new(config: DynamoDBConfig) -> Self {
        Self {
            state: Arc::new(DynamoDBServiceState::new()),
            config: Arc::new(config),
        }
    }

    /// Reset all state (for testing).
    pub fn reset(&self) {
        self.state.reset();
    }
}

// ---------------------------------------------------------------------------
// Table management
// ---------------------------------------------------------------------------

impl RustStackDynamoDB {
    /// Handle `CreateTable`.
    pub fn handle_create_table(
        &self,
        input: CreateTableInput,
    ) -> Result<CreateTableOutput, DynamoDBError> {
        // Validate key schema.
        let key_schema = parse_key_schema(&input.key_schema, &input.attribute_definitions)?;

        // Build storage.
        let storage = TableStorage::new(key_schema.clone());

        let table_name = input.table_name.clone();
        let arn = format!(
            "arn:aws:dynamodb:{}:000000000000:table/{}",
            self.config.default_region, table_name,
        );

        let billing = input.billing_mode.unwrap_or(BillingMode::PayPerRequest);

        let table = DynamoDBTable {
            name: table_name,
            status: TableStatus::Active,
            key_schema_elements: input.key_schema,
            key_schema,
            attribute_definitions: input.attribute_definitions,
            billing_mode: billing,
            provisioned_throughput: input.provisioned_throughput,
            gsi_definitions: input.global_secondary_indexes,
            lsi_definitions: input.local_secondary_indexes,
            stream_specification: input.stream_specification,
            sse_specification: input.sse_specification,
            tags: parking_lot::RwLock::new(input.tags),
            arn,
            created_at: chrono::Utc::now(),
            storage,
        };

        let table = self.state.create_table(table)?;
        Ok(CreateTableOutput {
            table_description: Some(table.to_description()),
        })
    }

    /// Handle `DeleteTable`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_delete_table(
        &self,
        input: DeleteTableInput,
    ) -> Result<DeleteTableOutput, DynamoDBError> {
        let table = self.state.delete_table(&input.table_name)?;
        Ok(DeleteTableOutput {
            table_description: Some(table.to_description()),
        })
    }

    /// Handle `DescribeTable`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_describe_table(
        &self,
        input: DescribeTableInput,
    ) -> Result<DescribeTableOutput, DynamoDBError> {
        let table = self.state.require_table(&input.table_name)?;
        Ok(DescribeTableOutput {
            table: Some(table.to_description()),
        })
    }

    /// Handle `ListTables`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_list_tables(
        &self,
        input: ListTablesInput,
    ) -> Result<ListTablesOutput, DynamoDBError> {
        let all_names = self.state.list_table_names();
        let limit = usize::try_from(input.limit.unwrap_or(100).clamp(1, 100)).unwrap_or(100);

        let start_idx = if let Some(ref start) = input.exclusive_start_table_name {
            all_names
                .iter()
                .position(|n| n.as_str() > start.as_str())
                .unwrap_or(all_names.len())
        } else {
            0
        };

        let page: Vec<String> = all_names
            .into_iter()
            .skip(start_idx)
            .take(limit + 1)
            .collect();

        let (table_names, last) = if page.len() > limit {
            let last = page[limit - 1].clone();
            (page[..limit].to_vec(), Some(last))
        } else {
            (page, None)
        };

        Ok(ListTablesOutput {
            table_names,
            last_evaluated_table_name: last,
        })
    }
}

// ---------------------------------------------------------------------------
// Item CRUD
// ---------------------------------------------------------------------------

impl RustStackDynamoDB {
    /// Handle `PutItem`.
    pub fn handle_put_item(&self, input: PutItemInput) -> Result<PutItemOutput, DynamoDBError> {
        let table = self.state.require_table(&input.table_name)?;

        // Validate item size.
        let size = calculate_item_size(&input.item);
        if size > MAX_ITEM_SIZE_BYTES {
            return Err(DynamoDBError::validation(format!(
                "Item size has exceeded the maximum allowed size of {MAX_ITEM_SIZE_BYTES} bytes"
            )));
        }

        // Evaluate condition expression if present.
        if let Some(ref condition) = input.condition_expression {
            let existing = {
                let pk = extract_primary_key(&table.key_schema, &input.item)
                    .map_err(storage_error_to_dynamodb)?;
                table.storage.get_item(&pk)
            };
            let empty = HashMap::new();
            let item_ref = existing.as_ref().unwrap_or(&empty);
            let expr = parse_condition(condition).map_err(expression_error_to_dynamodb)?;
            let ctx = EvalContext {
                item: item_ref,
                names: &input.expression_attribute_names,
                values: &input.expression_attribute_values,
            };
            let result = ctx.evaluate(&expr).map_err(expression_error_to_dynamodb)?;
            if !result {
                return Err(DynamoDBError::conditional_check_failed(
                    "The conditional request failed",
                ));
            }
        }

        let old = table
            .storage
            .put_item(input.item)
            .map_err(storage_error_to_dynamodb)?;

        let attributes = match input.return_values {
            Some(rv) if rv.as_str() == "ALL_OLD" => old.unwrap_or_default(),
            _ => HashMap::new(),
        };

        Ok(PutItemOutput {
            attributes,
            consumed_capacity: None,
            item_collection_metrics: None,
        })
    }

    /// Handle `GetItem`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_get_item(&self, input: GetItemInput) -> Result<GetItemOutput, DynamoDBError> {
        let table = self.state.require_table(&input.table_name)?;
        let pk = extract_primary_key(&table.key_schema, &input.key)
            .map_err(storage_error_to_dynamodb)?;

        let item = table.storage.get_item(&pk);

        // Apply projection if specified.
        let item = match (item, &input.projection_expression) {
            (Some(item), Some(proj_expr)) => {
                let paths = parse_projection(proj_expr).map_err(expression_error_to_dynamodb)?;
                let ctx = EvalContext {
                    item: &item,
                    names: &input.expression_attribute_names,
                    values: &HashMap::new(),
                };
                ctx.apply_projection(&paths)
            }
            (item, _) => item.unwrap_or_default(),
        };

        Ok(GetItemOutput {
            item,
            consumed_capacity: None,
        })
    }

    /// Handle `DeleteItem`.
    pub fn handle_delete_item(
        &self,
        input: DeleteItemInput,
    ) -> Result<DeleteItemOutput, DynamoDBError> {
        let table = self.state.require_table(&input.table_name)?;
        let pk = extract_primary_key(&table.key_schema, &input.key)
            .map_err(storage_error_to_dynamodb)?;

        // Evaluate condition expression if present.
        if let Some(ref condition) = input.condition_expression {
            let existing = table.storage.get_item(&pk);
            let empty = HashMap::new();
            let item_ref = existing.as_ref().unwrap_or(&empty);
            let expr = parse_condition(condition).map_err(expression_error_to_dynamodb)?;
            let ctx = EvalContext {
                item: item_ref,
                names: &input.expression_attribute_names,
                values: &input.expression_attribute_values,
            };
            let result = ctx.evaluate(&expr).map_err(expression_error_to_dynamodb)?;
            if !result {
                return Err(DynamoDBError::conditional_check_failed(
                    "The conditional request failed",
                ));
            }
        }

        let old = table.storage.delete_item(&pk);

        let attributes = match input.return_values {
            Some(rv) if rv.as_str() == "ALL_OLD" => old.unwrap_or_default(),
            _ => HashMap::new(),
        };

        Ok(DeleteItemOutput {
            attributes,
            consumed_capacity: None,
            item_collection_metrics: None,
        })
    }

    /// Handle `UpdateItem`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_update_item(
        &self,
        input: UpdateItemInput,
    ) -> Result<UpdateItemOutput, DynamoDBError> {
        let table = self.state.require_table(&input.table_name)?;
        let pk = extract_primary_key(&table.key_schema, &input.key)
            .map_err(storage_error_to_dynamodb)?;

        let existing = table.storage.get_item(&pk);
        let mut item = existing.clone().unwrap_or_else(|| input.key.clone());

        // Evaluate condition expression.
        if let Some(ref condition) = input.condition_expression {
            let empty = HashMap::new();
            let item_ref = existing.as_ref().unwrap_or(&empty);
            let expr = parse_condition(condition).map_err(expression_error_to_dynamodb)?;
            let ctx = EvalContext {
                item: item_ref,
                names: &input.expression_attribute_names,
                values: &input.expression_attribute_values,
            };
            let result = ctx.evaluate(&expr).map_err(expression_error_to_dynamodb)?;
            if !result {
                return Err(DynamoDBError::conditional_check_failed(
                    "The conditional request failed",
                ));
            }
        }

        // Apply update expression.
        if let Some(ref update_expr) = input.update_expression {
            let parsed = parse_update(update_expr).map_err(expression_error_to_dynamodb)?;
            let ctx = EvalContext {
                item: &item,
                names: &input.expression_attribute_names,
                values: &input.expression_attribute_values,
            };
            item = ctx
                .apply_update(&parsed)
                .map_err(expression_error_to_dynamodb)?;
        }

        // Validate updated item size.
        let size = calculate_item_size(&item);
        if size > MAX_ITEM_SIZE_BYTES {
            return Err(DynamoDBError::validation(format!(
                "Item size has exceeded the maximum allowed size of {MAX_ITEM_SIZE_BYTES} bytes"
            )));
        }

        // Store updated item.
        let old_item = table
            .storage
            .put_item(item.clone())
            .map_err(storage_error_to_dynamodb)?;

        let attributes = match input.return_values {
            Some(ref rv) if rv.as_str() == "ALL_OLD" => old_item.or(existing).unwrap_or_default(),
            Some(ref rv) if rv.as_str() == "ALL_NEW" => item,
            Some(ref rv) if rv.as_str() == "UPDATED_OLD" => {
                // Return only updated attributes from old item.
                old_item.or(existing).unwrap_or_default()
            }
            Some(ref rv) if rv.as_str() == "UPDATED_NEW" => item,
            _ => HashMap::new(),
        };

        Ok(UpdateItemOutput {
            attributes,
            consumed_capacity: None,
            item_collection_metrics: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Query & Scan
// ---------------------------------------------------------------------------

impl RustStackDynamoDB {
    /// Handle `Query`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_query(&self, input: QueryInput) -> Result<QueryOutput, DynamoDBError> {
        let table = self.state.require_table(&input.table_name)?;

        // Parse key condition expression.
        let key_condition = input.key_condition_expression.as_deref().ok_or_else(|| {
            DynamoDBError::validation("KeyConditionExpression is required for Query")
        })?;

        let expr = parse_condition(key_condition).map_err(expression_error_to_dynamodb)?;

        // Extract partition key value and sort condition from the parsed expression.
        let (partition_value, sort_condition) = extract_key_condition(
            &expr,
            &table.key_schema,
            &input.expression_attribute_names,
            &input.expression_attribute_values,
        )?;

        let scan_forward = input.scan_index_forward.unwrap_or(true);
        let limit = input
            .limit
            .map(|l| usize::try_from(l.max(0)).unwrap_or(usize::MAX));

        let exclusive_start_sort = if input.exclusive_start_key.is_empty() {
            None
        } else {
            let start_pk = extract_primary_key(&table.key_schema, &input.exclusive_start_key)
                .map_err(storage_error_to_dynamodb)?;
            start_pk.sort_key
        };

        let (mut items, last_key_sort) = table.storage.query(
            &partition_value,
            sort_condition.as_ref(),
            scan_forward,
            limit,
            exclusive_start_sort.as_ref(),
        );

        let scanned_count = i32::try_from(items.len()).unwrap_or(i32::MAX);

        // Apply filter expression if present.
        if let Some(ref filter) = input.filter_expression {
            let filter_expr = parse_condition(filter).map_err(expression_error_to_dynamodb)?;
            items.retain(|item| {
                let ctx = EvalContext {
                    item,
                    names: &input.expression_attribute_names,
                    values: &input.expression_attribute_values,
                };
                ctx.evaluate(&filter_expr).unwrap_or(false)
            });
        }

        // Apply projection expression if present.
        if let Some(ref proj) = input.projection_expression {
            let paths = parse_projection(proj).map_err(expression_error_to_dynamodb)?;
            items = items
                .into_iter()
                .map(|item| {
                    let ctx = EvalContext {
                        item: &item,
                        names: &input.expression_attribute_names,
                        values: &HashMap::new(),
                    };
                    ctx.apply_projection(&paths)
                })
                .collect();
        }

        let count = i32::try_from(items.len()).unwrap_or(i32::MAX);

        // Build last evaluated key for pagination.
        let last_evaluated_key = last_key_sort.map(|pk| {
            let sort_av = pk
                .sort_key
                .as_ref()
                .and_then(SortableAttributeValue::to_attribute_value);
            build_last_evaluated_key(&table.key_schema, &pk.partition_key, sort_av.as_ref())
        });

        Ok(QueryOutput {
            items,
            count,
            scanned_count,
            last_evaluated_key: last_evaluated_key.unwrap_or_default(),
            consumed_capacity: None,
        })
    }

    /// Handle `Scan`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_scan(&self, input: ScanInput) -> Result<ScanOutput, DynamoDBError> {
        let table = self.state.require_table(&input.table_name)?;

        let limit = input
            .limit
            .map(|l| usize::try_from(l.max(0)).unwrap_or(usize::MAX));
        let exclusive_start = if input.exclusive_start_key.is_empty() {
            None
        } else {
            Some(
                extract_primary_key(&table.key_schema, &input.exclusive_start_key)
                    .map_err(storage_error_to_dynamodb)?,
            )
        };

        let (mut items, last_key) = table.storage.scan(limit, exclusive_start.as_ref());

        let scanned_count = i32::try_from(items.len()).unwrap_or(i32::MAX);

        // Apply filter expression if present.
        if let Some(ref filter) = input.filter_expression {
            let filter_expr = parse_condition(filter).map_err(expression_error_to_dynamodb)?;
            items.retain(|item| {
                let ctx = EvalContext {
                    item,
                    names: &input.expression_attribute_names,
                    values: &input.expression_attribute_values,
                };
                ctx.evaluate(&filter_expr).unwrap_or(false)
            });
        }

        // Apply projection expression if present.
        if let Some(ref proj) = input.projection_expression {
            let paths = parse_projection(proj).map_err(expression_error_to_dynamodb)?;
            items = items
                .into_iter()
                .map(|item| {
                    let ctx = EvalContext {
                        item: &item,
                        names: &input.expression_attribute_names,
                        values: &HashMap::new(),
                    };
                    ctx.apply_projection(&paths)
                })
                .collect();
        }

        let count = i32::try_from(items.len()).unwrap_or(i32::MAX);

        let last_evaluated_key = last_key.map(|pk| {
            let sort_av = pk
                .sort_key
                .as_ref()
                .and_then(SortableAttributeValue::to_attribute_value);
            build_last_evaluated_key(&table.key_schema, &pk.partition_key, sort_av.as_ref())
        });

        Ok(ScanOutput {
            items,
            count,
            scanned_count,
            last_evaluated_key: last_evaluated_key.unwrap_or_default(),
            consumed_capacity: None,
        })
    }
}

// ---------------------------------------------------------------------------
// Batch operations
// ---------------------------------------------------------------------------

impl RustStackDynamoDB {
    /// Handle `BatchGetItem`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_batch_get_item(
        &self,
        input: BatchGetItemInput,
    ) -> Result<BatchGetItemOutput, DynamoDBError> {
        let mut responses: HashMap<String, Vec<HashMap<String, AttributeValue>>> = HashMap::new();

        for (table_name, keys_and_attrs) in &input.request_items {
            let table = self.state.require_table(table_name)?;
            let mut table_items = Vec::new();

            for key in &keys_and_attrs.keys {
                let pk = extract_primary_key(&table.key_schema, key)
                    .map_err(storage_error_to_dynamodb)?;
                if let Some(item) = table.storage.get_item(&pk) {
                    // Apply projection if specified.
                    let item = if let Some(ref proj) = keys_and_attrs.projection_expression {
                        let paths = parse_projection(proj).map_err(expression_error_to_dynamodb)?;
                        let names = keys_and_attrs
                            .expression_attribute_names
                            .clone()
                            .unwrap_or_default();
                        let ctx = EvalContext {
                            item: &item,
                            names: &names,
                            values: &HashMap::new(),
                        };
                        ctx.apply_projection(&paths)
                    } else {
                        item
                    };
                    table_items.push(item);
                }
            }

            if !table_items.is_empty() {
                responses.insert(table_name.clone(), table_items);
            }
        }

        Ok(BatchGetItemOutput {
            responses,
            unprocessed_keys: HashMap::new(),
            consumed_capacity: Vec::new(),
        })
    }

    /// Handle `BatchWriteItem`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle_batch_write_item(
        &self,
        input: BatchWriteItemInput,
    ) -> Result<BatchWriteItemOutput, DynamoDBError> {
        for (table_name, write_requests) in &input.request_items {
            let table = self.state.require_table(table_name)?;

            for wr in write_requests {
                if let Some(ref put) = wr.put_request {
                    // Validate item size.
                    let size = calculate_item_size(&put.item);
                    if size > MAX_ITEM_SIZE_BYTES {
                        return Err(DynamoDBError::validation(format!(
                            "Item size has exceeded the maximum allowed size of {MAX_ITEM_SIZE_BYTES} bytes"
                        )));
                    }
                    table
                        .storage
                        .put_item(put.item.clone())
                        .map_err(storage_error_to_dynamodb)?;
                } else if let Some(ref del) = wr.delete_request {
                    let pk = extract_primary_key(&table.key_schema, &del.key)
                        .map_err(storage_error_to_dynamodb)?;
                    table.storage.delete_item(&pk);
                }
            }
        }

        Ok(BatchWriteItemOutput {
            unprocessed_items: HashMap::new(),
            item_collection_metrics: HashMap::new(),
            consumed_capacity: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Parse key schema elements and attribute definitions into a `KeySchema`.
fn parse_key_schema(
    elements: &[ruststack_dynamodb_model::types::KeySchemaElement],
    definitions: &[AttributeDefinition],
) -> Result<KeySchema, DynamoDBError> {
    let mut partition_key = None;
    let mut sort_key = None;

    for elem in elements {
        match elem.key_type {
            KeyType::Hash => partition_key = Some(elem.attribute_name.clone()),
            KeyType::Range => sort_key = Some(elem.attribute_name.clone()),
        }
    }

    let pk_name = partition_key
        .ok_or_else(|| DynamoDBError::validation("Key schema must contain a HASH key element"))?;

    let pk_type = find_attribute_type(definitions, &pk_name)?;
    let sk_attr = if let Some(ref sk) = sort_key {
        let sk_type = find_attribute_type(definitions, sk)?;
        Some(KeyAttribute {
            name: sk.clone(),
            attr_type: sk_type,
        })
    } else {
        None
    };

    Ok(KeySchema {
        partition_key: KeyAttribute {
            name: pk_name,
            attr_type: pk_type,
        },
        sort_key: sk_attr,
    })
}

fn find_attribute_type(
    definitions: &[AttributeDefinition],
    name: &str,
) -> Result<ScalarAttributeType, DynamoDBError> {
    definitions
        .iter()
        .find(|d| d.attribute_name == name)
        .map(|d| d.attribute_type.clone())
        .ok_or_else(|| {
            DynamoDBError::validation(format!(
                "Attribute {name} referenced in key schema but not defined in AttributeDefinitions"
            ))
        })
}

/// Extract partition key value and optional sort key condition from a parsed
/// key condition expression. This is a simplified extraction that handles the
/// common patterns: `pk = :val` and `pk = :val AND sk <op> :val2`.
fn extract_key_condition(
    expr: &crate::expression::Expr,
    key_schema: &KeySchema,
    names: &HashMap<String, String>,
    values: &HashMap<String, AttributeValue>,
) -> Result<(AttributeValue, Option<SortKeyCondition>), DynamoDBError> {
    use crate::expression::ast::{CompareOp, Expr, LogicalOp};

    match expr {
        // Simple: pk = :val
        Expr::Compare {
            left,
            op: CompareOp::Eq,
            right,
        } => {
            let pk_val =
                resolve_key_value(left, right, &key_schema.partition_key.name, names, values)?;
            Ok((pk_val, None))
        }
        // AND: pk = :val AND sk <op> :val2
        Expr::Logical {
            op: LogicalOp::And,
            left,
            right,
        } => {
            // Try left as partition, right as sort.
            if let Ok((pk_val, _)) = extract_key_condition(left, key_schema, names, values) {
                let sort_cond = extract_sort_condition(right, key_schema, names, values)?;
                return Ok((pk_val, sort_cond));
            }
            // Try right as partition, left as sort.
            if let Ok((pk_val, _)) = extract_key_condition(right, key_schema, names, values) {
                let sort_cond = extract_sort_condition(left, key_schema, names, values)?;
                return Ok((pk_val, sort_cond));
            }
            Err(DynamoDBError::validation(
                "KeyConditionExpression must contain an equality condition on the partition key",
            ))
        }
        _ => Err(DynamoDBError::validation(
            "KeyConditionExpression must contain an equality condition on the partition key",
        )),
    }
}

/// Extract a sort key condition from an expression node.
fn extract_sort_condition(
    expr: &crate::expression::Expr,
    key_schema: &KeySchema,
    names: &HashMap<String, String>,
    values: &HashMap<String, AttributeValue>,
) -> Result<Option<SortKeyCondition>, DynamoDBError> {
    use crate::expression::ast::{CompareOp, Expr, FunctionName};

    let sk_name = match &key_schema.sort_key {
        Some(sk) => &sk.name,
        None => return Ok(None),
    };

    match expr {
        Expr::Compare { left, op, right } => {
            let val = resolve_sort_value(left, right, sk_name, names, values)?;
            let sortable = to_sortable(sk_name, &val)?;
            let cond = match op {
                CompareOp::Eq => SortKeyCondition::Eq(sortable),
                CompareOp::Lt => SortKeyCondition::Lt(sortable),
                CompareOp::Le => SortKeyCondition::Le(sortable),
                CompareOp::Gt => SortKeyCondition::Gt(sortable),
                CompareOp::Ge => SortKeyCondition::Ge(sortable),
                CompareOp::Ne => {
                    return Err(DynamoDBError::validation(
                        "Sort key condition does not support <> operator",
                    ));
                }
            };
            Ok(Some(cond))
        }
        Expr::Between {
            value: _,
            low,
            high,
        } => {
            let low_val = resolve_operand_value(low, names, values)?;
            let high_val = resolve_operand_value(high, names, values)?;
            let low_s = to_sortable(sk_name, &low_val)?;
            let high_s = to_sortable(sk_name, &high_val)?;
            Ok(Some(SortKeyCondition::Between(low_s, high_s)))
        }
        Expr::Function {
            name: FunctionName::BeginsWith,
            args,
        } if args.len() == 2 => {
            let prefix_val = resolve_operand_value(&args[1], names, values)?;
            match prefix_val {
                AttributeValue::S(s) => Ok(Some(SortKeyCondition::BeginsWith(s))),
                _ => Err(DynamoDBError::validation(
                    "begins_with requires a string argument",
                )),
            }
        }
        _ => Ok(None),
    }
}

/// Resolve an operand to an `AttributeValue`.
fn resolve_operand_value(
    operand: &crate::expression::ast::Operand,
    _names: &HashMap<String, String>,
    values: &HashMap<String, AttributeValue>,
) -> Result<AttributeValue, DynamoDBError> {
    use crate::expression::ast::Operand;
    match operand {
        Operand::Value(name) => values.get(name).cloned().ok_or_else(|| {
            DynamoDBError::validation(format!(
                "Value {name} not found in ExpressionAttributeValues"
            ))
        }),
        _ => Err(DynamoDBError::validation(
            "Expected a value reference (:value) in key condition",
        )),
    }
}

/// Resolve a key equality condition: one side should be the key path, the other
/// a value reference.
fn resolve_key_value(
    left: &crate::expression::ast::Operand,
    right: &crate::expression::ast::Operand,
    key_name: &str,
    names: &HashMap<String, String>,
    values: &HashMap<String, AttributeValue>,
) -> Result<AttributeValue, DynamoDBError> {
    use crate::expression::ast::Operand;

    let is_key_path = |op: &Operand| -> bool {
        if let Operand::Path(path) = op {
            if path.elements.len() == 1 {
                if let crate::expression::ast::PathElement::Attribute(name) = &path.elements[0] {
                    let resolved = if name.starts_with('#') {
                        names
                            .get(name.as_str())
                            .map_or(name.as_str(), String::as_str)
                    } else {
                        name.as_str()
                    };
                    return resolved == key_name;
                }
            }
        }
        false
    };

    if is_key_path(left) {
        resolve_operand_value(right, names, values)
    } else if is_key_path(right) {
        resolve_operand_value(left, names, values)
    } else {
        Err(DynamoDBError::validation(format!(
            "KeyConditionExpression must reference key attribute '{key_name}'"
        )))
    }
}

/// Same as `resolve_key_value` but for sort key.
fn resolve_sort_value(
    left: &crate::expression::ast::Operand,
    right: &crate::expression::ast::Operand,
    key_name: &str,
    names: &HashMap<String, String>,
    values: &HashMap<String, AttributeValue>,
) -> Result<AttributeValue, DynamoDBError> {
    resolve_key_value(left, right, key_name, names, values)
}

/// Convert an `AttributeValue` to a `SortableAttributeValue` for key conditions.
fn to_sortable(
    attr_name: &str,
    val: &AttributeValue,
) -> Result<SortableAttributeValue, DynamoDBError> {
    SortableAttributeValue::from_attribute_value(attr_name, val).map_err(storage_error_to_dynamodb)
}

/// Build a `last_evaluated_key` map from partition and optional sort key.
fn build_last_evaluated_key(
    key_schema: &KeySchema,
    partition: &AttributeValue,
    sort: Option<&AttributeValue>,
) -> HashMap<String, AttributeValue> {
    let mut key = HashMap::new();
    key.insert(key_schema.partition_key.name.clone(), partition.clone());
    if let (Some(sk), Some(sv)) = (&key_schema.sort_key, sort) {
        key.insert(sk.name.clone(), sv.clone());
    }
    key
}
