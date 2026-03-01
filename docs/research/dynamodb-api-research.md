# DynamoDB API Comprehensive Research

**Date:** 2026-02-28
**Purpose:** Comprehensive analysis of the AWS DynamoDB API surface, protocol, data model, Smithy model, and what a Rust-based local DynamoDB implementation would need.

---

## Table of Contents

1. [Protocol: AWS JSON 1.0 over HTTP](#1-protocol-aws-json-10-over-http)
2. [Data Model](#2-data-model)
3. [Complete API Operations (66 Operations)](#3-complete-api-operations-66-operations)
4. [Expression Language](#4-expression-language)
5. [PartiQL Support](#5-partiql-support)
6. [Error Handling](#6-error-handling)
7. [DynamoDB Streams (Separate Service)](#7-dynamodb-streams-separate-service)
8. [Constraints and Limits](#8-constraints-and-limits)
9. [Smithy Model and Code Generation](#9-smithy-model-and-code-generation)
10. [Key Differences from S3](#10-key-differences-from-s3)
11. [LocalStack DynamoDB Support](#11-localstack-dynamodb-support)
12. [DynamoDB Local (AWS Official) Limitations](#12-dynamodb-local-aws-official-limitations)
13. [Existing Implementations](#13-existing-implementations)
14. [Implementation Priority Matrix](#14-implementation-priority-matrix)
15. [Architecture Considerations for RustStack](#15-architecture-considerations-for-ruststack)

---

## 1. Protocol: AWS JSON 1.0 over HTTP

DynamoDB uses the `@awsJson1_0` Smithy protocol, which is fundamentally different from S3's REST-based `@restXml` protocol.

### 1.1 Request Format

All DynamoDB requests are **HTTP POST** to the root path (`/`). Operation dispatch is done via the `X-Amz-Target` header, not URL path routing.

```http
POST / HTTP/1.1
Host: dynamodb.<region>.amazonaws.com
Content-Type: application/x-amz-json-1.0
X-Amz-Target: DynamoDB_20120810.GetItem
Authorization: AWS4-HMAC-SHA256 Credential=<...>, SignedHeaders=<...>, Signature=<...>
X-Amz-Date: 20260228T120000Z
Content-Length: <n>

{
    "TableName": "Pets",
    "Key": {
        "AnimalType": {"S": "Dog"},
        "Name": {"S": "Fido"}
    }
}
```

**Key protocol details:**

| Property | Value |
|----------|-------|
| HTTP Method | `POST` (always) |
| URL Path | `/` (always) |
| Content-Type | `application/x-amz-json-1.0` (always) |
| X-Amz-Target | `DynamoDB_20120810.<OperationName>` |
| API Version | `20120810` |
| Auth | AWS Signature Version 4 |
| Body | JSON (UTF-8 encoded) |
| Empty input | `{}` (empty JSON object) |

### 1.2 Response Format

```http
HTTP/1.1 200 OK
x-amzn-RequestId: <RequestId>
x-amz-crc32: <Checksum>
Content-Type: application/x-amz-json-1.0
Content-Length: <n>

{
    "Item": {
        "Age": {"N": "8"},
        "Name": {"S": "Fido"},
        "Breed": {"S": "Beagle"}
    }
}
```

### 1.3 Shape Serialization Rules

| Smithy Type | JSON Representation |
|-------------|---------------------|
| `blob` | Base64-encoded string |
| `boolean` | JSON boolean |
| `byte/short/integer/long` | JSON number |
| `float/double` | JSON number (or `"NaN"`, `"Infinity"`, `"-Infinity"` for special values) |
| `string` | JSON string |
| `timestamp` | JSON number (epoch-seconds as double) |
| `document` | Any JSON value |
| `list` | JSON array |
| `map` | JSON object |
| `structure` | JSON object with member-named properties |
| `union` | JSON object with single non-null member |

### 1.4 Operation Routing

Unlike S3 where operation routing is based on HTTP method + URL path + query parameters, DynamoDB routes **entirely** via the `X-Amz-Target` header value. The server parses `DynamoDB_20120810.<OperationName>` and dispatches to the corresponding handler. This is much simpler to implement than S3's complex routing.

---

## 2. Data Model

### 2.1 Core Concepts

| Concept | Description |
|---------|-------------|
| **Table** | Collection of items; has a name and primary key schema |
| **Item** | A group of attributes; max 400 KB; uniquely identified by primary key |
| **Attribute** | Name-value pair; can be scalar, set, or document (nested up to 32 levels) |
| **Primary Key** | Either partition key only (simple) or partition key + sort key (composite) |

### 2.2 Primary Key Types

**Simple Primary Key (Partition Key Only):**
- One attribute serves as the partition key
- Hash of partition key determines physical storage partition
- No two items can share the same partition key value

**Composite Primary Key (Partition Key + Sort Key):**
- Two attributes: partition key + sort key (range key)
- Multiple items can share the same partition key if sort keys differ
- Items with the same partition key are stored together, sorted by sort key
- Enables range queries within a partition

### 2.3 Attribute Types (10 Types)

DynamoDB uses type descriptors in JSON to represent typed values:

| Descriptor | Type | JSON Example | Notes |
|------------|------|-------------|-------|
| `S` | String | `{"S": "Hello"}` | UTF-8; constrained by 400 KB item limit |
| `N` | Number | `{"N": "123.45"}` | Up to 38 digits precision; transmitted as string to preserve precision |
| `B` | Binary | `{"B": "dGhpcyB0ZXN0"}` | Base64-encoded |
| `SS` | String Set | `{"SS": ["Red", "Blue"]}` | Non-empty, unique elements |
| `NS` | Number Set | `{"NS": ["1", "2.5"]}` | Elements as strings |
| `BS` | Binary Set | `{"BS": ["U3Vubnk=", "UmFpbnk="]}` | Elements base64-encoded |
| `L` | List | `{"L": [{"S": "Red"}, {"N": "5"}]}` | Ordered, heterogeneous |
| `M` | Map | `{"M": {"Name": {"S": "Joe"}}}` | Unordered key-value pairs |
| `BOOL` | Boolean | `{"BOOL": true}` | Native JSON boolean |
| `NULL` | Null | `{"NULL": true}` | Represents absence of value |

**Number type range:**
- Positive: `1E-130` to `9.9999999999999999999999999999999999999E+125`
- Negative: `-9.9999999999999999999999999999999999999E+125` to `-1E-130`

### 2.4 Secondary Indexes

**Local Secondary Indexes (LSI):**
- Same partition key as base table, different sort key
- Up to 5 per table
- Must be defined at table creation (cannot be added later)
- 10 GB limit per partition key value (item collection)

**Global Secondary Indexes (GSI):**
- Different partition key and/or sort key from base table
- Up to 20 per table (default quota)
- Can be added/removed after table creation
- Eventually consistent reads only
- Has its own provisioned throughput settings

**Projections (both LSI and GSI):**
- `KEYS_ONLY`: Only index key attributes + table primary key
- `INCLUDE`: Specified non-key attributes + key attributes
- `ALL`: All table attributes projected into index
- Maximum 100 user-specified projected attributes across all indexes

### 2.5 Capacity Modes

**Provisioned Mode (`PROVISIONED`):**
- You specify Read Capacity Units (RCU) and Write Capacity Units (WCU)
- 1 RCU = 1 strongly consistent read/sec or 2 eventually consistent reads/sec for items up to 4 KB
- 1 WCU = 1 write/sec for items up to 1 KB
- Transactional operations cost 2x capacity units
- Billed hourly for provisioned capacity

**On-Demand Mode (`PAY_PER_REQUEST`):**
- No capacity planning needed; auto-scales
- Billed per read/write request
- Same capacity unit ratios apply for billing
- Default and recommended mode

**Mode switching:**
- Provisioned to On-Demand: up to 4 times per 24-hour window
- On-Demand to Provisioned: any time

### 2.6 DynamoDB Streams

Optional feature for change data capture:

| StreamViewType | What's Captured |
|---------------|-----------------|
| `KEYS_ONLY` | Only primary key of modified item |
| `NEW_IMAGE` | Entire item after modification |
| `OLD_IMAGE` | Entire item before modification |
| `NEW_AND_OLD_IMAGES` | Both before and after images |

Stream records are retained for 24 hours.

### 2.7 Table Classes

| Class | Use Case |
|-------|----------|
| `STANDARD` | Default; general purpose |
| `STANDARD_INFREQUENT_ACCESS` | Lower storage cost for infrequently accessed data |

---

## 3. Complete API Operations (66 Operations)

### 3.1 Table Management (6)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `CreateTable` | Create table with key schema, indexes, billing mode, streams | P0 |
| `DeleteTable` | Delete a table | P0 |
| `DescribeTable` | Get table metadata (status, schema, indexes, throughput) | P0 |
| `ListTables` | List all table names in account | P0 |
| `UpdateTable` | Modify billing, throughput, GSIs, streams, class, TTL | P1 |
| `DescribeEndpoints` | Return DynamoDB endpoints for region | P2 |

### 3.2 Item Operations (4)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `GetItem` | Retrieve single item by primary key (consistent/eventually consistent) | P0 |
| `PutItem` | Create or replace item; supports condition expressions | P0 |
| `DeleteItem` | Delete single item by primary key; supports condition expressions | P0 |
| `UpdateItem` | Update item attributes using update expressions; supports condition expressions | P0 |

### 3.3 Query and Scan (2)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `Query` | Query items by partition key + optional sort key condition; supports filters, projections, pagination | P0 |
| `Scan` | Read all items in table/index; supports filters, projections, parallel scan, pagination | P0 |

### 3.4 Batch Operations (2)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `BatchGetItem` | Retrieve up to 100 items from one or more tables (16 MB max) | P0 |
| `BatchWriteItem` | Write/delete up to 25 items across tables (16 MB max) | P0 |

### 3.5 Transaction Operations (2)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `TransactGetItems` | Atomically retrieve up to 100 items (4 MB max) | P1 |
| `TransactWriteItems` | Atomically write/update/delete up to 100 items (4 MB max); supports idempotent `ClientRequestToken` | P1 |

### 3.6 PartiQL Operations (3)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `ExecuteStatement` | Execute single PartiQL SELECT/INSERT/UPDATE/DELETE statement | P1 |
| `BatchExecuteStatement` | Execute multiple PartiQL statements in one request | P2 |
| `ExecuteTransaction` | Execute PartiQL statements transactionally | P2 |

### 3.7 TTL Operations (2)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `DescribeTimeToLive` | Get TTL settings for a table | P1 |
| `UpdateTimeToLive` | Enable/disable TTL on a table (specify TTL attribute) | P1 |

### 3.8 Tag Operations (3)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `TagResource` | Add tags to a table or backup | P2 |
| `UntagResource` | Remove tags from a table or backup | P2 |
| `ListTagsOfResource` | Get tags on a table or backup | P2 |

### 3.9 Global Table Operations (Version 2019) (2)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `UpdateTable` (with `ReplicaUpdates`) | Add/remove replicas for global tables (V2019 uses UpdateTable) | P3 |
| `DescribeTable` | Shows replica information in response | P3 |

### 3.10 Global Table Operations (Legacy V2017) (4)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `CreateGlobalTable` | Create a V2017 global table (deprecated) | P3 |
| `DescribeGlobalTable` | Describe V2017 global table | P3 |
| `UpdateGlobalTable` | Update V2017 global table | P3 |
| `ListGlobalTables` | List V2017 global tables | P3 |

### 3.11 Global Table Settings (Legacy V2017) (2)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `DescribeGlobalTableSettings` | Get V2017 global table settings | P3 |
| `UpdateGlobalTableSettings` | Update V2017 global table settings | P3 |

### 3.12 Backup Operations (5)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `CreateBackup` | Create on-demand backup | P3 |
| `DeleteBackup` | Delete a backup | P3 |
| `DescribeBackup` | Get backup info | P3 |
| `ListBackups` | List backups | P3 |
| `RestoreTableFromBackup` | Restore table from backup | P3 |

### 3.13 Point-in-Time Recovery (3)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `DescribeContinuousBackups` | Describe PITR settings | P3 |
| `UpdateContinuousBackups` | Enable/disable PITR | P3 |
| `RestoreTableToPointInTime` | Restore table to specific point in time | P3 |

### 3.14 Kinesis Streaming (3)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `DescribeKinesisStreamingDestination` | Describe Kinesis streaming config | P3 |
| `EnableKinesisStreamingDestination` | Enable Kinesis data stream | P3 |
| `DisableKinesisStreamingDestination` | Disable Kinesis data stream | P3 |
| `UpdateKinesisStreamingDestination` | Update Kinesis streaming config | P3 |

### 3.15 Export/Import (5)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `ExportTableToPointInTime` | Export table to S3 | P3 |
| `DescribeExport` | Get export details | P3 |
| `ListExports` | List exports | P3 |
| `ImportTable` | Import data from S3 | P3 |
| `DescribeImport` | Get import details | P3 |
| `ListImports` | List imports | P3 |

### 3.16 Resource Policy Operations (3)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `GetResourcePolicy` | Get resource-based policy | P3 |
| `PutResourcePolicy` | Attach resource-based policy | P3 |
| `DeleteResourcePolicy` | Remove resource-based policy | P3 |

### 3.17 Auto Scaling (2)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `DescribeTableReplicaAutoScaling` | Describe replica auto-scaling | P3 |
| `UpdateTableReplicaAutoScaling` | Update replica auto-scaling | P3 |

### 3.18 Other Operations (3)

| Operation | Description | Priority |
|-----------|-------------|----------|
| `DescribeLimits` | Get account limits and usage | P2 |
| `DescribeContributorInsights` | Get contributor insights | P3 |
| `ListContributorInsights` | List contributor insights | P3 |
| `UpdateContributorInsights` | Update contributor insights | P3 |

---

## 4. Expression Language

DynamoDB has a rich expression language used across multiple operations. This is one of the most complex parts to implement.

### 4.1 Expression Types

| Expression Type | Used In | Purpose |
|----------------|---------|---------|
| **KeyConditionExpression** | `Query` | Filter by primary key |
| **FilterExpression** | `Query`, `Scan` | Post-retrieval filtering |
| **ProjectionExpression** | `GetItem`, `Query`, `Scan`, `BatchGetItem` | Select which attributes to return |
| **ConditionExpression** | `PutItem`, `UpdateItem`, `DeleteItem`, `TransactWriteItems` | Conditional writes |
| **UpdateExpression** | `UpdateItem` | Specify attribute modifications |

### 4.2 Condition/Filter Expression Grammar

```
condition-expression ::=
    operand comparator operand
  | operand BETWEEN operand AND operand
  | operand IN ( operand (',' operand)* )
  | function
  | condition AND condition
  | condition OR condition
  | NOT condition
  | ( condition )

comparator ::= = | <> | < | <= | > | >=
```

### 4.3 Built-in Functions

| Function | Description |
|----------|-------------|
| `attribute_exists(path)` | True if attribute exists |
| `attribute_not_exists(path)` | True if attribute does not exist |
| `attribute_type(path, type)` | True if attribute is of specified type (S, N, B, SS, NS, BS, BOOL, NULL, L, M) |
| `begins_with(path, substr)` | True if string/binary attribute begins with substring |
| `contains(path, operand)` | True if string contains substring, or set/list contains element |
| `size(path)` | Returns size: string length, binary bytes, set/list element count |

### 4.4 Update Expression Grammar

```
update-expression ::=
    [ SET action [, action] ... ]
    [ REMOVE action [, action] ... ]
    [ ADD action [, action] ... ]
    [ DELETE action [, action] ... ]
```

**SET clause:**
```
set-action ::= path = value
value      ::= operand | operand '+' operand | operand '-' operand
operand    ::= path | function
function   ::= if_not_exists(path, value) | list_append(list1, list2)
```

- `if_not_exists(path, value)`: Use `value` only if attribute does not exist
- `list_append(list1, list2)`: Concatenate two lists (can prepend by swapping argument order)

**REMOVE clause:**
```
remove-action ::= path
```
Deletes attributes (or list elements by index).

**ADD clause:**
```
add-action ::= path value
```
Only for Number and Set types. Adds to number or appends to set. Creates attribute if it does not exist.

**DELETE clause:**
```
delete-action ::= path subset
```
Only for Set types. Removes elements from a set.

**Important**: All attribute references resolve against the item's state **before** any actions are applied.

### 4.5 Expression Substitution

- **Expression Attribute Names** (`#name`): Substitute attribute names that are reserved words or contain special characters. Provided as `ExpressionAttributeNames` map.
- **Expression Attribute Values** (`:val`): Substitute typed values. Provided as `ExpressionAttributeValues` map.

### 4.6 Expression Limits

| Limit | Value |
|-------|-------|
| Expression string max length | 4 KB |
| Single attribute name/value max | 255 bytes |
| All substitution variables combined | 2 MB |
| UpdateExpression operators/functions max | 300 |
| IN comparator operands max | 100 |

---

## 5. PartiQL Support

DynamoDB supports a subset of PartiQL (SQL-compatible query language) as an alternative to the native expression-based API.

### 5.1 Supported Statements

**SELECT:**
```sql
SELECT expression [, ...]
FROM table[.index]
[ WHERE condition ]
[ ORDER BY key [DESC|ASC] ]
```

**INSERT:**
```sql
INSERT INTO table VALUE {
    'pk': 'value',
    'sk': 'value',
    'attr': 'value'
}
```
Equivalent to PutItem.

**UPDATE:**
```sql
UPDATE table
SET path = value [, ...]
[REMOVE path [, ...]]
WHERE condition
[RETURNING returnvalues]
```
Can only update one item at a time.

**DELETE:**
```sql
DELETE FROM table WHERE condition
```
Can only delete one item at a time.

### 5.2 PartiQL API Operations

| Operation | Description |
|-----------|-------------|
| `ExecuteStatement` | Execute single statement (Parameters use `?` placeholders) |
| `BatchExecuteStatement` | Execute multiple statements (up to 25) |
| `ExecuteTransaction` | Execute statements transactionally (up to 100) |

---

## 6. Error Handling

### 6.1 Error Response Format

DynamoDB errors are returned as JSON with a specific format:

```json
{
    "__type": "com.amazonaws.dynamodb.v20120810#ResourceNotFoundException",
    "message": "Requested resource not found: Table: tablename not found"
}
```

**Error type identification** (clients must handle all three):
1. `X-Amzn-Errortype` header
2. `code` field in response body
3. `__type` field in response body (contains full Shape ID)

Clients sanitize by: removing content after `:` and extracting content after `#`.

### 6.2 Error Codes

**HTTP 400 (Client Errors):**

| Exception | Retryable? |
|-----------|-----------|
| `AccessDeniedException` | No |
| `ConditionalCheckFailedException` | No |
| `IncompleteSignatureException` | No |
| `ItemCollectionSizeLimitExceededException` | Yes |
| `LimitExceededException` | Yes |
| `MissingAuthenticationTokenException` | No |
| `ProvisionedThroughputExceededException` | Yes |
| `ReplicatedWriteConflictException` | Yes |
| `RequestLimitExceeded` | Yes |
| `ResourceInUseException` | No |
| `ResourceNotFoundException` | No |
| `ThrottlingException` | Yes |
| `UnrecognizedClientException` | Yes |
| `ValidationException` | No |
| `TransactionCanceledException` | No |
| `TransactionConflictException` | Yes |
| `TransactionInProgressException` | Yes |
| `IdempotentParameterMismatchException` | No |

**HTTP 5xx (Server Errors):**

| Exception | HTTP Code | Retryable? |
|-----------|-----------|-----------|
| `InternalServerError` | 500 | Yes |
| `ServiceUnavailable` | 503 | Yes |

### 6.3 Batch Operation Error Handling

- `BatchGetItem`: Unprocessed items returned in `UnprocessedKeys`
- `BatchWriteItem`: Unprocessed items returned in `UnprocessedItems`
- Client should retry unprocessed items with exponential backoff

---

## 7. DynamoDB Streams (Separate Service)

DynamoDB Streams is a **separate service** with its own endpoint and X-Amz-Target prefix.

### 7.1 Endpoint

| Property | Value |
|----------|-------|
| Endpoint (IPv4) | `streams.dynamodb.<region>.amazonaws.com` |
| Endpoint (dual-stack) | `streams-dynamodb.<region>.api.aws` |
| X-Amz-Target prefix | `DynamoDBStreams_20120810.<OperationName>` |

### 7.2 Operations (4)

| Operation | Description |
|-----------|-------------|
| `DescribeStream` | Get stream metadata and shard information |
| `GetRecords` | Read records from a shard (up to 1 MB or 1000 records) |
| `GetShardIterator` | Get iterator for reading from a shard |
| `ListStreams` | List all streams for an account or table |

### 7.3 Shard Iterator Types

| Type | Description |
|------|-------------|
| `TRIM_HORIZON` | Start at the oldest record in the shard |
| `LATEST` | Start at the most recent record |
| `AT_SEQUENCE_NUMBER` | Start at a specific sequence number |
| `AFTER_SEQUENCE_NUMBER` | Start after a specific sequence number |

---

## 8. Constraints and Limits

### 8.1 Naming

| Resource | Constraint |
|----------|-----------|
| Table/Index names | 3-255 chars; `[A-Za-z0-9_.-]` |
| Attribute names | 1 char to 64 KB; key attributes limited to 255 bytes |

### 8.2 Key Sizes

| Key | Min | Max |
|-----|-----|-----|
| Partition key | 1 byte | 2,048 bytes |
| Sort key | 1 byte | 1,024 bytes |

### 8.3 Item and Request Limits

| Limit | Value |
|-------|-------|
| Max item size | 400 KB (including attribute names) |
| Max nesting depth | 32 levels |
| BatchGetItem items | 100 items, 16 MB |
| BatchWriteItem items | 25 PutItem/DeleteItem, 16 MB |
| Query/Scan result set | 1 MB per call |
| Transaction items | 100 unique items, 4 MB |
| GSI per table | 20 (default) |
| LSI per table | 5 |
| LSI collection size | 10 GB per partition key value |
| Projected attributes | 100 total across all indexes (INCLUDE only) |

### 8.4 API Rate Limits

| Operation | Limit |
|-----------|-------|
| CreateTable/UpdateTable/DeleteTable (concurrent) | 500 |
| Mutable control plane requests | 2,500/sec |
| Read-only control plane requests | 2,500/sec |
| DescribeLimits | 1/minute |
| UpdateTimeToLive | 1/table/hour |

---

## 9. Smithy Model and Code Generation

### 9.1 Smithy Model Location

The DynamoDB Smithy model is available from multiple sources:

1. **aws-models repository** (recommended): `https://github.com/aws/api-models-aws`
   - Path: `models/dynamodb/<version>/dynamodb-<version>.json`
   - Format: JSON AST (machine-readable Smithy)

2. **aws-sdk-rust repository**: `https://github.com/awslabs/aws-sdk-rust`
   - Contains generated Rust code from the Smithy model

3. **smithy-rs repository**: `https://github.com/smithy-lang/smithy-rs`
   - Contains the code generator and runtime crates

### 9.2 Protocol Trait

DynamoDB uses `@awsJson1_0` protocol:

```smithy
@awsJson1_0
service DynamoDB {
    version: "2012-08-10",
    operations: [...]
}
```

This is already supported by smithy-rs server codegen (see `smithy-rs-server-codegen-research.md`):

| Protocol | Marker Type | DynamoDB? |
|----------|-------------|-----------|
| `@restJson1` | `RestJson1` | No |
| `@restXml` | `RestXml` | No (S3 uses this) |
| `@awsJson1_0` | `AwsJson10` | **Yes** |
| `@awsJson1_1` | `AwsJson11` | No |
| `@rpcv2Cbor` | `RpcV2Cbor` | No |

### 9.3 Code Generation Approach

The existing RustStack S3 implementation uses a custom codegen approach (see `/codegen/`) that generates Rust types from the S3 Smithy model JSON AST. For DynamoDB, we have two options:

**Option A: Extend the existing custom codegen** to handle `@awsJson1_0` protocol models. Since DynamoDB uses JSON (not XML), the serialization/deserialization is simpler than S3.

**Option B: Use smithy-rs server codegen** which already supports `@awsJson1_0` and would generate routing, (de)serialization, and handler traits automatically. Requires JDK 17+ and Gradle build step.

**Recommendation**: Option A is likely better for consistency with the existing S3 approach and to avoid the JDK/Gradle dependency. JSON serde is much simpler than XML serde, and we can leverage the `serde` + `serde_json` ecosystem directly. The DynamoDB model's `@awsJson1_0` protocol means all input/output types are simple JSON objects with serde-derived serialization, unlike S3's complex XML with namespace handling.

### 9.4 Key Smithy Model Shapes

The DynamoDB model defines these important shapes (among hundreds):

**Service:** `com.amazonaws.dynamodb#DynamoDB_20120810`

**Core input/output shapes** (examples):
- `CreateTableInput` / `CreateTableOutput`
- `PutItemInput` / `PutItemOutput`
- `GetItemInput` / `GetItemOutput`
- `QueryInput` / `QueryOutput`
- `ScanInput` / `ScanOutput`

**Key type shapes:**
- `AttributeValue` (tagged union: S, N, B, SS, NS, BS, L, M, BOOL, NULL)
- `KeySchemaElement` (AttributeName + KeyType)
- `AttributeDefinition` (AttributeName + AttributeType)
- `Projection` (ProjectionType + NonKeyAttributes)
- `ProvisionedThroughput` / `ProvisionedThroughputDescription`
- `GlobalSecondaryIndex` / `GlobalSecondaryIndexDescription`
- `LocalSecondaryIndex` / `LocalSecondaryIndexDescription`
- `StreamSpecification`
- `TableDescription`

---

## 10. Key Differences from S3

| Aspect | S3 | DynamoDB |
|--------|-----|----------|
| **Protocol** | REST (`@restXml`) | RPC (`@awsJson1_0`) |
| **HTTP Method** | GET, PUT, POST, DELETE, HEAD | POST only |
| **Routing** | URL path + query + method | `X-Amz-Target` header |
| **Serialization** | XML (with namespaces) | JSON |
| **Content-Type** | `application/xml` (mostly) | `application/x-amz-json-1.0` |
| **URL Pattern** | `/<bucket>/<key>` or virtual host | `/` (always root) |
| **Auth** | SigV4 (and SigV2 legacy) | SigV4 only |
| **Request body** | XML for some, binary for objects | JSON always |
| **Response body** | XML for metadata, binary for objects | JSON always |
| **Streaming** | Yes (object GET/PUT) | No (all in-memory JSON) |
| **Checksums** | MD5, CRC32, SHA256 | CRC32 response header |
| **Pagination** | Marker/ContinuationToken | `ExclusiveStartKey` / `LastEvaluatedKey` |
| **Data model** | Flat (buckets contain objects) | Hierarchical (tables, items with nested attributes) |
| **Schema** | Schemaless blobs | Semi-structured (key schema defined, attributes flexible) |
| **Max object/item** | 5 TB (multipart) | 400 KB |

### 10.1 Implications for RustStack Architecture

**Shared components (`ruststack-core`, `ruststack-auth`):**
- Account/region state management: reusable as-is
- SigV4 authentication: reusable as-is (DynamoDB uses only SigV4, no SigV2)
- SigV2 support: not needed for DynamoDB

**New DynamoDB-specific crates needed:**
- `ruststack-dynamodb-model`: Generated types from Smithy model
- `ruststack-dynamodb-core`: Business logic (table management, item storage, indexes, expressions)
- `ruststack-dynamodb-http`: HTTP routing, JSON codec, service layer
- No XML crate needed (unlike `ruststack-s3-xml`)

**Simpler aspects vs S3:**
- No virtual-host vs path-style routing ambiguity
- No multipart upload complexity
- No streaming (all requests/responses are JSON in memory)
- No CORS, website hosting, lifecycle policies
- Simpler routing (header-based dispatch)

**More complex aspects vs S3:**
- Expression language parser and evaluator (condition, filter, update, projection, key condition)
- PartiQL parser and evaluator
- Secondary index maintenance (automatic index updates on item writes)
- Transaction support (ACID guarantees for up to 100 items)
- Type system (10 attribute types vs S3's opaque blobs)

---

## 11. LocalStack DynamoDB Support

### 11.1 Architecture

LocalStack's DynamoDB emulation is powered by **DynamoDB Local** (AWS's official Java-based emulator) running under the hood. LocalStack adds features on top:

- Global Tables V2019 with cross-region replication
- TTL worker (runs every 60 minutes, scans and deletes expired items)
- Persistence across container restarts
- Cloud Pods for snapshotting
- Integration with DynamoDB Streams and Kinesis Data Streams
- Resource Browser in Web Application

### 11.2 Configuration

| Environment Variable | Description |
|---------------------|-------------|
| `DYNAMODB_IN_MEMORY=1` | Run in memory for performance (no persistence) |
| `DYNAMODB_REMOVE_EXPIRED_ITEMS` | Enable TTL expiration worker |

Manual expiration trigger: `DELETE /_aws/dynamodb/expired`

### 11.3 LocalStack Limitations

- Cannot remove original region from replicas without deleting all tables
- DynamoDB Streams only work on original tables, not replicas
- Batch operations unsupported for replicated tables
- V2017 global tables not supported (no replication)

---

## 12. DynamoDB Local (AWS Official) Limitations

DynamoDB Local is the Java emulator that LocalStack wraps. Key limitations:

| Feature | DynamoDB Service | DynamoDB Local |
|---------|-----------------|----------------|
| PITR | Yes | No |
| Tagging | Yes | No |
| Streams shard behavior | Partition-aware | No partitioning |
| Transaction conflicts | `TransactionConflictException` | Not thrown |
| Parallel scan | Yes | No (sequential only) |
| Billing summary | Returns data | Always returns `null` |
| Item collection metrics | Tracked | Not tracked (returns null) |
| Table case sensitivity | Case-sensitive | **Case-insensitive** |
| Capacity limits | Enforced | Not enforced |
| ExecuteStatement Limit | Enforced | **Ignored** |

These limitations represent opportunities for a Rust-based implementation to provide better fidelity.

---

## 13. Existing Implementations

### 13.1 DynamoDB Local (AWS Official)

- **Language:** Java
- **Storage:** SQLite-based
- **Distribution:** JAR download or Docker image
- **Pros:** Official, widely used, good API compatibility
- **Cons:** JVM startup overhead, missing features (see section 12), closed source

### 13.2 ScyllaDB Alternator

- **Language:** C++
- **Storage:** ScyllaDB (distributed NoSQL)
- **Architecture:** Parses DynamoDB JSON requests directly, calls internal ScyllaDB C++ functions (no CQL generation)
- **Supported:** Core CRUD, Query, Scan, Batch, GSI, LSI, TTL, Tags, Streams (experimental)
- **Not supported:** Multi-item transactions (TransactGetItems/TransactWriteItems), PartiQL, Backup/Restore, Kinesis integration, Export/Import
- **Consistency:** LOCAL_QUORUM for writes and strong reads; LOCAL_ONE for eventual reads

### 13.3 Dynalite (Node.js)

- **Language:** JavaScript/Node.js
- **Storage:** LevelDB (or in-memory)
- **Architecture:** HTTP server, aims for exact DynamoDB behavioral parity including error messages and limits
- **Pros:** Fast startup (milliseconds), no JVM, in-memory option
- **Cons:** Node.js dependency, maintenance status uncertain

### 13.4 Rust Ecosystem (Clients Only)

There are **no existing Rust-based DynamoDB server implementations**. Existing Rust crates are all clients:

| Crate | Type | Status |
|-------|------|--------|
| `aws-sdk-dynamodb` | Official AWS SDK client | Active |
| `dynomite` | DynamoDB ORM/mapper | Archived |
| `modyne` | Single-table design helper | Active |
| `dynamodb-tools` | Test helper for DynamoDB Local | Active |
| `rusoto_dynamodb` | Legacy AWS SDK client | Deprecated |

---

## 14. Implementation Priority Matrix

### P0: Core (MVP for basic functionality)

These operations cover 90%+ of typical local development use cases:

| Category | Operations | Complexity |
|----------|-----------|------------|
| Table CRUD | CreateTable, DeleteTable, DescribeTable, ListTables | Medium |
| Item CRUD | PutItem, GetItem, UpdateItem, DeleteItem | High (expressions) |
| Query/Scan | Query, Scan | High (expressions, pagination, indexes) |
| Batch | BatchGetItem, BatchWriteItem | Medium |
| **Total P0** | **12 operations** | |

### P1: Important (common in production code)

| Category | Operations | Complexity |
|----------|-----------|------------|
| Table Update | UpdateTable | Medium |
| Transactions | TransactGetItems, TransactWriteItems | High |
| TTL | DescribeTimeToLive, UpdateTimeToLive | Low |
| PartiQL (single) | ExecuteStatement | High (parser) |
| **Total P1** | **5 operations** | |

### P2: Nice to Have

| Category | Operations | Complexity |
|----------|-----------|------------|
| Tags | TagResource, UntagResource, ListTagsOfResource | Low |
| Limits | DescribeLimits, DescribeEndpoints | Low |
| PartiQL (batch) | BatchExecuteStatement, ExecuteTransaction | Medium |
| **Total P2** | **6 operations** | |

### P3: Low Priority (rarely needed locally)

Everything else: Backups, PITR, Global Tables, Kinesis, Export/Import, Resource Policies, Auto Scaling, Contributor Insights. These total ~35 operations but are almost never used in local development.

---

## 15. Architecture Considerations for RustStack

### 15.1 Proposed Crate Structure

```
ruststack-dynamodb-model/     # Generated types from Smithy model (JSON serde)
ruststack-dynamodb-core/      # Business logic
  - table/                    # Table management, schema validation
  - item/                     # Item storage, type validation
  - index/                    # GSI/LSI maintenance and querying
  - expression/               # Expression parser and evaluator
    - condition.rs            # Condition/filter expression evaluation
    - update.rs               # Update expression evaluation
    - projection.rs           # Projection expression evaluation
    - key_condition.rs        # Key condition expression evaluation
    - parser.rs               # Expression tokenizer/parser
  - query/                    # Query engine
  - scan/                     # Scan engine
  - batch/                    # Batch operations
  - transaction/              # Transaction support
  - ttl/                      # TTL management
ruststack-dynamodb-http/      # HTTP layer
  - router.rs                 # X-Amz-Target dispatch (much simpler than S3)
  - codec.rs                  # JSON codec (serde_json, trivial vs XML)
  - service.rs                # Tower service
```

### 15.2 Storage Design

For a local emulator, item storage needs to support:

1. **Primary key lookup** (GetItem, DeleteItem): Hash map by partition key, then B-tree by sort key
2. **Query by partition key + sort key range**: Ordered storage within a partition
3. **Scan**: Full table iteration with optional filters
4. **GSI/LSI**: Separate indexes maintained on write operations
5. **Atomic transactions**: Multi-item atomicity guarantees

**Suggested in-memory approach:**
```
Table {
    items: BTreeMap<PartitionKey, BTreeMap<SortKey, Item>>  // or HashMap + BTreeMap
    gsi: HashMap<IndexName, BTreeMap<GSIPartitionKey, BTreeMap<GSISortKey, ProjectedItem>>>
    lsi: HashMap<IndexName, BTreeMap<PartitionKey, BTreeMap<LSISortKey, ProjectedItem>>>
}
```

### 15.3 Expression Engine

The expression language is the most complex component. Key considerations:

1. **Lexer/Parser**: Tokenize expression strings into AST nodes
2. **Evaluator**: Walk AST and evaluate against item attributes
3. **Type coercion**: Handle DynamoDB's typed comparisons (N vs S vs B)
4. **Path resolution**: Navigate nested Map/List attributes (e.g., `Address.City`, `Colors[0]`)
5. **Substitution**: Replace `#name` and `:val` placeholders before evaluation

Consider using a parser combinator library like `winnow` or `nom` for expression parsing.

### 15.4 Key Implementation Challenges

1. **Expression language**: Full parser + evaluator for 5 expression types with nested attribute paths, functions, and arithmetic
2. **PartiQL**: SQL-like parser (could defer to P2 or use an existing parser library)
3. **Index maintenance**: Every PutItem/UpdateItem/DeleteItem must update all GSI/LSI entries atomically
4. **Transaction isolation**: Ensuring ACID for TransactWriteItems across multiple tables
5. **Type validation**: Enforcing DynamoDB's type system (400 KB limit, 32-level nesting, number precision)
6. **Pagination**: Consistent pagination with `ExclusiveStartKey`/`LastEvaluatedKey` across Query and Scan

### 15.5 Advantages of Rust Implementation Over DynamoDB Local

| Aspect | DynamoDB Local (Java) | RustStack (Rust) |
|--------|----------------------|------------------|
| Startup time | Seconds (JVM warmup) | Milliseconds |
| Memory usage | High (JVM overhead) | Low |
| Binary size | Large (JAR + JVM) | Small (single binary) |
| Docker image | ~600 MB (Java) | ~10 MB (from scratch) |
| Feature fidelity | Missing many features | Can implement fully |
| Case sensitivity | Bug: case-insensitive tables | Correct: case-sensitive |
| Parallel scan | Not supported | Can implement |
| Transaction conflicts | Not thrown | Can implement |

---

## Sources

- [AWS DynamoDB API Reference - All Operations](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_Operations_Amazon_DynamoDB.html)
- [AWS DynamoDB Low-Level API Protocol](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.LowLevelAPI.html)
- [AWS DynamoDB Core Components](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html)
- [AWS DynamoDB Constraints](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Constraints.html)
- [AWS DynamoDB Error Handling](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html)
- [AWS DynamoDB Expressions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.html)
- [AWS DynamoDB Update Expressions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.UpdateExpressions.html)
- [AWS DynamoDB Condition/Filter Expressions](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.OperatorsAndFunctions.html)
- [AWS DynamoDB Capacity Modes](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/capacity-mode.html)
- [AWS DynamoDB Streams](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Streams.html)
- [AWS DynamoDB Local Usage Notes](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.UsageNotes.html)
- [AWS DynamoDB PartiQL](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ql-reference.statements.html)
- [Smithy AWS JSON 1.0 Protocol Spec](https://smithy.io/2.0/aws/protocols/aws-json-1_0-protocol.html)
- [AWS API Models Repository](https://github.com/aws/api-models-aws)
- [smithy-rs Repository](https://github.com/smithy-lang/smithy-rs)
- [LocalStack DynamoDB Documentation](https://docs.localstack.cloud/aws/services/dynamodb/)
- [ScyllaDB Alternator Compatibility](https://github.com/scylladb/scylladb/blob/master/docs/alternator/compatibility.md)
- [Dynalite (Node.js DynamoDB Emulator)](https://github.com/architect/dynalite)
- [DynamoDB Endpoints and Quotas](https://docs.aws.amazon.com/general/latest/gr/ddb.html)
- [CreateTable API Reference](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CreateTable.html)
- [TransactWriteItems API Reference](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_TransactWriteItems.html)
