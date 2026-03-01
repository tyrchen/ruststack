//! DynamoDB output types for the 12 MVP operations.
//!
//! All output structs use `PascalCase` JSON field naming to match the DynamoDB
//! wire protocol (`awsJson1_0`). Optional fields are omitted when `None`,
//! empty `HashMap`s and `Vec`s are omitted to produce minimal JSON responses.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::attribute_value::AttributeValue;
use crate::types::{
    ConsumedCapacity, ItemCollectionMetrics, KeysAndAttributes, TableDescription, WriteRequest,
};

// ---------------------------------------------------------------------------
// Table management
// ---------------------------------------------------------------------------

/// Output for the `CreateTable` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateTableOutput {
    /// The properties of the newly created table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_description: Option<TableDescription>,
}

/// Output for the `DeleteTable` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteTableOutput {
    /// The properties of the table that was deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_description: Option<TableDescription>,
}

/// Output for the `DescribeTable` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeTableOutput {
    /// The properties of the table.
    #[serde(rename = "Table", skip_serializing_if = "Option::is_none")]
    pub table: Option<TableDescription>,
}

/// Output for the `UpdateTable` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateTableOutput {
    /// The properties of the updated table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_description: Option<TableDescription>,
}

/// Output for the `ListTables` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListTablesOutput {
    /// The names of the tables associated with the current account and region.
    #[serde(default)]
    pub table_names: Vec<String>,

    /// The name of the last table in the current page of results. Use this
    /// value as `ExclusiveStartTableName` in a subsequent request to continue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_evaluated_table_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Item CRUD
// ---------------------------------------------------------------------------

/// Output for the `PutItem` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PutItemOutput {
    /// The attribute values as they appeared before the `PutItem` operation
    /// (only returned when `ReturnValues` is specified).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, AttributeValue>,

    /// The capacity units consumed by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_capacity: Option<ConsumedCapacity>,

    /// Information about item collections modified by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_collection_metrics: Option<ItemCollectionMetrics>,
}

/// Output for the `GetItem` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetItemOutput {
    /// A map of attribute names to `AttributeValue` objects for the retrieved
    /// item. Returns `None` if the item does not exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<HashMap<String, AttributeValue>>,

    /// The capacity units consumed by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_capacity: Option<ConsumedCapacity>,
}

/// Output for the `UpdateItem` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateItemOutput {
    /// The attribute values as they appeared before or after the update
    /// (depending on the `ReturnValues` setting).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, AttributeValue>,

    /// The capacity units consumed by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_capacity: Option<ConsumedCapacity>,

    /// Information about item collections modified by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_collection_metrics: Option<ItemCollectionMetrics>,
}

/// Output for the `DeleteItem` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteItemOutput {
    /// The attribute values as they appeared before the deletion (only
    /// returned when `ReturnValues` is `ALL_OLD`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, AttributeValue>,

    /// The capacity units consumed by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_capacity: Option<ConsumedCapacity>,

    /// Information about item collections modified by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_collection_metrics: Option<ItemCollectionMetrics>,
}

// ---------------------------------------------------------------------------
// Query & Scan
// ---------------------------------------------------------------------------

/// Output for the `Query` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueryOutput {
    /// An array of item attributes that match the query conditions.
    /// Omitted when `Select=COUNT` (empty vec is not serialized).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<HashMap<String, AttributeValue>>,

    /// The number of items in the response.
    pub count: i32,

    /// The number of items evaluated before the filter expression was applied.
    pub scanned_count: i32,

    /// The primary key of the item where the query operation stopped. Use this
    /// value as `ExclusiveStartKey` in a subsequent query to continue.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub last_evaluated_key: HashMap<String, AttributeValue>,

    /// The capacity units consumed by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_capacity: Option<ConsumedCapacity>,
}

/// Output for the `Scan` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ScanOutput {
    /// An array of item attributes that match the scan conditions.
    /// Omitted when `Select=COUNT` (empty vec is not serialized).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<HashMap<String, AttributeValue>>,

    /// The number of items in the response.
    pub count: i32,

    /// The number of items evaluated before the filter expression was applied.
    pub scanned_count: i32,

    /// The primary key of the item where the scan operation stopped. Use this
    /// value as `ExclusiveStartKey` in a subsequent scan to continue.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub last_evaluated_key: HashMap<String, AttributeValue>,

    /// The capacity units consumed by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_capacity: Option<ConsumedCapacity>,
}

// ---------------------------------------------------------------------------
// Batch operations
// ---------------------------------------------------------------------------

/// Output for the `BatchGetItem` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BatchGetItemOutput {
    /// A map of table names to the items retrieved from each table.
    #[serde(default)]
    pub responses: HashMap<String, Vec<HashMap<String, AttributeValue>>>,

    /// A map of tables and their respective keys that were not processed. Use
    /// these values as `RequestItems` in a subsequent `BatchGetItem` call.
    #[serde(default)]
    pub unprocessed_keys: HashMap<String, KeysAndAttributes>,

    /// The capacity units consumed by the operation for each table.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumed_capacity: Vec<ConsumedCapacity>,
}

/// Output for the `BatchWriteItem` operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BatchWriteItemOutput {
    /// A map of tables and their respective `WriteRequest` objects that were
    /// not processed. Use these values as `RequestItems` in a subsequent
    /// `BatchWriteItem` call.
    #[serde(default)]
    pub unprocessed_items: HashMap<String, Vec<WriteRequest>>,

    /// A map of tables to item collection metrics for the tables that were
    /// affected by the operation.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub item_collection_metrics: HashMap<String, Vec<ItemCollectionMetrics>>,

    /// The capacity units consumed by the operation for each table.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumed_capacity: Vec<ConsumedCapacity>,
}
