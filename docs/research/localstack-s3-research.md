# LocalStack S3 Implementation Research

**Date:** 2026-02-26
**Purpose:** Comprehensive analysis of what a Rust-based LocalStack S3 service needs to support for feature parity.

---

## Table of Contents

1. [AWS S3 API Operations (Complete List)](#1-aws-s3-api-operations-complete-list)
2. [LocalStack S3 Architecture](#2-localstack-s3-architecture)
3. [Key S3 Operations (Detailed)](#3-key-s3-operations-detailed)
4. [S3 Features Supported by LocalStack](#4-s3-features-supported-by-localstack)
5. [Local Storage Implementation](#5-local-storage-implementation)
6. [S3 API XML Format Reference](#6-s3-api-xml-format-reference)
7. [s3s Crate Analysis (Rust Foundation)](#7-s3s-crate-analysis-rust-foundation)
8. [S3 Error Codes Reference](#8-s3-error-codes-reference)
9. [Implementation Priority Matrix](#9-implementation-priority-matrix)
10. [Feature Parity Checklist](#10-feature-parity-checklist)

---

## 1. AWS S3 API Operations (Complete List)

AWS S3 defines **111 REST API operations** as of 2025. The s3s Rust crate covers **96 operations** via its S3 trait.

### 1.1 Complete AWS S3 Operation List (111 operations)

#### Bucket CRUD (4)

| Operation | HTTP Method | s3s? | LocalStack? |
|-----------|-------------|------|-------------|
| CreateBucket | PUT / | Yes | Yes |
| DeleteBucket | DELETE / | Yes | Yes |
| HeadBucket | HEAD / | Yes | Yes |
| ListBuckets | GET / | Yes | Yes |

#### Object CRUD (8)

| Operation | HTTP Method | s3s? | LocalStack? |
|-----------|-------------|------|-------------|
| PutObject | PUT /{Key+} | Yes | Yes |
| GetObject | GET /{Key+} | Yes | Yes |
| HeadObject | HEAD /{Key+} | Yes | Yes |
| DeleteObject | DELETE /{Key+} | Yes | Yes |
| DeleteObjects | POST /?delete | Yes | Yes |
| CopyObject | PUT /{Key+} (x-amz-copy-source) | Yes | Yes |
| RenameObject | PUT /{Key+}?rename | Yes | No |
| RestoreObject | POST /{Key+}?restore | Yes | Partial |

#### Multipart Upload (7)

| Operation | HTTP Method | s3s? | LocalStack? |
|-----------|-------------|------|-------------|
| CreateMultipartUpload | POST /{Key+}?uploads | Yes | Yes |
| UploadPart | PUT /{Key+}?partNumber&uploadId | Yes | Yes |
| UploadPartCopy | PUT /{Key+}?partNumber&uploadId (x-amz-copy-source) | Yes | Yes |
| CompleteMultipartUpload | POST /{Key+}?uploadId | Yes | Yes |
| AbortMultipartUpload | DELETE /{Key+}?uploadId | Yes | Yes |
| ListParts | GET /{Key+}?uploadId | Yes | Yes |
| ListMultipartUploads | GET /?uploads | Yes | Yes |

#### List Operations (5)

| Operation | HTTP Method | s3s? | LocalStack? |
|-----------|-------------|------|-------------|
| ListObjects | GET / | Yes | Yes |
| ListObjectsV2 | GET /?list-type=2 | Yes | Yes |
| ListObjectVersions | GET /?versions | Yes | Yes |
| ListDirectoryBuckets | GET / (directory) | No | No |
| ListBuckets | GET / | Yes | Yes |

#### Bucket Configuration (52)

| Operation | s3s? | LocalStack? | Notes |
|-----------|------|-------------|-------|
| GetBucketLocation | Yes | Yes | |
| PutBucketVersioning | Yes | Yes | |
| GetBucketVersioning | Yes | Yes | |
| PutBucketEncryption | Yes | Yes | Stores config, no actual encryption |
| GetBucketEncryption | Yes | Yes | |
| DeleteBucketEncryption | Yes | Yes | |
| PutBucketTagging | Yes | Yes | |
| GetBucketTagging | Yes | Yes | |
| DeleteBucketTagging | Yes | Yes | |
| PutBucketCors | Yes | Yes | |
| GetBucketCors | Yes | Yes | |
| DeleteBucketCors | Yes | Yes | |
| PutBucketPolicy | Yes | Yes | |
| GetBucketPolicy | Yes | Yes | |
| DeleteBucketPolicy | Yes | Yes | |
| GetBucketPolicyStatus | Yes | Yes | |
| PutBucketLifecycleConfiguration | Yes | Yes | Stores config, limited execution |
| GetBucketLifecycleConfiguration | Yes | Yes | |
| DeleteBucketLifecycle | Yes | Yes | |
| PutBucketNotificationConfiguration | Yes | Yes | |
| GetBucketNotificationConfiguration | Yes | Yes | |
| PutBucketLogging | Yes | Yes | |
| GetBucketLogging | Yes | Yes | |
| PutBucketAcl | Yes | Yes | |
| GetBucketAcl | Yes | Yes | |
| PutPublicAccessBlock | Yes | Yes | |
| GetPublicAccessBlock | Yes | Yes | |
| DeletePublicAccessBlock | Yes | Yes | |
| PutBucketOwnershipControls | Yes | Yes | |
| GetBucketOwnershipControls | Yes | Yes | |
| DeleteBucketOwnershipControls | Yes | Yes | |
| PutBucketAccelerateConfiguration | Yes | Yes | Stores config only |
| GetBucketAccelerateConfiguration | Yes | Yes | |
| PutBucketRequestPayment | Yes | Yes | |
| GetBucketRequestPayment | Yes | Yes | |
| PutBucketWebsite | Yes | Yes | |
| GetBucketWebsite | Yes | Yes | |
| DeleteBucketWebsite | Yes | Yes | |
| PutBucketReplication | Yes | Yes | Stores config only |
| GetBucketReplication | Yes | Yes | |
| DeleteBucketReplication | Yes | Yes | |
| PutBucketIntelligentTieringConfiguration | Yes | Partial | |
| GetBucketIntelligentTieringConfiguration | Yes | Partial | |
| DeleteBucketIntelligentTieringConfiguration | Yes | Partial | |
| PutBucketAnalyticsConfiguration | Yes | Partial | |
| GetBucketAnalyticsConfiguration | Yes | Partial | |
| DeleteBucketAnalyticsConfiguration | Yes | Partial | |
| ListBucketAnalyticsConfigurations | Yes | Partial | |
| PutBucketMetricsConfiguration | Yes | Partial | |
| GetBucketMetricsConfiguration | Yes | Partial | |
| DeleteBucketMetricsConfiguration | Yes | Partial | |
| ListBucketMetricsConfigurations | Yes | Partial | |
| PutBucketInventoryConfiguration | Yes | Partial | |
| GetBucketInventoryConfiguration | Yes | Partial | |
| DeleteBucketInventoryConfiguration | Yes | Partial | |
| ListBucketInventoryConfigurations | Yes | Partial | |

#### Object Configuration (14)

| Operation | s3s? | LocalStack? | Notes |
|-----------|------|-------------|-------|
| PutObjectTagging | Yes | Yes | |
| GetObjectTagging | Yes | Yes | |
| DeleteObjectTagging | Yes | Yes | |
| PutObjectAcl | Yes | Yes | |
| GetObjectAcl | Yes | Yes | |
| GetObjectAttributes | Yes | Yes | |
| PutObjectLockConfiguration | Yes | Yes | |
| GetObjectLockConfiguration | Yes | Yes | |
| PutObjectRetention | Yes | Yes | |
| GetObjectRetention | Yes | Yes | |
| PutObjectLegalHold | Yes | Yes | |
| GetObjectLegalHold | Yes | Yes | |
| GetObjectTorrent | Yes | No | |
| SelectObjectContent | Yes | No | |

#### Misc (6)

| Operation | s3s? | LocalStack? | Notes |
|-----------|------|-------------|-------|
| WriteGetObjectResponse | Yes | No | Object Lambda |
| CreateSession | No | No | Express One Zone |
| PostObject | No (HTML form) | Yes | Browser upload |
| GetBucketNotification (legacy) | No | Yes | |
| PutBucketNotification (legacy) | No | Yes | |
| PutBucketLifecycle (legacy) | No | Yes | |

#### Metadata Table Operations (Not in s3s, new 2024-2025)

| Operation | s3s? | LocalStack? |
|-----------|------|-------------|
| CreateBucketMetadataTableConfiguration | Yes | No |
| GetBucketMetadataTableConfiguration | Yes | No |
| DeleteBucketMetadataTableConfiguration | Yes | No |
| UpdateBucketMetadataInventoryTableConfiguration | No | No |
| UpdateBucketMetadataJournalTableConfiguration | No | No |

#### Other new operations (Not in s3s)

| Operation | s3s? | LocalStack? |
|-----------|------|-------------|
| GetBucketAbac | No | No |
| PutBucketAbac | No | No |
| CreateBucketMetadataConfiguration | No | No |
| GetBucketMetadataConfiguration | No | No |
| DeleteBucketMetadataConfiguration | No | No |
| UpdateObjectEncryption | No | No |

### 1.2 s3s S3 Trait (96 operations)

The complete list of methods in the s3s `S3` trait:

```
abort_multipart_upload         get_bucket_website              put_bucket_cors
complete_multipart_upload      get_object                      put_bucket_encryption
copy_object                    get_object_acl                  put_bucket_intelligent_tiering_configuration
create_bucket                  get_object_attributes           put_bucket_inventory_configuration
create_bucket_metadata_table_configuration  get_object_legal_hold  put_bucket_lifecycle_configuration
create_multipart_upload        get_object_lock_configuration   put_bucket_logging
delete_bucket                  get_object_retention            put_bucket_metrics_configuration
delete_bucket_analytics_configuration  get_object_tagging      put_bucket_notification_configuration
delete_bucket_cors             get_object_torrent              put_bucket_ownership_controls
delete_bucket_encryption       get_public_access_block         put_bucket_policy
delete_bucket_intelligent_tiering_configuration  head_bucket   put_bucket_replication
delete_bucket_inventory_configuration  head_object              put_bucket_request_payment
delete_bucket_lifecycle        list_bucket_analytics_configurations  put_bucket_tagging
delete_bucket_metadata_table_configuration  list_bucket_intelligent_tiering_configurations  put_bucket_versioning
delete_bucket_metrics_configuration  list_bucket_inventory_configurations  put_bucket_website
delete_bucket_ownership_controls  list_bucket_metrics_configurations  put_object
delete_bucket_policy           list_buckets                    put_object_acl
delete_bucket_replication      list_multipart_uploads          put_object_legal_hold
delete_bucket_tagging          list_object_versions            put_object_lock_configuration
delete_bucket_website          list_objects                    put_object_retention
delete_object                  list_objects_v2                 put_object_tagging
delete_object_tagging          list_parts                      put_public_access_block
delete_objects                 put_bucket_accelerate_configuration  restore_object
delete_public_access_block     put_bucket_acl                  select_object_content
get_bucket_accelerate_configuration  put_bucket_analytics_configuration  upload_part
get_bucket_acl                                                 upload_part_copy
get_bucket_analytics_configuration                             write_get_object_response
get_bucket_cors
get_bucket_encryption
get_bucket_intelligent_tiering_configuration
get_bucket_inventory_configuration
get_bucket_lifecycle_configuration
get_bucket_location
get_bucket_logging
get_bucket_metadata_table_configuration
get_bucket_metrics_configuration
get_bucket_notification_configuration
get_bucket_ownership_controls
get_bucket_policy
get_bucket_policy_status
get_bucket_replication
get_bucket_request_payment
get_bucket_tagging
get_bucket_versioning
```

---

## 2. LocalStack S3 Architecture

### 2.1 Codebase Structure

LocalStack's S3 implementation comprises approximately **11,500 lines of Python** across these components:

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| Provider (business logic) | `provider.py` | 5,072 | Main S3Api implementation with all handler methods |
| Data models | `models.py` | 818 | S3Bucket, S3Object, S3Part, KeyStore, VersionedKeyStore |
| Utilities | `utils.py` | 1,194 | ETag computation, key validation, copy source parsing |
| Notifications | `notifications.py` | 802 | S3 event notifications to SQS/SNS/Lambda |
| Presigned URLs | `presigned_url.py` | 935 | SigV2/V4 presigned URL validation |
| CORS | `cors.py` | 312 | CORS rule matching and response header generation |
| Website hosting | `website_hosting.py` | 411 | Static website hosting mode |
| Validation | `validation.py` | 528 | Request validation (bucket names, keys, ACLs) |
| Checksums | `checksums.py` | 169 | CRC32, CRC32C, SHA1, SHA256 computation |
| Storage (abstract + impl) | `storage/` | ~800 | S3ObjectStore trait + EphemeralS3ObjectStore |
| Constants, headers, codec | various | ~420 | Shared constants and header handling |

### 2.2 Layered Architecture

```
                     +-------------------------------------+
                     |    AWS SDK / CLI / boto3             |
                     +------------------+------------------+
                                        | HTTP :4566
                                        v
+-----------------------------------------------------------------------+
|                    HTTP Server (Hypercorn)                              |
|  +-------------------------------------------------------------------+|
|  |            LocalstackAwsGateway (Handler Chain)                    ||
|  |                                                                    ||
|  |  Request -> Parse Service -> Parse Protocol -> Route -> Dispatch   ||
|  +------------------------------+------------------------------------+|
+----------------------------------+-------------------------------------+
                                   |
                                   v
+-----------------------------------------------------------------------+
|                         S3Provider                                      |
|                                                                        |
|  Inherits: S3Api + ServiceLifecycleHook                               |
|                                                                        |
|  Key Components:                                                       |
|  - _storage_backend: S3ObjectStore (pluggable)                        |
|  - _notification_dispatcher: NotificationDispatcher                    |
|  - _cors_handler: S3CorsHandler                                       |
|  - _preconditions_locks: per-bucket/key locking                       |
|                                                                        |
|  State Layer:                                                          |
|  - AccountRegionBundle -> S3Store                                      |
|    - buckets: Dict[str, S3Bucket]                                      |
|    - global_bucket_map: Dict[str, AccountId]                          |
|    - TaggingService                                                    |
|                                                                        |
|  S3Bucket:                                                             |
|    - versioning_status: None | Enabled | Suspended                    |
|    - objects: KeyStore | VersionedKeyStore                            |
|    - multiparts: Dict[UploadId, MultipartUpload]                      |
|    - cors_rules, lifecycle_rules, notification_config                 |
|    - encryption_config, bucket_policy, acl                            |
|    - tags, logging_config, website_config                             |
|    - object_lock_config, ownership_controls                           |
|    - public_access_block                                               |
+-----------------------------------------------------------------------+
```

### 2.3 Request Processing Flow

1. HTTP request arrives at Hypercorn server on port 4566
2. Handler chain processes: Parse Service -> Parse Protocol -> Route
3. S3 protocol is REST-XML (not JSON)
4. Request is deserialized into typed input parameters
5. S3Provider handler method is called
6. Handler validates request (bucket exists, key valid, permissions)
7. Handler interacts with state layer (S3Store, S3Bucket, KeyStore)
8. Handler interacts with storage backend (read/write object data)
9. Response is constructed and serialized as XML
10. CORS headers added if applicable
11. Notifications dispatched if applicable

### 2.4 Concurrency Model

- **Per-bucket/key locking**: `_preconditions_locks` dictionary provides per-bucket, per-key locking for conditional operations (If-Match, If-None-Match, If-Modified-Since, If-Unmodified-Since)
- **Thread-safe storage**: `LockedSpooledTemporaryFile` uses:
  - `position_lock` (RLock): ensures atomic seek+read operations
  - `readwrite_lock` (RWLockWrite): multiple concurrent readers, exclusive writer

### 2.5 External Dependencies

The current Python S3 provider calls these other services:
- **KMS**: Validate encryption key IDs (can be stubbed)
- **SQS/SNS/Lambda/EventBridge**: Event notifications
- **IAM**: Policy evaluation (implicit)

---

## 3. Key S3 Operations (Detailed)

### 3.1 CreateBucket

**Complexity:** Medium

**Request:** `PUT /` with optional `CreateBucketConfiguration` XML body

```xml
<CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <LocationConstraint>eu-west-1</LocationConstraint>
</CreateBucketConfiguration>
```

**Key behaviors:**
- Bucket names are globally unique across all accounts
- LocationConstraint specifies the region (defaults to us-east-1)
- Validates bucket name (3-63 chars, lowercase, no periods for virtual-hosted)
- Optional headers: x-amz-acl, x-amz-grant-*, x-amz-bucket-object-lock-enabled, x-amz-object-ownership
- Returns `Location: /<bucket-name>` header
- Errors: BucketAlreadyExists (409), BucketAlreadyOwnedByYou (409)

### 3.2 PutObject

**Complexity:** High (40+ parameters)

**Request:** `PUT /{Key+}` with body containing object data

**Key behaviors:**
- Streaming write support for large objects
- Validate object key (max 1024 bytes UTF-8, non-empty)
- Extract user metadata from `x-amz-meta-*` headers (case-insensitive, lowercased)
- System metadata: Content-Type, Content-Encoding, Content-Language, Content-Disposition, Cache-Control, Expires
- Checksum computation: CRC32, CRC32C, SHA1, SHA256 (based on x-amz-checksum-algorithm)
- SSE encryption headers (stored as metadata, no actual encryption in LocalStack)
- ACL via x-amz-acl or grant headers
- Tagging via x-amz-tagging header (URL-encoded key=value pairs)
- Object Lock: retention mode/date via headers
- If versioning enabled: generate version ID (13-14 char base64 string)
- If versioning disabled: version ID is "null"
- Compute ETag as MD5 hex digest of object data
- Trigger notifications if configured
- Returns: ETag, x-amz-version-id, checksum headers

### 3.3 GetObject

**Complexity:** High

**Request:** `GET /{Key+}` with optional query params and conditional headers

**Key behaviors:**
- Streaming read support
- Range requests via `Range: bytes=start-end` header
  - `bytes=0-499` (first 500 bytes)
  - `bytes=-500` (last 500 bytes)
  - `bytes=500-` (from byte 500 onward)
  - Returns 206 Partial Content with Content-Range header
- Conditional headers:
  - `If-Match`: return only if ETag matches
  - `If-None-Match`: return only if ETag does not match (304 Not Modified)
  - `If-Modified-Since`: return only if modified after date
  - `If-Unmodified-Since`: return only if not modified after date (412 Precondition Failed)
- partNumber query parameter for multipart object parts
- versionId query parameter for specific version
- Response headers include all system metadata + user metadata
- If latest version is a delete marker: return 404 NoSuchKey with x-amz-delete-marker: true
- If specific versionId is a delete marker: return 405 MethodNotAllowed

### 3.4 HeadObject

**Complexity:** Medium

Same as GetObject but returns only headers, no body. Returns 200 on success with all metadata headers.

### 3.5 DeleteObject

**Complexity:** Medium

**Request:** `DELETE /{Key+}` with optional `versionId` query parameter

**Key behaviors:**
- Unversioned bucket: permanently removes object
- Versioned bucket (no versionId): creates delete marker (latest version becomes a delete marker)
- Versioned bucket (with versionId): permanently removes that specific version
- Returns x-amz-delete-marker: true if a delete marker was created
- Returns x-amz-version-id for the version affected

### 3.6 DeleteObjects (Batch Delete)

**Complexity:** Medium

**Request:** `POST /?delete` with XML body listing up to 1000 keys

```xml
<Delete>
    <Quiet>false</Quiet>
    <Object>
        <Key>key1</Key>
        <VersionId>version1</VersionId>
    </Object>
    <Object>
        <Key>key2</Key>
    </Object>
</Delete>
```

**Response:**
```xml
<DeleteResult>
    <Deleted>
        <Key>key1</Key>
        <VersionId>version1</VersionId>
    </Deleted>
    <Error>
        <Key>key2</Key>
        <Code>AccessDenied</Code>
        <Message>Access Denied</Message>
    </Error>
</DeleteResult>
```

### 3.7 CopyObject

**Complexity:** Very High (640 LOC in Python implementation)

**Request:** `PUT /{Key+}` with `x-amz-copy-source` header

**Key behaviors:**
- Copy between buckets or within same bucket
- x-amz-metadata-directive: COPY (default) or REPLACE
  - COPY: keeps source metadata
  - REPLACE: uses metadata from request headers
- Conditional headers on source: x-amz-copy-source-if-match, x-amz-copy-source-if-none-match, x-amz-copy-source-if-modified-since, x-amz-copy-source-if-unmodified-since
- Copy source format: `/bucket/key?versionId=xxx` (URL-encoded, + represents space)
- Supports cross-bucket copy with different encryption settings
- For same key copy with REPLACE: effectively updates metadata in-place
- Generate new version ID if destination versioning enabled
- New ETag if data changes
- Returns XML response with ETag and LastModified

**Response:**
```xml
<CopyObjectResult>
    <ETag>"etag"</ETag>
    <LastModified>2009-10-12T17:50:30.000Z</LastModified>
</CopyObjectResult>
```

### 3.8 Multipart Upload Operations

#### CreateMultipartUpload

**Request:** `POST /{Key+}?uploads`

Generates a unique upload ID. Stores metadata (content-type, encryption, tags, ACL) for later assembly.

**Response:**
```xml
<InitiateMultipartUploadResult>
    <Bucket>bucket</Bucket>
    <Key>key</Key>
    <UploadId>upload-id</UploadId>
</InitiateMultipartUploadResult>
```

#### UploadPart

**Request:** `PUT /{Key+}?partNumber=N&uploadId=ID`

- Minimum 5 MiB per part (except last part)
- Maximum 10,000 parts
- Compute MD5 ETag for part
- Optional checksum computation

#### UploadPartCopy

**Request:** `PUT /{Key+}?partNumber=N&uploadId=ID` with `x-amz-copy-source`

- Copy byte range from existing object as a part
- Supports x-amz-copy-source-range header
- Conditional headers supported

#### CompleteMultipartUpload

**Request:** `POST /{Key+}?uploadId=ID` with XML body

```xml
<CompleteMultipartUpload>
    <Part>
        <PartNumber>1</PartNumber>
        <ETag>"etag1"</ETag>
    </Part>
    <Part>
        <PartNumber>2</PartNumber>
        <ETag>"etag2"</ETag>
    </Part>
</CompleteMultipartUpload>
```

**Key behaviors:**
- Validate all parts exist and ETags match
- Parts must be in ascending order by part number
- Assemble parts into final object
- Composite ETag: `MD5(concat(MD5(part1), MD5(part2), ...))-N` where N is part count
- Composite checksums: `base64(hash(part1_checksum + part2_checksum + ...))-N`
- Generate version ID if versioning enabled

**Response:**
```xml
<CompleteMultipartUploadResult>
    <Location>http://bucket.s3.region.amazonaws.com/key</Location>
    <Bucket>bucket</Bucket>
    <Key>key</Key>
    <ETag>"composite-etag-N"</ETag>
</CompleteMultipartUploadResult>
```

**Important:** A 200 OK response can contain either success or error XML (must check response body).

#### AbortMultipartUpload

**Request:** `DELETE /{Key+}?uploadId=ID`

Cleans up all uploaded parts.

#### ListParts

**Request:** `GET /{Key+}?uploadId=ID`

```xml
<ListPartsResult>
    <Bucket>bucket</Bucket>
    <Key>key</Key>
    <UploadId>upload-id</UploadId>
    <PartNumberMarker>0</PartNumberMarker>
    <NextPartNumberMarker>2</NextPartNumberMarker>
    <MaxParts>1000</MaxParts>
    <IsTruncated>false</IsTruncated>
    <Part>
        <PartNumber>1</PartNumber>
        <LastModified>2009-10-12T17:50:30.000Z</LastModified>
        <ETag>"etag"</ETag>
        <Size>5242880</Size>
    </Part>
</ListPartsResult>
```

#### ListMultipartUploads

**Request:** `GET /?uploads`

```xml
<ListMultipartUploadsResult>
    <Bucket>bucket</Bucket>
    <KeyMarker></KeyMarker>
    <UploadIdMarker></UploadIdMarker>
    <NextKeyMarker>key</NextKeyMarker>
    <NextUploadIdMarker>upload-id</NextUploadIdMarker>
    <MaxUploads>1000</MaxUploads>
    <IsTruncated>false</IsTruncated>
    <Upload>
        <Key>key</Key>
        <UploadId>upload-id</UploadId>
        <Initiator>
            <ID>id</ID>
            <DisplayName>name</DisplayName>
        </Initiator>
        <Owner>
            <ID>id</ID>
            <DisplayName>name</DisplayName>
        </Owner>
        <StorageClass>STANDARD</StorageClass>
        <Initiated>2009-10-12T17:50:30.000Z</Initiated>
    </Upload>
</ListMultipartUploadsResult>
```

### 3.9 List Operations

#### ListObjects (v1, deprecated but widely used)

**Request:** `GET /` with query params: prefix, delimiter, marker, max-keys, encoding-type

**Response:** `ListBucketResult` XML with Contents elements sorted lexicographically.

Pagination: Use `Marker` from response `NextMarker` or last key for next request.

#### ListObjectsV2

**Request:** `GET /?list-type=2` with query params: prefix, delimiter, continuation-token, start-after, max-keys, fetch-owner, encoding-type

**Response:** `ListBucketResult` XML with Contents, KeyCount, ContinuationToken, NextContinuationToken.

Pagination: Use `NextContinuationToken` for next request.

**Key difference from v1:** Uses continuation-token instead of marker, returns KeyCount, supports start-after.

#### ListObjectVersions

**Request:** `GET /?versions` with query params: prefix, delimiter, key-marker, version-id-marker, max-keys, encoding-type

**Response:** `ListVersionsResult` XML with interleaved `Version` and `DeleteMarker` elements.

Pagination: Use NextKeyMarker + NextVersionIdMarker.

### 3.10 Presigned URLs

**Key behaviors:**
- s3s handles SigV4 verification via the `S3Auth` trait
- Presigned URLs contain query parameters: X-Amz-Algorithm, X-Amz-Credential, X-Amz-Date, X-Amz-Expires, X-Amz-SignedHeaders, X-Amz-Signature
- LocalStack validates both SigV2 and SigV4
- Default expiry: 3600 seconds (configurable)
- For local dev: skip signature validation (S3_SKIP_SIGNATURE_VALIDATION=true)

---

## 4. S3 Features Supported by LocalStack

### 4.1 Versioning

**States:**
- **Unversioned** (default): Single version per key, overwrites replace, version ID is "null"
- **Enabled**: New version created on each write with generated version ID (13-14 char base64 string)
- **Suspended**: New writes use "null" version ID, existing versions persist

**Data structures:**
- `KeyStore`: Simple dict-like store for unversioned buckets
- `VersionedKeyStore`: Maintains key index, version index, tracks latest versions

**Delete markers:**
- Delete without versionId in versioned bucket creates `S3DeleteMarker`
- GetObject on delete marker returns `NoSuchKey` (404) with `x-amz-delete-marker: true`
- GetObject with delete marker's versionId returns `MethodNotAllowed` (405)
- Delete with specific versionId permanently removes that version/delete marker

**Transition:**
- When versioning is enabled, bucket switches from KeyStore to VersionedKeyStore
- Existing "null" versioned objects are preserved
- When suspended, new writes overwrite the "null" version

### 4.2 CORS

**Implementation:** S3CorsHandler middleware

**Key behaviors:**
- OPTIONS preflight: Extract bucket, match Origin + Access-Control-Request-Method + Access-Control-Request-Headers against stored rules
- Regular requests: Match Origin against bucket CORS rules, add Access-Control-Allow-Origin, Access-Control-Expose-Headers
- Rules support wildcards in AllowedOrigins and AllowedHeaders
- MaxAgeSeconds for preflight caching

**CORS Rule structure:**
```xml
<CORSConfiguration>
    <CORSRule>
        <AllowedOrigin>*</AllowedOrigin>
        <AllowedMethod>GET</AllowedMethod>
        <AllowedMethod>PUT</AllowedMethod>
        <AllowedHeader>*</AllowedHeader>
        <ExposeHeader>x-amz-request-id</ExposeHeader>
        <MaxAgeSeconds>3600</MaxAgeSeconds>
    </CORSRule>
</CORSConfiguration>
```

### 4.3 ACLs

**Canned ACLs:**
- private (default)
- public-read
- public-read-write
- authenticated-read
- aws-exec-read
- bucket-owner-read
- bucket-owner-full-control
- log-delivery-write

**Grant types:** READ, WRITE, READ_ACP, WRITE_ACP, FULL_CONTROL

**Note:** Object Ownership setting (BucketOwnerEnforced) disables ACLs.

### 4.4 Bucket Policies

- Stored as JSON documents
- GetBucketPolicyStatus returns whether the policy grants public access
- LocalStack stores but does not fully enforce policies (limited IAM evaluation)

### 4.5 Encryption

- **SSE-S3**: AES-256 default encryption config stored per bucket
- **SSE-KMS**: KMS key ID stored (no actual key validation without KMS service)
- **SSE-C**: Parameter validation only, no actual encryption/decryption
- Encryption headers stored in object metadata
- BucketKeyEnabled setting stored

### 4.6 Tagging

**Bucket tags:** Unlimited tags via PutBucketTagging

**Object tags:** Up to 10 tags per object
- Key: 1-128 Unicode characters
- Value: 0-256 Unicode characters
- Set during PutObject via x-amz-tagging header
- Managed via PutObjectTagging, GetObjectTagging, DeleteObjectTagging

### 4.7 Object Lock (WORM)

- **Object Lock Configuration**: Set at bucket creation time (x-amz-bucket-object-lock-enabled: true)
- **Retention modes:**
  - GOVERNANCE: can be overridden by privileged users
  - COMPLIANCE: cannot be overridden
- **Retention period:** Date-based
- **Legal hold:** Prevents deletion regardless of retention settings
- Operations: PutObjectRetention, GetObjectRetention, PutObjectLegalHold, GetObjectLegalHold

### 4.8 Lifecycle Rules

- Configuration stored but limited execution
- Rules support: Expiration, Transition, NoncurrentVersionExpiration, NoncurrentVersionTransition
- Filters: Prefix, Tag, And (multiple conditions)

### 4.9 Checksums

**Supported algorithms:**
- CRC32
- CRC32C
- CRC64NVME (newer)
- SHA1
- SHA256

**Checksum types:**
- Single object: direct hash of object data
- Multipart: composite checksum `base64(hash(part1_checksum + part2_checksum + ...))-N`

### 4.10 Notifications

**Event types:**
- s3:ObjectCreated:* (Put, Post, Copy, CompleteMultipartUpload)
- s3:ObjectRemoved:* (Delete, DeleteMarkerCreated)
- s3:ObjectRestore:* (Post, Completed)
- s3:ObjectTagging:* (Put, Delete)
- s3:ObjectAcl:Put

**Destinations:**
- SQS Queue
- SNS Topic
- Lambda Function
- EventBridge

**Filter rules:** Prefix and suffix matching on key names.

### 4.11 Website Hosting

- Index document and error document configuration
- Routing rules with conditions and redirects
- Handled by S3 website hosting module

### 4.12 Other Features

- **Accelerate configuration**: Stored only
- **Request payment**: Stored only
- **Logging**: Configuration stored
- **Replication**: Configuration stored, no actual replication
- **Analytics/Metrics/Inventory**: Configuration stored
- **Intelligent tiering**: Configuration stored
- **Public access block**: Stored and partially enforced
- **Ownership controls**: Stored and enforced for ACL behavior

---

## 5. Local Storage Implementation

### 5.1 Storage Backend Trait

LocalStack defines `S3ObjectStore` as an abstract class with these key methods:

```
- open(bucket, s3_object) -> S3StoredObject (read handle)
- write(s3_stored_object) -> write object data
- copy(src_bucket, src_object, dest_bucket, dest_object) -> copy data
- remove(bucket, objects_to_remove) -> delete data
- create_bucket(bucket_name) -> create storage area
- delete_bucket(bucket_name) -> delete storage area
- write_part(upload_id, part_number, data) -> write multipart part
- get_part(upload_id, part_number) -> read part
- remove_parts(upload_id) -> cleanup parts
- complete_multipart(upload_id, parts) -> assemble final object
```

### 5.2 Ephemeral Storage (Default)

`EphemeralS3ObjectStore` implementation:

- Uses `SpooledTemporaryFile` (Python stdlib)
- **Threshold:** 512 KB (configurable via S3_MAX_MEMORY_OBJECT_SIZE)
- Objects <= 512 KB: kept entirely in memory
- Objects > 512 KB: spilled to temporary disk files
- Temp directory: `/tmp/localstack-s3/` (inside container)
- All data lost on container stop

### 5.3 Persistent Storage

When `PERSISTENCE=1`:
- State data stored under `/var/lib/localstack/state/`
- Uses pickle/dill serialization for state snapshots
- Object data stored on filesystem
- Survives container restarts when volume is mounted

### 5.4 Directory Layout

```
/var/lib/localstack/
  state/           # Service state (pickle/dill serialized)
  tmp/             # Temporary data, cleared on startup
  cache/           # Cache data persisting across runs
  logs/            # Log files
  lib/             # Extensions, lazy-loaded deps

/tmp/localstack-s3/   # Spilled temporary files for large objects
```

### 5.5 Object Data Model

```python
class S3Object:
    key: str
    version_id: str          # "null" for unversioned
    size: int
    etag: str                # MD5 hex, or composite for multipart
    last_modified: datetime
    storage_class: str       # STANDARD, GLACIER, etc.
    user_metadata: dict      # x-amz-meta-* headers
    system_metadata: dict    # Content-Type, Content-Encoding, etc.
    checksum_algorithm: str  # CRC32, CRC32C, SHA1, SHA256
    checksum_value: str      # Computed checksum
    encryption_algorithm: str
    kms_key_id: str
    bucket_key_enabled: bool
    lock_mode: str           # GOVERNANCE, COMPLIANCE
    lock_until: datetime
    legal_hold: bool
    acl: ObjectAcl
    tags: dict
    website_redirect_location: str
```

---

## 6. S3 API XML Format Reference

### 6.1 XML Namespace

All S3 XML uses the namespace: `http://s3.amazonaws.com/doc/2006-03-01/`

### 6.2 Key XML Response Formats

#### ListBuckets

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Owner>
        <ID>owner-id</ID>
        <DisplayName>display-name</DisplayName>
    </Owner>
    <Buckets>
        <Bucket>
            <Name>bucket-name</Name>
            <CreationDate>2024-01-01T00:00:00.000Z</CreationDate>
            <BucketRegion>us-east-1</BucketRegion>
        </Bucket>
    </Buckets>
    <ContinuationToken>token</ContinuationToken>
    <Prefix>prefix</Prefix>
</ListAllMyBucketsResult>
```

#### ListObjectsV2

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Name>bucket</Name>
    <Prefix>prefix</Prefix>
    <Delimiter>/</Delimiter>
    <MaxKeys>1000</MaxKeys>
    <KeyCount>2</KeyCount>
    <IsTruncated>false</IsTruncated>
    <ContinuationToken>token</ContinuationToken>
    <NextContinuationToken>next-token</NextContinuationToken>
    <StartAfter>key</StartAfter>
    <EncodingType>url</EncodingType>
    <Contents>
        <Key>my-image.jpg</Key>
        <LastModified>2009-10-12T17:50:30.000Z</LastModified>
        <ETag>"fba9dede5f27731c9771645a39863328"</ETag>
        <Size>434234</Size>
        <StorageClass>STANDARD</StorageClass>
        <ChecksumAlgorithm>CRC32</ChecksumAlgorithm>
        <Owner>
            <ID>owner-id</ID>
            <DisplayName>display-name</DisplayName>
        </Owner>
    </Contents>
    <CommonPrefixes>
        <Prefix>photos/</Prefix>
    </CommonPrefixes>
</ListBucketResult>
```

#### ListObjects (v1)

Same as ListObjectsV2 but uses `Marker`/`NextMarker` instead of continuation tokens, and no `KeyCount`.

#### ListObjectVersions

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Name>bucket</Name>
    <Prefix></Prefix>
    <KeyMarker></KeyMarker>
    <VersionIdMarker></VersionIdMarker>
    <NextKeyMarker>key</NextKeyMarker>
    <NextVersionIdMarker>version</NextVersionIdMarker>
    <MaxKeys>1000</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Version>
        <Key>my-image.jpg</Key>
        <VersionId>3/L4kqtJl40Nr8X8gdRQBpUMLUo</VersionId>
        <IsLatest>true</IsLatest>
        <LastModified>2009-10-12T17:50:30.000Z</LastModified>
        <ETag>"fba9dede5f27731c9771645a39863328"</ETag>
        <Size>434234</Size>
        <StorageClass>STANDARD</StorageClass>
        <Owner>
            <ID>owner-id</ID>
        </Owner>
    </Version>
    <DeleteMarker>
        <Key>my-second-image.jpg</Key>
        <VersionId>03jpff543dhffds434rfdsFDN943fdsFkdmqnh892</VersionId>
        <IsLatest>true</IsLatest>
        <LastModified>2009-11-12T17:50:30.000Z</LastModified>
        <Owner>
            <ID>owner-id</ID>
        </Owner>
    </DeleteMarker>
</ListVersionsResult>
```

#### CompleteMultipartUpload Request

```xml
<CompleteMultipartUpload xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Part>
        <PartNumber>1</PartNumber>
        <ETag>"a54357aff0632cce46d942af68356b38"</ETag>
    </Part>
    <Part>
        <PartNumber>2</PartNumber>
        <ETag>"0c78aef83f66abc1fa1e8477f296d394"</ETag>
    </Part>
</CompleteMultipartUpload>
```

#### CompleteMultipartUpload Response

```xml
<CompleteMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Location>http://bucket.s3.region.amazonaws.com/key</Location>
    <Bucket>bucket</Bucket>
    <Key>key</Key>
    <ETag>"3858f62230ac3c915f300c664312c11f-9"</ETag>
</CompleteMultipartUploadResult>
```

#### Error Response

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>NoSuchKey</Code>
    <Message>The specified key does not exist.</Message>
    <Key>my-key</Key>
    <BucketName>my-bucket</BucketName>
    <Resource>/my-bucket/my-key</Resource>
    <RequestId>4442587FB7D0A2F9</RequestId>
</Error>
```

#### DeleteObjects Request

```xml
<Delete>
    <Quiet>false</Quiet>
    <Object>
        <Key>key1</Key>
        <VersionId>version1</VersionId>
    </Object>
</Delete>
```

#### DeleteObjects Response

```xml
<DeleteResult>
    <Deleted>
        <Key>key1</Key>
        <VersionId>version1</VersionId>
        <DeleteMarker>true</DeleteMarker>
        <DeleteMarkerVersionId>marker-version</DeleteMarkerVersionId>
    </Deleted>
    <Error>
        <Key>key2</Key>
        <Code>AccessDenied</Code>
        <Message>Access Denied</Message>
    </Error>
</DeleteResult>
```

#### CreateBucketConfiguration

```xml
<CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <LocationConstraint>eu-west-1</LocationConstraint>
</CreateBucketConfiguration>
```

### 6.3 Key XML Notes

- s3s handles all XML serialization/deserialization automatically from Smithy models
- Timestamps are ISO 8601 format: `2009-10-12T17:50:30.000Z`
- ETags are always quoted: `"fba9dede5f27731c9771645a39863328"`
- Size is a long integer (supports objects up to 5 TiB)
- StorageClass values: STANDARD, REDUCED_REDUNDANCY, GLACIER, STANDARD_IA, ONEZONE_IA, INTELLIGENT_TIERING, DEEP_ARCHIVE, OUTPOSTS, GLACIER_IR, SNOW, EXPRESS_ONEZONE

---

## 7. s3s Crate Analysis (Rust Foundation)

### 7.1 Overview

- **Crate:** `s3s` v0.11.1 (47K SLoC, Apache-2.0 license)
- **Purpose:** S3 Service Adapter - converts HTTP to S3 operations and back
- **Architecture:** Generic hyper service implementing S3 REST API

### 7.2 What s3s Provides

| Feature | Status | Notes |
|---------|--------|-------|
| HTTP server (hyper+tower) | Yes | HTTP/1.1 + HTTP/2 |
| Virtual-hosted style addressing | Yes | S3Host trait with domain config |
| Path-style addressing | Yes | Automatic fallback |
| XML serialization/deserialization | Yes | Generated from Smithy model |
| SigV4 signature verification | Yes | S3Auth trait |
| Request routing (96 operations) | Yes | Auto-generated from Smithy |
| Streaming upload/download | Yes | ByteStream trait |
| Tower middleware integration | Yes | Standard tower::Layer support |
| S3-standard error formatting | Yes | XML error responses |
| Multipart upload routing | Yes | All 5 multipart operations |

### 7.3 What s3s Does NOT Provide

- Business logic (you implement the S3 trait)
- Storage backend
- State management
- CORS handling (need custom middleware)
- Health check endpoints
- Notifications
- Any S3 feature logic

### 7.4 Workspace Crates

| Crate | Purpose |
|-------|---------|
| s3s | Core library (S3 trait, routing, XML) |
| s3s-aws | AWS SDK type conversions |
| s3s-fs | Reference filesystem S3 implementation |

### 7.5 Security Note

s3s explicitly warns: "S3Service and other adapters have no security protection." Users must implement body limits, rate limiting, back pressure.

### 7.6 S3 Trait Pattern

Each operation follows:
```rust
async fn operation_name(
    &self,
    req: S3Request<OperationInput>,
) -> S3Result<S3Response<OperationOutput>>;
```

Default implementation returns `S3Error::NotImplemented`.

---

## 8. S3 Error Codes Reference

### 8.1 Most Common Error Codes for Implementation

| Error Code | HTTP Status | When |
|-----------|------------|------|
| NoSuchBucket | 404 | Bucket does not exist |
| NoSuchKey | 404 | Object key does not exist |
| NoSuchUpload | 404 | Multipart upload ID not found |
| NoSuchVersion | 404 | Version ID not found |
| BucketAlreadyExists | 409 | Bucket name taken |
| BucketAlreadyOwnedByYou | 409 | You already own this bucket |
| BucketNotEmpty | 409 | Cannot delete non-empty bucket |
| InvalidBucketName | 400 | Invalid bucket name format |
| InvalidArgument | 400 | Generic invalid parameter |
| InvalidRange | 416 | Range not satisfiable |
| InvalidPart | 400 | Part not found or ETag mismatch |
| InvalidPartOrder | 400 | Parts not in ascending order |
| EntityTooSmall | 400 | Part smaller than 5 MiB minimum |
| EntityTooLarge | 400 | Upload exceeds max size |
| BadDigest | 400 | Content-MD5/checksum mismatch |
| PreconditionFailed | 412 | Conditional header check failed |
| NotModified | 304 | If-None-Match matched |
| MethodNotAllowed | 405 | Invalid method for resource |
| MalformedXML | 400 | XML not well-formed |
| AccessDenied | 403 | Permission denied |
| InternalError | 500 | Server error |
| NotImplemented | 501 | Operation not implemented |
| NoSuchCORSConfiguration | 404 | No CORS config on bucket |
| NoSuchBucketPolicy | 404 | No policy on bucket |
| NoSuchLifecycleConfiguration | 404 | No lifecycle config |
| NoSuchWebsiteConfiguration | 404 | No website config |
| NoSuchTagSet | 404 | No tags on resource |
| ObjectLockConfigurationNotFoundError | 404 | No Object Lock config |
| OwnershipControlsNotFoundError | 404 | No ownership controls |
| OperationAborted | 409 | Conflicting concurrent operation |
| SlowDown | 503 | Rate limiting |

### 8.2 Error Response XML Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>ErrorCode</Code>
    <Message>Human-readable message</Message>
    <Resource>/bucket/key</Resource>
    <RequestId>request-id</RequestId>
</Error>
```

---

## 9. Implementation Priority Matrix

### 9.1 Phase 1 (MVP) - Target 80% test pass rate

**P0 - Must have (test infrastructure + core CRUD):**

1. CreateBucket, DeleteBucket, HeadBucket, ListBuckets
2. PutObject, GetObject, HeadObject, DeleteObject, DeleteObjects
3. CopyObject
4. ListObjects, ListObjectsV2
5. CreateMultipartUpload, UploadPart, UploadPartCopy, CompleteMultipartUpload, AbortMultipartUpload, ListParts, ListMultipartUploads
6. PutBucketVersioning, GetBucketVersioning + versioned object behavior
7. ListObjectVersions
8. GetBucketLocation
9. Health check endpoint (/_localstack/health)

**P1 - Important (bucket/object config):**

10. PutBucketEncryption, GetBucketEncryption, DeleteBucketEncryption
11. PutBucketTagging, GetBucketTagging, DeleteBucketTagging
12. PutObjectTagging, GetObjectTagging, DeleteObjectTagging
13. PutBucketCors, GetBucketCors, DeleteBucketCors + CORS middleware
14. PutBucketPolicy, GetBucketPolicy, DeleteBucketPolicy, GetBucketPolicyStatus
15. PutBucketAcl, GetBucketAcl, PutObjectAcl, GetObjectAcl
16. PutPublicAccessBlock, GetPublicAccessBlock, DeletePublicAccessBlock
17. PutBucketOwnershipControls, GetBucketOwnershipControls, DeleteBucketOwnershipControls
18. PutObjectLockConfiguration, GetObjectLockConfiguration
19. PutObjectRetention, GetObjectRetention, PutObjectLegalHold, GetObjectLegalHold
20. GetObjectAttributes
21. Presigned URL pass-through via S3Auth

**P2 - Nice to have (remaining configs):**

22. PutBucketLifecycleConfiguration, GetBucketLifecycleConfiguration, DeleteBucketLifecycle
23. PutBucketNotificationConfiguration, GetBucketNotificationConfiguration (store only)
24. PutBucketLogging, GetBucketLogging
25. PutBucketAccelerateConfiguration, GetBucketAccelerateConfiguration
26. PutBucketRequestPayment, GetBucketRequestPayment
27. PutBucketWebsite, GetBucketWebsite, DeleteBucketWebsite
28. PutBucketReplication, GetBucketReplication, DeleteBucketReplication
29. RestoreObject
30. PostObject (HTML form upload)

### 9.2 Phase 2 (Full Parity) - Target 95% test pass rate

31. Notification dispatch (SQS, SNS, Lambda, EventBridge)
32. Website hosting mode
33. Persistence (snapshot/restore)
34. Lifecycle rule execution (TTL expiration)
35. Analytics/Metrics/Inventory configuration management
36. Intelligent tiering configuration

### 9.3 Likely Not Needed

- SelectObjectContent (SQL queries on objects)
- GetObjectTorrent
- WriteGetObjectResponse (Object Lambda)
- CreateSession (Express One Zone)
- Directory bucket operations
- S3 Tables / S3 Vectors operations
- Metadata table operations

---

## 10. Feature Parity Checklist

### 10.1 Core Protocol

- [ ] REST-XML protocol handling (via s3s)
- [ ] Virtual-hosted style addressing (`bucket.s3.localhost.localstack.cloud`)
- [ ] Path-style addressing (`/bucket/key`)
- [ ] SigV4 signature verification (or skip for dev mode)
- [ ] Presigned URL support (SigV2 and SigV4)
- [ ] Chunked transfer encoding
- [ ] Range request headers
- [ ] Conditional request headers (If-Match, If-None-Match, If-Modified-Since, If-Unmodified-Since)
- [ ] x-amz-* header handling
- [ ] URL encoding for object keys

### 10.2 Bucket Features

- [ ] Global bucket name uniqueness
- [ ] Bucket name validation (3-63 chars, lowercase, etc.)
- [ ] Location constraint
- [ ] Versioning (Enabled/Suspended states)
- [ ] Encryption configuration (SSE-S3, SSE-KMS metadata)
- [ ] CORS configuration + preflight handling
- [ ] Bucket policy (store and return)
- [ ] ACLs (canned + grant-based)
- [ ] Public access block
- [ ] Ownership controls
- [ ] Object Lock configuration
- [ ] Lifecycle configuration (store)
- [ ] Notification configuration (store)
- [ ] Logging configuration (store)
- [ ] Website configuration (store)
- [ ] Replication configuration (store)
- [ ] Tagging
- [ ] Accelerate configuration (store)
- [ ] Request payment (store)

### 10.3 Object Features

- [ ] Streaming upload/download
- [ ] User metadata (x-amz-meta-*)
- [ ] System metadata (Content-Type, etc.)
- [ ] ETag computation (MD5 for single, composite for multipart)
- [ ] Checksums (CRC32, CRC32C, SHA1, SHA256)
- [ ] Version IDs
- [ ] Delete markers
- [ ] Object tagging
- [ ] Object ACLs
- [ ] Object Lock (retention + legal hold)
- [ ] Storage class (metadata only)
- [ ] SSE headers (metadata only)
- [ ] GetObjectAttributes

### 10.4 Multipart Upload

- [ ] Upload initiation with metadata
- [ ] Part upload with checksum
- [ ] Part copy from existing objects
- [ ] Part assembly with validation
- [ ] Composite ETag computation
- [ ] Composite checksum computation
- [ ] Part listing with pagination
- [ ] Upload listing with pagination
- [ ] Upload abort with cleanup
- [ ] Minimum part size validation (5 MiB except last)
- [ ] Maximum 10,000 parts

### 10.5 List Operations

- [ ] Prefix filtering
- [ ] Delimiter grouping (CommonPrefixes)
- [ ] Pagination (marker/continuation-token based)
- [ ] Encoding type (url)
- [ ] Max-keys limiting
- [ ] Fetch-owner option (v2)
- [ ] Version listing with delete markers

### 10.6 Infrastructure

- [ ] Health check endpoint (/_localstack/health)
- [ ] Configurable port (default 4566)
- [ ] Environment variable configuration
- [ ] Logging with tracing
- [ ] Docker image (scratch-based, <50 MiB)
- [ ] Multi-architecture (amd64 + arm64)

---

## Sources

- [LocalStack S3 Documentation](https://docs.localstack.cloud/aws/services/s3/)
- [LocalStack S3 Service Architecture (DeepWiki)](https://deepwiki.com/localstack/localstack/3.1-s3-service)
- [LocalStack GitHub Repository](https://github.com/localstack/localstack)
- [LocalStack API Coverage](https://docs.localstack.cloud/references/coverage)
- [LocalStack Filesystem Layout](https://docs.localstack.cloud/references/filesystem/)
- [AWS S3 API Reference](https://docs.aws.amazon.com/AmazonS3/latest/API/Type_API_Reference.html)
- [AWS S3 Error Responses](https://docs.aws.amazon.com/AmazonS3/latest/API/ErrorResponses.html)
- [s3s Crate (crates.io)](https://crates.io/crates/s3s)
- [s3s GitHub Repository](https://github.com/Nugine/s3s)
- [s3s S3 Trait Source](https://github.com/Nugine/s3s/blob/main/crates/s3s/src/s3_trait.rs)
- [s3s Documentation (docs.rs)](https://docs.rs/s3s/latest/s3s/)
