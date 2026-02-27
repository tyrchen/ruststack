//! S3 XML deserialization: parsing S3-compatible XML into Rust types.
//!
//! This module provides the [`S3Deserialize`] trait and implementations for all S3 types
//! that need to be deserialized from XML request bodies. The deserialization follows the
//! AWS S3 RestXml protocol conventions.

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::XmlError;

/// Trait for deserializing S3 types from XML.
///
/// Implementors parse XML elements from the reader and populate the struct fields.
/// The root element has already been consumed by the caller; the implementation
/// reads child elements until the matching end tag.
pub trait S3Deserialize: Sized {
    /// Deserialize an instance from the given XML reader.
    ///
    /// The reader is positioned just after the opening tag of this element.
    /// The implementation should read all child content and return when
    /// the matching end tag is consumed.
    ///
    /// # Errors
    ///
    /// Returns `XmlError` if the XML is malformed or required fields are missing.
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError>;
}

/// Deserialize S3-compatible XML into a typed value.
///
/// Finds the root element and delegates to the type's `S3Deserialize` implementation.
///
/// # Errors
///
/// Returns `XmlError` if the XML is malformed or deserialization fails.
pub fn from_xml<T: S3Deserialize>(xml: &[u8]) -> Result<T, XmlError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    // Skip the XML declaration and find the root element.
    loop {
        match reader.read_event()? {
            Event::Start(_) => {
                return T::deserialize_xml(&mut reader);
            }
            Event::Eof => {
                return Err(XmlError::MissingElement("root element".to_string()));
            }
            // Skip declaration, comments, processing instructions, whitespace.
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions for reading common XML patterns
// ---------------------------------------------------------------------------

/// Read the text content of the current element and consume its end tag.
///
/// Expects the reader to be positioned right after a `Start` event. Reads
/// the text content and consumes through the matching `End` event.
fn read_text_content(reader: &mut Reader<&[u8]>) -> Result<String, XmlError> {
    let mut text = String::new();
    loop {
        match reader.read_event()? {
            Event::Text(e) => {
                let decoded = e
                    .decode()
                    .map_err(|err| XmlError::ParseError(err.to_string()))?;
                let unescaped = quick_xml::escape::unescape(&decoded)
                    .map_err(|err| XmlError::ParseError(err.to_string()))?;
                text.push_str(&unescaped);
            }
            Event::End(_) => {
                return Ok(text);
            }
            Event::Eof => {
                return Err(XmlError::UnexpectedElement(
                    "unexpected EOF while reading text content".to_string(),
                ));
            }
            _ => {}
        }
    }
}

/// Skip over an element and all its children.
fn skip_element(reader: &mut Reader<&[u8]>) -> Result<(), XmlError> {
    let mut depth: u32 = 1;
    loop {
        match reader.read_event()? {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            Event::Eof => {
                return Err(XmlError::UnexpectedElement(
                    "unexpected EOF while skipping element".to_string(),
                ));
            }
            _ => {}
        }
    }
}

/// Parse a boolean from XML text ("true"/"false").
fn parse_bool(s: &str) -> Result<bool, XmlError> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(XmlError::ParseError(format!("invalid boolean: {s}"))),
    }
}

/// Parse an i32 from XML text.
fn parse_i32(s: &str) -> Result<i32, XmlError> {
    s.parse::<i32>()
        .map_err(|e| XmlError::ParseError(format!("invalid i32 '{s}': {e}")))
}

/// Parse an i64 from XML text.
fn parse_i64(s: &str) -> Result<i64, XmlError> {
    s.parse::<i64>()
        .map_err(|e| XmlError::ParseError(format!("invalid i64 '{s}': {e}")))
}

/// Parse an ISO 8601 timestamp from XML text.
fn parse_timestamp(s: &str) -> Result<chrono::DateTime<chrono::Utc>, XmlError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            // Try parsing the S3 format: 2006-02-03T16:45:09.000Z
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ")
                .map(|ndt| ndt.and_utc())
        })
        .map_err(|e| XmlError::ParseError(format!("invalid timestamp '{s}': {e}")))
}

// ---------------------------------------------------------------------------
// S3Deserialize implementations for input types
// ---------------------------------------------------------------------------

use ruststack_s3_model::types::{
    AbortIncompleteMultipartUpload, AccelerateConfiguration, AccessControlPolicy,
    BucketAccelerateStatus, BucketInfo, BucketLifecycleConfiguration, BucketLoggingStatus,
    BucketLogsPermission, BucketVersioningStatus, CORSConfiguration, CORSRule,
    CompletedMultipartUpload, CompletedPart, Condition, CreateBucketConfiguration,
    DefaultRetention, Delete, ErrorDocument, EventBridgeConfiguration, ExpirationStatus,
    FilterRule, FilterRuleName, Grant, Grantee, IndexDocument, LambdaFunctionConfiguration,
    LifecycleExpiration, LifecycleRule, LifecycleRuleAndOperator, LifecycleRuleFilter,
    LocationInfo, LoggingEnabled, MFADelete, NoncurrentVersionExpiration,
    NoncurrentVersionTransition, NotificationConfiguration, NotificationConfigurationFilter,
    ObjectIdentifier, ObjectLockConfiguration, ObjectLockEnabled, ObjectLockLegalHold,
    ObjectLockLegalHoldStatus, ObjectLockRetention, ObjectLockRetentionMode, ObjectLockRule,
    ObjectOwnership, Owner, OwnershipControls, OwnershipControlsRule, PartitionedPrefix, Payer,
    Permission, Protocol, PublicAccessBlockConfiguration, QueueConfiguration, Redirect,
    RedirectAllRequestsTo, RequestPaymentConfiguration, RoutingRule, S3KeyFilter,
    ServerSideEncryptionByDefault, ServerSideEncryptionConfiguration, ServerSideEncryptionRule,
    SimplePrefix, Tag, Tagging, TargetGrant, TargetObjectKeyFormat, TopicConfiguration, Transition,
    TransitionStorageClass, VersioningConfiguration, WebsiteConfiguration,
};
use ruststack_s3_model::types::{
    BucketLocationConstraint, BucketType, DataRedundancy, Event as S3Event, LocationType,
    ServerSideEncryption,
};

impl S3Deserialize for Tag {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut key = None;
        let mut value = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Key" => key = Some(read_text_content(reader)?),
                        "Value" => value = Some(read_text_content(reader)?),
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Tag".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Tag {
            key: key.unwrap_or_default(),
            value: value.unwrap_or_default(),
        })
    }
}

impl S3Deserialize for Tagging {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut tag_set = Vec::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "TagSet" => {
                            tag_set = deserialize_list(reader, "Tag")?;
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Tagging".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Tagging { tag_set })
    }
}

/// Deserialize a list of items where each item is wrapped in the given element name.
fn deserialize_list<T: S3Deserialize>(
    reader: &mut Reader<&[u8]>,
    item_tag: &str,
) -> Result<Vec<T>, XmlError> {
    let mut items = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = e.name();
                let tag_name = std::str::from_utf8(name.as_ref())
                    .map_err(|e| XmlError::ParseError(e.to_string()))?;
                if tag_name == item_tag {
                    items.push(T::deserialize_xml(reader)?);
                } else {
                    skip_element(reader)?;
                }
            }
            Event::End(_) => break,
            Event::Eof => {
                return Err(XmlError::UnexpectedElement(
                    "unexpected EOF in list".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(items)
}

impl S3Deserialize for Owner {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut id = None;
        let mut display_name = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "ID" => id = Some(read_text_content(reader)?),
                        "DisplayName" => display_name = Some(read_text_content(reader)?),
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Owner".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Owner { id, display_name })
    }
}

impl S3Deserialize for Grantee {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut display_name = None;
        let mut email_address = None;
        let mut id = None;
        let mut grantee_type = ruststack_s3_model::types::Type::default();
        let mut uri = None;

        // Note: the xsi:type attribute is on the <Grantee> element which has already
        // been consumed. We'd need to parse it from the Start event, but for deserialization
        // from client input, the type is typically inferred from which sub-fields are present.
        // For robustness, we also accept the <Type> child element.

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "DisplayName" => display_name = Some(read_text_content(reader)?),
                        "EmailAddress" => email_address = Some(read_text_content(reader)?),
                        "ID" => id = Some(read_text_content(reader)?),
                        "URI" => uri = Some(read_text_content(reader)?),
                        "Type" => {
                            let text = read_text_content(reader)?;
                            grantee_type = ruststack_s3_model::types::Type::from(text.as_str());
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Grantee".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Grantee {
            display_name,
            email_address,
            id,
            r#type: grantee_type,
            uri,
        })
    }
}

impl S3Deserialize for Grant {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut grantee = None;
        let mut permission = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Grantee" => grantee = Some(Grantee::deserialize_xml(reader)?),
                        "Permission" => {
                            let text = read_text_content(reader)?;
                            permission = Some(Permission::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Grant".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Grant {
            grantee,
            permission,
        })
    }
}

impl S3Deserialize for AccessControlPolicy {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut owner = None;
        let mut grants = Vec::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Owner" => owner = Some(Owner::deserialize_xml(reader)?),
                        "AccessControlList" => {
                            grants = deserialize_list(reader, "Grant")?;
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in AccessControlPolicy".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(AccessControlPolicy { grants, owner })
    }
}

impl S3Deserialize for CORSRule {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut allowed_headers = Vec::new();
        let mut allowed_methods = Vec::new();
        let mut allowed_origins = Vec::new();
        let mut expose_headers = Vec::new();
        let mut id = None;
        let mut max_age_seconds = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "AllowedHeader" => {
                            allowed_headers.push(read_text_content(reader)?);
                        }
                        "AllowedMethod" => {
                            allowed_methods.push(read_text_content(reader)?);
                        }
                        "AllowedOrigin" => {
                            allowed_origins.push(read_text_content(reader)?);
                        }
                        "ExposeHeader" => {
                            expose_headers.push(read_text_content(reader)?);
                        }
                        "ID" => id = Some(read_text_content(reader)?),
                        "MaxAgeSeconds" => {
                            let text = read_text_content(reader)?;
                            max_age_seconds = Some(parse_i32(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in CORSRule".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(CORSRule {
            allowed_headers,
            allowed_methods,
            allowed_origins,
            expose_headers,
            id,
            max_age_seconds,
        })
    }
}

impl S3Deserialize for CORSConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let cors_rules = deserialize_list(reader, "CORSRule")?;
        Ok(CORSConfiguration { cors_rules })
    }
}

impl S3Deserialize for ServerSideEncryptionByDefault {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut sse_algorithm = ServerSideEncryption::default();
        let mut kms_master_key_id = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "SSEAlgorithm" => {
                            let text = read_text_content(reader)?;
                            sse_algorithm = ServerSideEncryption::from(text.as_str());
                        }
                        "KMSMasterKeyID" => {
                            kms_master_key_id = Some(read_text_content(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ServerSideEncryptionByDefault".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ServerSideEncryptionByDefault {
            sse_algorithm,
            kms_master_key_id,
        })
    }
}

impl S3Deserialize for ServerSideEncryptionRule {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut apply_default = None;
        let mut bucket_key_enabled = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "ApplyServerSideEncryptionByDefault" => {
                            apply_default =
                                Some(ServerSideEncryptionByDefault::deserialize_xml(reader)?);
                        }
                        "BucketKeyEnabled" => {
                            let text = read_text_content(reader)?;
                            bucket_key_enabled = Some(parse_bool(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ServerSideEncryptionRule".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: apply_default,
            blocked_encryption_types: None,
            bucket_key_enabled,
        })
    }
}

impl S3Deserialize for ServerSideEncryptionConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let rules = deserialize_list(reader, "Rule")?;
        Ok(ServerSideEncryptionConfiguration { rules })
    }
}

impl S3Deserialize for LifecycleExpiration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut date = None;
        let mut days = None;
        let mut expired_object_delete_marker = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Date" => {
                            let text = read_text_content(reader)?;
                            date = Some(parse_timestamp(&text)?);
                        }
                        "Days" => {
                            let text = read_text_content(reader)?;
                            days = Some(parse_i32(&text)?);
                        }
                        "ExpiredObjectDeleteMarker" => {
                            let text = read_text_content(reader)?;
                            expired_object_delete_marker = Some(parse_bool(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in LifecycleExpiration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(LifecycleExpiration {
            date,
            days,
            expired_object_delete_marker,
        })
    }
}

impl S3Deserialize for Transition {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut date = None;
        let mut days = None;
        let mut storage_class = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Date" => {
                            let text = read_text_content(reader)?;
                            date = Some(parse_timestamp(&text)?);
                        }
                        "Days" => {
                            let text = read_text_content(reader)?;
                            days = Some(parse_i32(&text)?);
                        }
                        "StorageClass" => {
                            let text = read_text_content(reader)?;
                            storage_class = Some(TransitionStorageClass::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Transition".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Transition {
            date,
            days,
            storage_class,
        })
    }
}

impl S3Deserialize for NoncurrentVersionExpiration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut noncurrent_days = None;
        let mut newer_noncurrent_versions = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "NoncurrentDays" => {
                            let text = read_text_content(reader)?;
                            noncurrent_days = Some(parse_i32(&text)?);
                        }
                        "NewerNoncurrentVersions" => {
                            let text = read_text_content(reader)?;
                            newer_noncurrent_versions = Some(parse_i32(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in NoncurrentVersionExpiration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(NoncurrentVersionExpiration {
            noncurrent_days,
            newer_noncurrent_versions,
        })
    }
}

impl S3Deserialize for NoncurrentVersionTransition {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut noncurrent_days = None;
        let mut storage_class = None;
        let mut newer_noncurrent_versions = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "NoncurrentDays" => {
                            let text = read_text_content(reader)?;
                            noncurrent_days = Some(parse_i32(&text)?);
                        }
                        "StorageClass" => {
                            let text = read_text_content(reader)?;
                            storage_class = Some(TransitionStorageClass::from(text.as_str()));
                        }
                        "NewerNoncurrentVersions" => {
                            let text = read_text_content(reader)?;
                            newer_noncurrent_versions = Some(parse_i32(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in NoncurrentVersionTransition".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(NoncurrentVersionTransition {
            noncurrent_days,
            storage_class,
            newer_noncurrent_versions,
        })
    }
}

impl S3Deserialize for AbortIncompleteMultipartUpload {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut days_after_initiation = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "DaysAfterInitiation" => {
                            let text = read_text_content(reader)?;
                            days_after_initiation = Some(parse_i32(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in AbortIncompleteMultipartUpload".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(AbortIncompleteMultipartUpload {
            days_after_initiation,
        })
    }
}

impl S3Deserialize for LifecycleRuleAndOperator {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut prefix = None;
        let mut tags = Vec::new();
        let mut object_size_greater_than = None;
        let mut object_size_less_than = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Prefix" => prefix = Some(read_text_content(reader)?),
                        "Tag" => tags.push(Tag::deserialize_xml(reader)?),
                        "ObjectSizeGreaterThan" => {
                            let text = read_text_content(reader)?;
                            object_size_greater_than = Some(parse_i64(&text)?);
                        }
                        "ObjectSizeLessThan" => {
                            let text = read_text_content(reader)?;
                            object_size_less_than = Some(parse_i64(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in LifecycleRuleAndOperator".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(LifecycleRuleAndOperator {
            prefix,
            tags,
            object_size_greater_than,
            object_size_less_than,
        })
    }
}

impl S3Deserialize for LifecycleRuleFilter {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut prefix = None;
        let mut tag = None;
        let mut and = None;
        let mut object_size_greater_than = None;
        let mut object_size_less_than = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Prefix" => prefix = Some(read_text_content(reader)?),
                        "Tag" => tag = Some(Tag::deserialize_xml(reader)?),
                        "And" => and = Some(LifecycleRuleAndOperator::deserialize_xml(reader)?),
                        "ObjectSizeGreaterThan" => {
                            let text = read_text_content(reader)?;
                            object_size_greater_than = Some(parse_i64(&text)?);
                        }
                        "ObjectSizeLessThan" => {
                            let text = read_text_content(reader)?;
                            object_size_less_than = Some(parse_i64(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in LifecycleRuleFilter".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(LifecycleRuleFilter {
            prefix,
            tag,
            and,
            object_size_greater_than,
            object_size_less_than,
        })
    }
}

impl S3Deserialize for LifecycleRule {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut id = None;
        let mut prefix = None;
        let mut filter = None;
        let mut status = ExpirationStatus::default();
        let mut expiration = None;
        let mut transitions = Vec::new();
        let mut noncurrent_version_expiration = None;
        let mut noncurrent_version_transitions = Vec::new();
        let mut abort_incomplete_multipart_upload = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "ID" => id = Some(read_text_content(reader)?),
                        "Prefix" => prefix = Some(read_text_content(reader)?),
                        "Filter" => {
                            filter = Some(LifecycleRuleFilter::deserialize_xml(reader)?);
                        }
                        "Status" => {
                            let text = read_text_content(reader)?;
                            status = ExpirationStatus::from(text.as_str());
                        }
                        "Expiration" => {
                            expiration = Some(LifecycleExpiration::deserialize_xml(reader)?);
                        }
                        "Transition" => {
                            transitions.push(Transition::deserialize_xml(reader)?);
                        }
                        "NoncurrentVersionExpiration" => {
                            noncurrent_version_expiration =
                                Some(NoncurrentVersionExpiration::deserialize_xml(reader)?);
                        }
                        "NoncurrentVersionTransition" => {
                            noncurrent_version_transitions
                                .push(NoncurrentVersionTransition::deserialize_xml(reader)?);
                        }
                        "AbortIncompleteMultipartUpload" => {
                            abort_incomplete_multipart_upload =
                                Some(AbortIncompleteMultipartUpload::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in LifecycleRule".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(LifecycleRule {
            id,
            prefix,
            filter,
            status,
            expiration,
            transitions,
            noncurrent_version_expiration,
            noncurrent_version_transitions,
            abort_incomplete_multipart_upload,
        })
    }
}

impl S3Deserialize for BucketLifecycleConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let rules = deserialize_list(reader, "Rule")?;
        Ok(BucketLifecycleConfiguration { rules })
    }
}

impl S3Deserialize for FilterRule {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut name = None;
        let mut value = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let qname = e.name();
                    let tag_name = std::str::from_utf8(qname.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Name" => {
                            let text = read_text_content(reader)?;
                            name = Some(FilterRuleName::from(text.as_str()));
                        }
                        "Value" => value = Some(read_text_content(reader)?),
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in FilterRule".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(FilterRule { name, value })
    }
}

impl S3Deserialize for S3KeyFilter {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let filter_rules = deserialize_list(reader, "FilterRule")?;
        Ok(S3KeyFilter { filter_rules })
    }
}

impl S3Deserialize for NotificationConfigurationFilter {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut key = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "S3Key" => key = Some(S3KeyFilter::deserialize_xml(reader)?),
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in NotificationConfigurationFilter".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(NotificationConfigurationFilter { key })
    }
}

impl S3Deserialize for TopicConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut id = None;
        let mut topic_arn = String::new();
        let mut events = Vec::new();
        let mut filter = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Id" => id = Some(read_text_content(reader)?),
                        "Topic" => topic_arn = read_text_content(reader)?,
                        "Event" => {
                            let text = read_text_content(reader)?;
                            events.push(S3Event::from(text.as_str()));
                        }
                        "Filter" => {
                            filter =
                                Some(NotificationConfigurationFilter::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in TopicConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(TopicConfiguration {
            id,
            topic_arn,
            events,
            filter,
        })
    }
}

impl S3Deserialize for QueueConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut id = None;
        let mut queue_arn = String::new();
        let mut events = Vec::new();
        let mut filter = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Id" => id = Some(read_text_content(reader)?),
                        "Queue" => queue_arn = read_text_content(reader)?,
                        "Event" => {
                            let text = read_text_content(reader)?;
                            events.push(S3Event::from(text.as_str()));
                        }
                        "Filter" => {
                            filter =
                                Some(NotificationConfigurationFilter::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in QueueConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(QueueConfiguration {
            id,
            queue_arn,
            events,
            filter,
        })
    }
}

impl S3Deserialize for LambdaFunctionConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut id = None;
        let mut lambda_function_arn = String::new();
        let mut events = Vec::new();
        let mut filter = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Id" => id = Some(read_text_content(reader)?),
                        "CloudFunction" => lambda_function_arn = read_text_content(reader)?,
                        "Event" => {
                            let text = read_text_content(reader)?;
                            events.push(S3Event::from(text.as_str()));
                        }
                        "Filter" => {
                            filter =
                                Some(NotificationConfigurationFilter::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in LambdaFunctionConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(LambdaFunctionConfiguration {
            id,
            lambda_function_arn,
            events,
            filter,
        })
    }
}

impl S3Deserialize for EventBridgeConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        // EventBridgeConfiguration is an empty struct; just consume until end tag.
        loop {
            match reader.read_event()? {
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in EventBridgeConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }
        Ok(EventBridgeConfiguration {})
    }
}

impl S3Deserialize for NotificationConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut topic_configurations = Vec::new();
        let mut queue_configurations = Vec::new();
        let mut lambda_function_configurations = Vec::new();
        let mut event_bridge_configuration = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "TopicConfiguration" => {
                            topic_configurations.push(TopicConfiguration::deserialize_xml(reader)?);
                        }
                        "QueueConfiguration" => {
                            queue_configurations.push(QueueConfiguration::deserialize_xml(reader)?);
                        }
                        "CloudFunctionConfiguration" => {
                            lambda_function_configurations
                                .push(LambdaFunctionConfiguration::deserialize_xml(reader)?);
                        }
                        "EventBridgeConfiguration" => {
                            event_bridge_configuration =
                                Some(EventBridgeConfiguration::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in NotificationConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(NotificationConfiguration {
            topic_configurations,
            queue_configurations,
            lambda_function_configurations,
            event_bridge_configuration,
        })
    }
}

impl S3Deserialize for TargetGrant {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut grantee = None;
        let mut permission = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Grantee" => grantee = Some(Grantee::deserialize_xml(reader)?),
                        "Permission" => {
                            let text = read_text_content(reader)?;
                            permission = Some(BucketLogsPermission::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in TargetGrant".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(TargetGrant {
            grantee,
            permission,
        })
    }
}

impl S3Deserialize for SimplePrefix {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        loop {
            match reader.read_event()? {
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in SimplePrefix".to_string(),
                    ));
                }
                _ => {}
            }
        }
        Ok(SimplePrefix {})
    }
}

impl S3Deserialize for PartitionedPrefix {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut partition_date_source = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "PartitionDateSource" => {
                            let text = read_text_content(reader)?;
                            partition_date_source = Some(
                                ruststack_s3_model::types::PartitionDateSource::from(text.as_str()),
                            );
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in PartitionedPrefix".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(PartitionedPrefix {
            partition_date_source,
        })
    }
}

impl S3Deserialize for TargetObjectKeyFormat {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut simple_prefix = None;
        let mut partitioned_prefix = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "SimplePrefix" => {
                            simple_prefix = Some(SimplePrefix::deserialize_xml(reader)?);
                        }
                        "PartitionedPrefix" => {
                            partitioned_prefix = Some(PartitionedPrefix::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in TargetObjectKeyFormat".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(TargetObjectKeyFormat {
            simple_prefix,
            partitioned_prefix,
        })
    }
}

impl S3Deserialize for LoggingEnabled {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut target_bucket = String::new();
        let mut target_grants = Vec::new();
        let mut target_prefix = String::new();
        let mut target_object_key_format = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "TargetBucket" => target_bucket = read_text_content(reader)?,
                        "TargetGrants" => {
                            target_grants = deserialize_list(reader, "Grant")?;
                        }
                        "TargetPrefix" => target_prefix = read_text_content(reader)?,
                        "TargetObjectKeyFormat" => {
                            target_object_key_format =
                                Some(TargetObjectKeyFormat::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in LoggingEnabled".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(LoggingEnabled {
            target_bucket,
            target_grants,
            target_prefix,
            target_object_key_format,
        })
    }
}

impl S3Deserialize for BucketLoggingStatus {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut logging_enabled = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "LoggingEnabled" => {
                            logging_enabled = Some(LoggingEnabled::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in BucketLoggingStatus".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(BucketLoggingStatus { logging_enabled })
    }
}

impl S3Deserialize for PublicAccessBlockConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut block_public_acls = None;
        let mut block_public_policy = None;
        let mut ignore_public_acls = None;
        let mut restrict_public_buckets = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "BlockPublicAcls" => {
                            let text = read_text_content(reader)?;
                            block_public_acls = Some(parse_bool(&text)?);
                        }
                        "BlockPublicPolicy" => {
                            let text = read_text_content(reader)?;
                            block_public_policy = Some(parse_bool(&text)?);
                        }
                        "IgnorePublicAcls" => {
                            let text = read_text_content(reader)?;
                            ignore_public_acls = Some(parse_bool(&text)?);
                        }
                        "RestrictPublicBuckets" => {
                            let text = read_text_content(reader)?;
                            restrict_public_buckets = Some(parse_bool(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in PublicAccessBlockConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(PublicAccessBlockConfiguration {
            block_public_acls,
            block_public_policy,
            ignore_public_acls,
            restrict_public_buckets,
        })
    }
}

impl S3Deserialize for OwnershipControlsRule {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut object_ownership = ObjectOwnership::default();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "ObjectOwnership" => {
                            let text = read_text_content(reader)?;
                            object_ownership = ObjectOwnership::from(text.as_str());
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in OwnershipControlsRule".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(OwnershipControlsRule { object_ownership })
    }
}

impl S3Deserialize for OwnershipControls {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let rules = deserialize_list(reader, "Rule")?;
        Ok(OwnershipControls { rules })
    }
}

impl S3Deserialize for DefaultRetention {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut mode = None;
        let mut days = None;
        let mut years = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Mode" => {
                            let text = read_text_content(reader)?;
                            mode = Some(ObjectLockRetentionMode::from(text.as_str()));
                        }
                        "Days" => {
                            let text = read_text_content(reader)?;
                            days = Some(parse_i32(&text)?);
                        }
                        "Years" => {
                            let text = read_text_content(reader)?;
                            years = Some(parse_i32(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in DefaultRetention".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(DefaultRetention { mode, days, years })
    }
}

impl S3Deserialize for ObjectLockRule {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut default_retention = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "DefaultRetention" => {
                            default_retention = Some(DefaultRetention::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ObjectLockRule".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ObjectLockRule { default_retention })
    }
}

impl S3Deserialize for ObjectLockConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut object_lock_enabled = None;
        let mut rule = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "ObjectLockEnabled" => {
                            let text = read_text_content(reader)?;
                            object_lock_enabled = Some(ObjectLockEnabled::from(text.as_str()));
                        }
                        "Rule" => {
                            rule = Some(ObjectLockRule::deserialize_xml(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ObjectLockConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ObjectLockConfiguration {
            object_lock_enabled,
            rule,
        })
    }
}

impl S3Deserialize for ObjectLockRetention {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut mode = None;
        let mut retain_until_date = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Mode" => {
                            let text = read_text_content(reader)?;
                            mode = Some(ObjectLockRetentionMode::from(text.as_str()));
                        }
                        "RetainUntilDate" => {
                            let text = read_text_content(reader)?;
                            retain_until_date = Some(parse_timestamp(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ObjectLockRetention".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ObjectLockRetention {
            mode,
            retain_until_date,
        })
    }
}

impl S3Deserialize for ObjectLockLegalHold {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut status = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Status" => {
                            let text = read_text_content(reader)?;
                            status = Some(ObjectLockLegalHoldStatus::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ObjectLockLegalHold".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ObjectLockLegalHold { status })
    }
}

impl S3Deserialize for AccelerateConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut status = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Status" => {
                            let text = read_text_content(reader)?;
                            status = Some(BucketAccelerateStatus::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in AccelerateConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(AccelerateConfiguration { status })
    }
}

impl S3Deserialize for RequestPaymentConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut payer = Payer::default();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Payer" => {
                            let text = read_text_content(reader)?;
                            payer = Payer::from(text.as_str());
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in RequestPaymentConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(RequestPaymentConfiguration { payer })
    }
}

impl S3Deserialize for Condition {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut http_error_code_returned_equals = None;
        let mut key_prefix_equals = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "HttpErrorCodeReturnedEquals" => {
                            http_error_code_returned_equals = Some(read_text_content(reader)?);
                        }
                        "KeyPrefixEquals" => {
                            key_prefix_equals = Some(read_text_content(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Condition".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Condition {
            http_error_code_returned_equals,
            key_prefix_equals,
        })
    }
}

impl S3Deserialize for Redirect {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut host_name = None;
        let mut http_redirect_code = None;
        let mut protocol = None;
        let mut replace_key_prefix_with = None;
        let mut replace_key_with = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "HostName" => host_name = Some(read_text_content(reader)?),
                        "HttpRedirectCode" => {
                            http_redirect_code = Some(read_text_content(reader)?);
                        }
                        "Protocol" => {
                            let text = read_text_content(reader)?;
                            protocol = Some(Protocol::from(text.as_str()));
                        }
                        "ReplaceKeyPrefixWith" => {
                            replace_key_prefix_with = Some(read_text_content(reader)?);
                        }
                        "ReplaceKeyWith" => {
                            replace_key_with = Some(read_text_content(reader)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Redirect".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Redirect {
            host_name,
            http_redirect_code,
            protocol,
            replace_key_prefix_with,
            replace_key_with,
        })
    }
}

impl S3Deserialize for RoutingRule {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut condition = None;
        let mut redirect = Redirect::default();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Condition" => condition = Some(Condition::deserialize_xml(reader)?),
                        "Redirect" => redirect = Redirect::deserialize_xml(reader)?,
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in RoutingRule".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(RoutingRule {
            condition,
            redirect,
        })
    }
}

impl S3Deserialize for RedirectAllRequestsTo {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut host_name = String::new();
        let mut protocol = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "HostName" => host_name = read_text_content(reader)?,
                        "Protocol" => {
                            let text = read_text_content(reader)?;
                            protocol = Some(Protocol::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in RedirectAllRequestsTo".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(RedirectAllRequestsTo {
            host_name,
            protocol,
        })
    }
}

impl S3Deserialize for IndexDocument {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut suffix = String::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Suffix" => suffix = read_text_content(reader)?,
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in IndexDocument".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(IndexDocument { suffix })
    }
}

impl S3Deserialize for ErrorDocument {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut key = String::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Key" => key = read_text_content(reader)?,
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ErrorDocument".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ErrorDocument { key })
    }
}

impl S3Deserialize for WebsiteConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut error_document = None;
        let mut index_document = None;
        let mut redirect_all_requests_to = None;
        let mut routing_rules = Vec::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "ErrorDocument" => {
                            error_document = Some(ErrorDocument::deserialize_xml(reader)?);
                        }
                        "IndexDocument" => {
                            index_document = Some(IndexDocument::deserialize_xml(reader)?);
                        }
                        "RedirectAllRequestsTo" => {
                            redirect_all_requests_to =
                                Some(RedirectAllRequestsTo::deserialize_xml(reader)?);
                        }
                        "RoutingRules" => {
                            routing_rules = deserialize_list(reader, "RoutingRule")?;
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in WebsiteConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(WebsiteConfiguration {
            error_document,
            index_document,
            redirect_all_requests_to,
            routing_rules,
        })
    }
}

impl S3Deserialize for VersioningConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut status = None;
        let mut mfa_delete = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Status" => {
                            let text = read_text_content(reader)?;
                            status = Some(BucketVersioningStatus::from(text.as_str()));
                        }
                        "MfaDelete" | "MFADelete" => {
                            let text = read_text_content(reader)?;
                            mfa_delete = Some(MFADelete::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in VersioningConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(VersioningConfiguration { status, mfa_delete })
    }
}

impl S3Deserialize for CreateBucketConfiguration {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut location_constraint = None;
        let mut bucket = None;
        let mut location = None;
        let mut tags = Vec::new();

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "LocationConstraint" => {
                            let text = read_text_content(reader)?;
                            location_constraint =
                                Some(BucketLocationConstraint::from(text.as_str()));
                        }
                        "Bucket" => {
                            bucket = Some(BucketInfo::deserialize_xml(reader)?);
                        }
                        "Location" => {
                            location = Some(LocationInfo::deserialize_xml(reader)?);
                        }
                        "Tag" => tags.push(Tag::deserialize_xml(reader)?),
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in CreateBucketConfiguration".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(CreateBucketConfiguration {
            location_constraint,
            bucket,
            location,
            tags,
        })
    }
}

impl S3Deserialize for BucketInfo {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut data_redundancy = None;
        let mut bucket_type = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "DataRedundancy" => {
                            let text = read_text_content(reader)?;
                            data_redundancy = Some(DataRedundancy::from(text.as_str()));
                        }
                        "Type" => {
                            let text = read_text_content(reader)?;
                            bucket_type = Some(BucketType::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in BucketInfo".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(BucketInfo {
            data_redundancy,
            r#type: bucket_type,
        })
    }
}

impl S3Deserialize for LocationInfo {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut name = None;
        let mut loc_type = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let qname = e.name();
                    let tag_name = std::str::from_utf8(qname.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Name" => name = Some(read_text_content(reader)?),
                        "Type" => {
                            let text = read_text_content(reader)?;
                            loc_type = Some(LocationType::from(text.as_str()));
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in LocationInfo".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(LocationInfo {
            name,
            r#type: loc_type,
        })
    }
}

impl S3Deserialize for ObjectIdentifier {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut key = String::new();
        let mut version_id = None;
        let mut e_tag = None;
        let mut size = None;
        let mut last_modified_time = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Key" => key = read_text_content(reader)?,
                        "VersionId" => version_id = Some(read_text_content(reader)?),
                        "ETag" => e_tag = Some(read_text_content(reader)?),
                        "Size" => {
                            let text = read_text_content(reader)?;
                            size = Some(parse_i64(&text)?);
                        }
                        "LastModifiedTime" => {
                            let text = read_text_content(reader)?;
                            last_modified_time = Some(parse_timestamp(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in ObjectIdentifier".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(ObjectIdentifier {
            key,
            version_id,
            e_tag,
            size,
            last_modified_time,
        })
    }
}

impl S3Deserialize for Delete {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut objects = Vec::new();
        let mut quiet = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "Object" => objects.push(ObjectIdentifier::deserialize_xml(reader)?),
                        "Quiet" => {
                            let text = read_text_content(reader)?;
                            quiet = Some(parse_bool(&text)?);
                        }
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in Delete".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Delete { objects, quiet })
    }
}

impl S3Deserialize for CompletedPart {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let mut part_number = None;
        let mut e_tag = None;
        let mut checksum_crc32 = None;
        let mut checksum_crc32c = None;
        let mut checksum_crc64nvme = None;
        let mut checksum_sha1 = None;
        let mut checksum_sha256 = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    let name = e.name();
                    let tag_name = std::str::from_utf8(name.as_ref())
                        .map_err(|e| XmlError::ParseError(e.to_string()))?;
                    match tag_name {
                        "PartNumber" => {
                            let text = read_text_content(reader)?;
                            part_number = Some(parse_i32(&text)?);
                        }
                        "ETag" => e_tag = Some(read_text_content(reader)?),
                        "ChecksumCRC32" => checksum_crc32 = Some(read_text_content(reader)?),
                        "ChecksumCRC32C" => checksum_crc32c = Some(read_text_content(reader)?),
                        "ChecksumCRC64NVME" => {
                            checksum_crc64nvme = Some(read_text_content(reader)?);
                        }
                        "ChecksumSHA1" => checksum_sha1 = Some(read_text_content(reader)?),
                        "ChecksumSHA256" => checksum_sha256 = Some(read_text_content(reader)?),
                        _ => skip_element(reader)?,
                    }
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlError::UnexpectedElement(
                        "unexpected EOF in CompletedPart".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(CompletedPart {
            part_number,
            e_tag,
            checksum_crc32,
            checksum_crc32c,
            checksum_crc64nvme,
            checksum_sha1,
            checksum_sha256,
        })
    }
}

impl S3Deserialize for CompletedMultipartUpload {
    fn deserialize_xml(reader: &mut Reader<&[u8]>) -> Result<Self, XmlError> {
        let parts = deserialize_list(reader, "Part")?;
        Ok(CompletedMultipartUpload { parts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_deserialize_tagging() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <Tagging xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <TagSet>
                <Tag><Key>env</Key><Value>prod</Value></Tag>
                <Tag><Key>team</Key><Value>backend</Value></Tag>
            </TagSet>
        </Tagging>"#;

        let tagging: Tagging = from_xml(xml).expect("deserialization should succeed");
        assert_eq!(tagging.tag_set.len(), 2);
        assert_eq!(tagging.tag_set[0].key, "env");
        assert_eq!(tagging.tag_set[0].value, "prod");
        assert_eq!(tagging.tag_set[1].key, "team");
        assert_eq!(tagging.tag_set[1].value, "backend");
    }

    #[test]
    fn test_should_deserialize_versioning_configuration() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <VersioningConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Status>Enabled</Status>
        </VersioningConfiguration>"#;

        let vc: VersioningConfiguration = from_xml(xml).expect("deserialization should succeed");
        assert_eq!(vc.status, Some(BucketVersioningStatus::Enabled));
        assert!(vc.mfa_delete.is_none());
    }

    #[test]
    fn test_should_deserialize_delete_objects() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <Delete>
            <Quiet>true</Quiet>
            <Object><Key>file1.txt</Key></Object>
            <Object><Key>file2.txt</Key><VersionId>v1</VersionId></Object>
        </Delete>"#;

        let delete: Delete = from_xml(xml).expect("deserialization should succeed");
        assert_eq!(delete.quiet, Some(true));
        assert_eq!(delete.objects.len(), 2);
        assert_eq!(delete.objects[0].key, "file1.txt");
        assert!(delete.objects[0].version_id.is_none());
        assert_eq!(delete.objects[1].key, "file2.txt");
        assert_eq!(delete.objects[1].version_id.as_deref(), Some("v1"));
    }

    #[test]
    fn test_should_deserialize_completed_multipart_upload() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <CompleteMultipartUpload>
            <Part>
                <PartNumber>1</PartNumber>
                <ETag>"etag1"</ETag>
            </Part>
            <Part>
                <PartNumber>2</PartNumber>
                <ETag>"etag2"</ETag>
            </Part>
        </CompleteMultipartUpload>"#;

        let cmu: CompletedMultipartUpload = from_xml(xml).expect("deserialization should succeed");
        assert_eq!(cmu.parts.len(), 2);
        assert_eq!(cmu.parts[0].part_number, Some(1));
        assert_eq!(cmu.parts[0].e_tag.as_deref(), Some("\"etag1\""));
        assert_eq!(cmu.parts[1].part_number, Some(2));
    }

    #[test]
    fn test_should_deserialize_public_access_block() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <PublicAccessBlockConfiguration>
            <BlockPublicAcls>true</BlockPublicAcls>
            <IgnorePublicAcls>false</IgnorePublicAcls>
            <BlockPublicPolicy>true</BlockPublicPolicy>
            <RestrictPublicBuckets>false</RestrictPublicBuckets>
        </PublicAccessBlockConfiguration>"#;

        let config: PublicAccessBlockConfiguration =
            from_xml(xml).expect("deserialization should succeed");
        assert_eq!(config.block_public_acls, Some(true));
        assert_eq!(config.ignore_public_acls, Some(false));
        assert_eq!(config.block_public_policy, Some(true));
        assert_eq!(config.restrict_public_buckets, Some(false));
    }

    #[test]
    fn test_should_deserialize_create_bucket_configuration() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <CreateBucketConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <LocationConstraint>us-west-2</LocationConstraint>
        </CreateBucketConfiguration>"#;

        let config: CreateBucketConfiguration =
            from_xml(xml).expect("deserialization should succeed");
        assert_eq!(
            config.location_constraint,
            Some(BucketLocationConstraint::UsWest2)
        );
    }

    #[test]
    fn test_should_deserialize_access_control_policy() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <AccessControlPolicy>
            <Owner>
                <ID>owner-id</ID>
                <DisplayName>owner-name</DisplayName>
            </Owner>
            <AccessControlList>
                <Grant>
                    <Grantee>
                        <ID>grantee-id</ID>
                        <Type>CanonicalUser</Type>
                    </Grantee>
                    <Permission>FULL_CONTROL</Permission>
                </Grant>
            </AccessControlList>
        </AccessControlPolicy>"#;

        let acp: AccessControlPolicy = from_xml(xml).expect("deserialization should succeed");
        assert!(acp.owner.is_some());
        assert_eq!(
            acp.owner.as_ref().and_then(|o| o.id.as_deref()),
            Some("owner-id")
        );
        assert_eq!(acp.grants.len(), 1);
        assert_eq!(acp.grants[0].permission, Some(Permission::FullControl));
    }

    #[test]
    fn test_should_roundtrip_tagging() {
        let original = Tagging {
            tag_set: vec![
                Tag {
                    key: "k1".to_string(),
                    value: "v1".to_string(),
                },
                Tag {
                    key: "k2".to_string(),
                    value: "v2".to_string(),
                },
            ],
        };

        let xml =
            crate::serialize::to_xml("Tagging", &original).expect("serialization should succeed");
        let deserialized: Tagging = from_xml(&xml).expect("deserialization should succeed");

        assert_eq!(deserialized.tag_set.len(), 2);
        assert_eq!(deserialized.tag_set[0].key, "k1");
        assert_eq!(deserialized.tag_set[0].value, "v1");
        assert_eq!(deserialized.tag_set[1].key, "k2");
        assert_eq!(deserialized.tag_set[1].value, "v2");
    }

    #[test]
    fn test_should_roundtrip_versioning_configuration() {
        let original = VersioningConfiguration {
            status: Some(BucketVersioningStatus::Suspended),
            mfa_delete: Some(MFADelete::Disabled),
        };

        let xml = crate::serialize::to_xml("VersioningConfiguration", &original)
            .expect("serialization should succeed");
        let deserialized: VersioningConfiguration =
            from_xml(&xml).expect("deserialization should succeed");

        assert_eq!(deserialized.status, Some(BucketVersioningStatus::Suspended));
        assert_eq!(deserialized.mfa_delete, Some(MFADelete::Disabled));
    }
}
