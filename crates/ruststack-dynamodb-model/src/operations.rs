//! DynamoDB operation enum.

use std::fmt;

/// All supported DynamoDB operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DynamoDBOperation {
    // Table management
    /// Create a new table.
    CreateTable,
    /// Delete a table.
    DeleteTable,
    /// Describe a table.
    DescribeTable,
    /// List all tables.
    ListTables,

    // Item CRUD
    /// Put (insert or replace) an item.
    PutItem,
    /// Get an item by primary key.
    GetItem,
    /// Update an item.
    UpdateItem,
    /// Delete an item by primary key.
    DeleteItem,

    // Query & Scan
    /// Query items by key condition.
    Query,
    /// Scan all items in a table.
    Scan,

    // Batch operations
    /// Batch get items from multiple tables.
    BatchGetItem,
    /// Batch write (put/delete) items to multiple tables.
    BatchWriteItem,
}

impl DynamoDBOperation {
    /// Returns the AWS operation name string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreateTable => "CreateTable",
            Self::DeleteTable => "DeleteTable",
            Self::DescribeTable => "DescribeTable",
            Self::ListTables => "ListTables",
            Self::PutItem => "PutItem",
            Self::GetItem => "GetItem",
            Self::UpdateItem => "UpdateItem",
            Self::DeleteItem => "DeleteItem",
            Self::Query => "Query",
            Self::Scan => "Scan",
            Self::BatchGetItem => "BatchGetItem",
            Self::BatchWriteItem => "BatchWriteItem",
        }
    }

    /// Parse an operation name string into a `DynamoDBOperation`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "CreateTable" => Some(Self::CreateTable),
            "DeleteTable" => Some(Self::DeleteTable),
            "DescribeTable" => Some(Self::DescribeTable),
            "ListTables" => Some(Self::ListTables),
            "PutItem" => Some(Self::PutItem),
            "GetItem" => Some(Self::GetItem),
            "UpdateItem" => Some(Self::UpdateItem),
            "DeleteItem" => Some(Self::DeleteItem),
            "Query" => Some(Self::Query),
            "Scan" => Some(Self::Scan),
            "BatchGetItem" => Some(Self::BatchGetItem),
            "BatchWriteItem" => Some(Self::BatchWriteItem),
            _ => None,
        }
    }
}

impl fmt::Display for DynamoDBOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
