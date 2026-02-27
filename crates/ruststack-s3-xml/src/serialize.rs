//! S3 XML serialization: converting Rust types to S3-compatible XML.
//!
//! This module provides the [`S3Serialize`] trait and implementations for all S3 types
//! that need to be serialized to XML for response bodies. The serialization follows the
//! AWS S3 RestXml protocol conventions:
//!
//! - Namespace: `http://s3.amazonaws.com/doc/2006-03-01/`
//! - Booleans: lowercase `true`/`false`
//! - Timestamps: ISO 8601 format (`2006-02-03T16:45:09.000Z`)
//! - XML declaration: `<?xml version="1.0" encoding="UTF-8"?>`

use std::io::{self, Write};

use quick_xml::Writer;
use quick_xml::events::{BytesText, Event};

use crate::error::XmlError;

/// The S3 XML namespace.
pub const S3_NAMESPACE: &str = "http://s3.amazonaws.com/doc/2006-03-01/";

/// Trait for serializing S3 types to XML.
///
/// Implementors write their content as child elements inside the current XML context.
/// The root element name and namespace are handled by the top-level [`to_xml`] function.
///
/// Uses `io::Result` because `quick_xml::Writer` closures require `io::Result<()>`.
pub trait S3Serialize {
    /// Serialize this value as XML child elements into the given writer.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if writing to the underlying writer fails.
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()>;
}

/// Serialize a value as S3-compatible XML with declaration and namespace.
///
/// Produces a complete XML document with:
/// - XML declaration (`<?xml version="1.0" encoding="UTF-8"?>`)
/// - Root element with the S3 namespace
/// - Serialized content from the value
///
/// # Errors
///
/// Returns `XmlError` if serialization fails.
pub fn to_xml<T: S3Serialize>(root_element: &str, value: &T) -> Result<Vec<u8>, XmlError> {
    let mut buf = Vec::with_capacity(512);
    let mut writer = Writer::new(&mut buf);

    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )))?;

    writer
        .create_element(root_element)
        .with_attribute(("xmlns", S3_NAMESPACE))
        .write_inner_content(|w| value.serialize_xml(w))?;

    Ok(buf)
}

// ---------------------------------------------------------------------------
// Helper functions for writing common XML patterns
// ---------------------------------------------------------------------------

/// Write a simple `<tag>text</tag>` element.
fn write_text_element<W: Write>(writer: &mut Writer<W>, tag: &str, text: &str) -> io::Result<()> {
    writer
        .create_element(tag)
        .write_text_content(BytesText::new(text))?;
    Ok(())
}

/// Write `<tag>text</tag>` only if the value is `Some`.
fn write_optional_text<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: Option<&str>,
) -> io::Result<()> {
    if let Some(v) = value {
        write_text_element(writer, tag, v)?;
    }
    Ok(())
}

/// Write `<tag>value</tag>` for an optional boolean.
fn write_optional_bool<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: Option<bool>,
) -> io::Result<()> {
    if let Some(v) = value {
        write_text_element(writer, tag, if v { "true" } else { "false" })?;
    }
    Ok(())
}

/// Write `<tag>value</tag>` for an optional i32.
fn write_optional_i32<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: Option<i32>,
) -> io::Result<()> {
    if let Some(v) = value {
        write_text_element(writer, tag, &v.to_string())?;
    }
    Ok(())
}

/// Write `<tag>value</tag>` for an optional i64.
fn write_optional_i64<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: Option<i64>,
) -> io::Result<()> {
    if let Some(v) = value {
        write_text_element(writer, tag, &v.to_string())?;
    }
    Ok(())
}

/// Write `<tag>value</tag>` for an optional enum that has `as_str()`.
fn write_optional_enum<W: Write, E: AsStr>(
    writer: &mut Writer<W>,
    tag: &str,
    value: Option<&E>,
) -> io::Result<()> {
    if let Some(v) = value {
        write_text_element(writer, tag, v.as_str())?;
    }
    Ok(())
}

/// Write `<tag>iso8601</tag>` for an optional timestamp.
fn write_optional_timestamp<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    value: Option<&chrono::DateTime<chrono::Utc>>,
) -> io::Result<()> {
    if let Some(v) = value {
        write_text_element(writer, tag, &format_timestamp(v))?;
    }
    Ok(())
}

/// Format a `DateTime<Utc>` as ISO 8601 with milliseconds and `Z` suffix.
fn format_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Trait for enum types that can convert to their string representation.
trait AsStr {
    fn as_str(&self) -> &'static str;
}

// Implement AsStr for all S3 enum types we need.
macro_rules! impl_as_str {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AsStr for $ty {
                fn as_str(&self) -> &'static str {
                    self.as_str()
                }
            }
        )+
    };
}

use ruststack_s3_model::types::{
    BucketAccelerateStatus, BucketLocationConstraint, BucketLogsPermission, BucketVersioningStatus,
    ChecksumAlgorithm, ChecksumType, EncodingType, Event as S3Event, ExpirationStatus,
    FilterRuleName, MFADelete, MFADeleteStatus, ObjectLockEnabled, ObjectLockLegalHoldStatus,
    ObjectLockRetentionMode, ObjectOwnership, ObjectStorageClass, ObjectVersionStorageClass, Payer,
    Permission, Protocol, StorageClass, TransitionStorageClass, Type as GranteeType,
};

impl_as_str!(
    BucketAccelerateStatus,
    BucketLocationConstraint,
    BucketLogsPermission,
    BucketVersioningStatus,
    ChecksumAlgorithm,
    ChecksumType,
    EncodingType,
    S3Event,
    ExpirationStatus,
    FilterRuleName,
    MFADelete,
    MFADeleteStatus,
    ObjectLockEnabled,
    ObjectLockLegalHoldStatus,
    ObjectLockRetentionMode,
    ObjectOwnership,
    ObjectStorageClass,
    ObjectVersionStorageClass,
    Payer,
    Permission,
    Protocol,
    StorageClass,
    TransitionStorageClass,
    GranteeType,
);

// ---------------------------------------------------------------------------
// S3Serialize implementations for shared types
// ---------------------------------------------------------------------------

use ruststack_s3_model::output::{
    CompleteMultipartUploadOutput, CopyObjectOutput, CreateMultipartUploadOutput,
    DeleteObjectsOutput, GetBucketAccelerateConfigurationOutput, GetBucketAclOutput,
    GetBucketCorsOutput, GetBucketEncryptionOutput, GetBucketLifecycleConfigurationOutput,
    GetBucketLoggingOutput, GetBucketNotificationConfigurationOutput,
    GetBucketOwnershipControlsOutput, GetBucketPolicyStatusOutput, GetBucketRequestPaymentOutput,
    GetBucketTaggingOutput, GetBucketVersioningOutput, GetBucketWebsiteOutput, GetObjectAclOutput,
    GetObjectAttributesOutput, GetObjectLegalHoldOutput, GetObjectLockConfigurationOutput,
    GetObjectRetentionOutput, GetObjectTaggingOutput, GetPublicAccessBlockOutput,
    ListBucketsOutput, ListMultipartUploadsOutput, ListObjectVersionsOutput, ListObjectsOutput,
    ListObjectsV2Output, ListPartsOutput, UploadPartCopyOutput,
};
use ruststack_s3_model::types::{
    AbortIncompleteMultipartUpload, AccelerateConfiguration, AccessControlPolicy, Bucket,
    BucketInfo, BucketLifecycleConfiguration, BucketLoggingStatus, CORSConfiguration, CORSRule,
    Checksum, CommonPrefix, CompletedMultipartUpload, CompletedPart, Condition, CopyObjectResult,
    CopyPartResult, CreateBucketConfiguration, DefaultRetention, Delete, DeleteMarkerEntry,
    DeletedObject, Error, ErrorDocument, EventBridgeConfiguration, FilterRule,
    GetObjectAttributesParts, Grant, Grantee, IndexDocument, Initiator,
    LambdaFunctionConfiguration, LifecycleExpiration, LifecycleRule, LifecycleRuleAndOperator,
    LifecycleRuleFilter, LocationInfo, LoggingEnabled, MultipartUpload,
    NoncurrentVersionExpiration, NoncurrentVersionTransition, NotificationConfiguration,
    NotificationConfigurationFilter, Object, ObjectIdentifier, ObjectLockConfiguration,
    ObjectLockLegalHold, ObjectLockRetention, ObjectLockRule, ObjectPart, ObjectVersion, Owner,
    OwnershipControls, OwnershipControlsRule, Part, PartitionedPrefix, PolicyStatus,
    PublicAccessBlockConfiguration, QueueConfiguration, Redirect, RedirectAllRequestsTo,
    RequestPaymentConfiguration, RoutingRule, S3KeyFilter, ServerSideEncryptionByDefault,
    ServerSideEncryptionConfiguration, ServerSideEncryptionRule, SimplePrefix, Tag, Tagging,
    TargetGrant, TargetObjectKeyFormat, TopicConfiguration, Transition, VersioningConfiguration,
    WebsiteConfiguration,
};

impl S3Serialize for Tag {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Tag").write_inner_content(|w| {
            write_text_element(w, "Key", &self.key)?;
            write_text_element(w, "Value", &self.value)?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for Tagging {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("TagSet").write_inner_content(|w| {
            for tag in &self.tag_set {
                tag.serialize_xml(w)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for Owner {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Owner").write_inner_content(|w| {
            write_optional_text(w, "ID", self.id.as_deref())?;
            write_optional_text(w, "DisplayName", self.display_name.as_deref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for Grantee {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("Grantee")
            .with_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"))
            .with_attribute(("xsi:type", self.r#type.as_str()))
            .write_inner_content(|w| {
                write_optional_text(w, "DisplayName", self.display_name.as_deref())?;
                write_optional_text(w, "EmailAddress", self.email_address.as_deref())?;
                write_optional_text(w, "ID", self.id.as_deref())?;
                write_optional_text(w, "URI", self.uri.as_deref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for Grant {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Grant").write_inner_content(|w| {
            if let Some(ref grantee) = self.grantee {
                grantee.serialize_xml(w)?;
            }
            write_optional_enum(w, "Permission", self.permission.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for AccessControlPolicy {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref owner) = self.owner {
            owner.serialize_xml(writer)?;
        }
        writer
            .create_element("AccessControlList")
            .write_inner_content(|w| {
                for grant in &self.grants {
                    grant.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for CORSRule {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("CORSRule").write_inner_content(|w| {
            write_optional_text(w, "ID", self.id.as_deref())?;
            for h in &self.allowed_headers {
                write_text_element(w, "AllowedHeader", h)?;
            }
            for m in &self.allowed_methods {
                write_text_element(w, "AllowedMethod", m)?;
            }
            for o in &self.allowed_origins {
                write_text_element(w, "AllowedOrigin", o)?;
            }
            for h in &self.expose_headers {
                write_text_element(w, "ExposeHeader", h)?;
            }
            write_optional_i32(w, "MaxAgeSeconds", self.max_age_seconds)?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for CORSConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for rule in &self.cors_rules {
            rule.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for ServerSideEncryptionByDefault {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("ApplyServerSideEncryptionByDefault")
            .write_inner_content(|w| {
                write_text_element(w, "SSEAlgorithm", self.sse_algorithm.as_str())?;
                write_optional_text(w, "KMSMasterKeyID", self.kms_master_key_id.as_deref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for ServerSideEncryptionRule {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Rule").write_inner_content(|w| {
            if let Some(ref default) = self.apply_server_side_encryption_by_default {
                default.serialize_xml(w)?;
            }
            write_optional_bool(w, "BucketKeyEnabled", self.bucket_key_enabled)?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for ServerSideEncryptionConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for rule in &self.rules {
            rule.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for LifecycleExpiration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("Expiration")
            .write_inner_content(|w| {
                write_optional_timestamp(w, "Date", self.date.as_ref())?;
                write_optional_i32(w, "Days", self.days)?;
                write_optional_bool(
                    w,
                    "ExpiredObjectDeleteMarker",
                    self.expired_object_delete_marker,
                )?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for Transition {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("Transition")
            .write_inner_content(|w| {
                write_optional_timestamp(w, "Date", self.date.as_ref())?;
                write_optional_i32(w, "Days", self.days)?;
                write_optional_enum(w, "StorageClass", self.storage_class.as_ref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for NoncurrentVersionExpiration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("NoncurrentVersionExpiration")
            .write_inner_content(|w| {
                write_optional_i32(w, "NoncurrentDays", self.noncurrent_days)?;
                write_optional_i32(w, "NewerNoncurrentVersions", self.newer_noncurrent_versions)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for NoncurrentVersionTransition {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("NoncurrentVersionTransition")
            .write_inner_content(|w| {
                write_optional_i32(w, "NoncurrentDays", self.noncurrent_days)?;
                write_optional_enum(w, "StorageClass", self.storage_class.as_ref())?;
                write_optional_i32(w, "NewerNoncurrentVersions", self.newer_noncurrent_versions)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for AbortIncompleteMultipartUpload {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("AbortIncompleteMultipartUpload")
            .write_inner_content(|w| {
                write_optional_i32(w, "DaysAfterInitiation", self.days_after_initiation)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for LifecycleRuleAndOperator {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("And").write_inner_content(|w| {
            write_optional_text(w, "Prefix", self.prefix.as_deref())?;
            for tag in &self.tags {
                tag.serialize_xml(w)?;
            }
            write_optional_i64(w, "ObjectSizeGreaterThan", self.object_size_greater_than)?;
            write_optional_i64(w, "ObjectSizeLessThan", self.object_size_less_than)?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for LifecycleRuleFilter {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Filter").write_inner_content(|w| {
            write_optional_text(w, "Prefix", self.prefix.as_deref())?;
            if let Some(ref tag) = self.tag {
                tag.serialize_xml(w)?;
            }
            if let Some(ref and) = self.and {
                and.serialize_xml(w)?;
            }
            write_optional_i64(w, "ObjectSizeGreaterThan", self.object_size_greater_than)?;
            write_optional_i64(w, "ObjectSizeLessThan", self.object_size_less_than)?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for LifecycleRule {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Rule").write_inner_content(|w| {
            write_optional_text(w, "ID", self.id.as_deref())?;
            write_optional_text(w, "Prefix", self.prefix.as_deref())?;
            if let Some(ref filter) = self.filter {
                filter.serialize_xml(w)?;
            }
            write_text_element(w, "Status", self.status.as_str())?;
            if let Some(ref exp) = self.expiration {
                exp.serialize_xml(w)?;
            }
            for transition in &self.transitions {
                transition.serialize_xml(w)?;
            }
            if let Some(ref nve) = self.noncurrent_version_expiration {
                nve.serialize_xml(w)?;
            }
            for nvt in &self.noncurrent_version_transitions {
                nvt.serialize_xml(w)?;
            }
            if let Some(ref abort) = self.abort_incomplete_multipart_upload {
                abort.serialize_xml(w)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for BucketLifecycleConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for rule in &self.rules {
            rule.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for FilterRule {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("FilterRule")
            .write_inner_content(|w| {
                write_optional_enum(w, "Name", self.name.as_ref())?;
                write_optional_text(w, "Value", self.value.as_deref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for S3KeyFilter {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("S3Key").write_inner_content(|w| {
            for rule in &self.filter_rules {
                rule.serialize_xml(w)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for NotificationConfigurationFilter {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Filter").write_inner_content(|w| {
            if let Some(ref key) = self.key {
                key.serialize_xml(w)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

/// Helper to serialize notification events as flattened `<Event>` elements.
fn write_events<W: Write>(writer: &mut Writer<W>, events: &[S3Event]) -> io::Result<()> {
    for event in events {
        write_text_element(writer, "Event", event.as_str())?;
    }
    Ok(())
}

impl S3Serialize for TopicConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("TopicConfiguration")
            .write_inner_content(|w| {
                write_optional_text(w, "Id", self.id.as_deref())?;
                write_text_element(w, "Topic", &self.topic_arn)?;
                write_events(w, &self.events)?;
                if let Some(ref filter) = self.filter {
                    filter.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for QueueConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("QueueConfiguration")
            .write_inner_content(|w| {
                write_optional_text(w, "Id", self.id.as_deref())?;
                write_text_element(w, "Queue", &self.queue_arn)?;
                write_events(w, &self.events)?;
                if let Some(ref filter) = self.filter {
                    filter.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for LambdaFunctionConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("CloudFunctionConfiguration")
            .write_inner_content(|w| {
                write_optional_text(w, "Id", self.id.as_deref())?;
                write_text_element(w, "CloudFunction", &self.lambda_function_arn)?;
                write_events(w, &self.events)?;
                if let Some(ref filter) = self.filter {
                    filter.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for EventBridgeConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("EventBridgeConfiguration")
            .write_empty()?;
        Ok(())
    }
}

impl S3Serialize for NotificationConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for tc in &self.topic_configurations {
            tc.serialize_xml(writer)?;
        }
        for qc in &self.queue_configurations {
            qc.serialize_xml(writer)?;
        }
        for lc in &self.lambda_function_configurations {
            lc.serialize_xml(writer)?;
        }
        if let Some(ref ebc) = self.event_bridge_configuration {
            ebc.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for TargetGrant {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Grant").write_inner_content(|w| {
            if let Some(ref grantee) = self.grantee {
                grantee.serialize_xml(w)?;
            }
            write_optional_enum(w, "Permission", self.permission.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for SimplePrefix {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("SimplePrefix").write_empty()?;
        Ok(())
    }
}

impl S3Serialize for PartitionedPrefix {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("PartitionedPrefix")
            .write_inner_content(|w| {
                if let Some(ref pds) = self.partition_date_source {
                    write_text_element(w, "PartitionDateSource", pds.as_str())?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for TargetObjectKeyFormat {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("TargetObjectKeyFormat")
            .write_inner_content(|w| {
                if let Some(ref sp) = self.simple_prefix {
                    sp.serialize_xml(w)?;
                }
                if let Some(ref pp) = self.partitioned_prefix {
                    pp.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for LoggingEnabled {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("LoggingEnabled")
            .write_inner_content(|w| {
                write_text_element(w, "TargetBucket", &self.target_bucket)?;
                if !self.target_grants.is_empty() {
                    w.create_element("TargetGrants").write_inner_content(|w2| {
                        for grant in &self.target_grants {
                            grant.serialize_xml(w2)?;
                        }
                        Ok(())
                    })?;
                }
                write_text_element(w, "TargetPrefix", &self.target_prefix)?;
                if let Some(ref fmt) = self.target_object_key_format {
                    fmt.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for BucketLoggingStatus {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref le) = self.logging_enabled {
            le.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for PublicAccessBlockConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_bool(writer, "BlockPublicAcls", self.block_public_acls)?;
        write_optional_bool(writer, "IgnorePublicAcls", self.ignore_public_acls)?;
        write_optional_bool(writer, "BlockPublicPolicy", self.block_public_policy)?;
        write_optional_bool(
            writer,
            "RestrictPublicBuckets",
            self.restrict_public_buckets,
        )?;
        Ok(())
    }
}

impl S3Serialize for OwnershipControlsRule {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Rule").write_inner_content(|w| {
            write_text_element(w, "ObjectOwnership", self.object_ownership.as_str())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for OwnershipControls {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for rule in &self.rules {
            rule.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for DefaultRetention {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("DefaultRetention")
            .write_inner_content(|w| {
                write_optional_enum(w, "Mode", self.mode.as_ref())?;
                write_optional_i32(w, "Days", self.days)?;
                write_optional_i32(w, "Years", self.years)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for ObjectLockRule {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Rule").write_inner_content(|w| {
            if let Some(ref dr) = self.default_retention {
                dr.serialize_xml(w)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for ObjectLockConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(
            writer,
            "ObjectLockEnabled",
            self.object_lock_enabled.as_ref(),
        )?;
        if let Some(ref rule) = self.rule {
            rule.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for ObjectLockRetention {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(writer, "Mode", self.mode.as_ref())?;
        write_optional_timestamp(writer, "RetainUntilDate", self.retain_until_date.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for ObjectLockLegalHold {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(writer, "Status", self.status.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for AccelerateConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(writer, "Status", self.status.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for RequestPaymentConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_text_element(writer, "Payer", self.payer.as_str())?;
        Ok(())
    }
}

impl S3Serialize for Condition {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("Condition")
            .write_inner_content(|w| {
                write_optional_text(
                    w,
                    "HttpErrorCodeReturnedEquals",
                    self.http_error_code_returned_equals.as_deref(),
                )?;
                write_optional_text(w, "KeyPrefixEquals", self.key_prefix_equals.as_deref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for Redirect {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Redirect").write_inner_content(|w| {
            write_optional_text(w, "HostName", self.host_name.as_deref())?;
            write_optional_text(w, "HttpRedirectCode", self.http_redirect_code.as_deref())?;
            write_optional_enum(w, "Protocol", self.protocol.as_ref())?;
            write_optional_text(
                w,
                "ReplaceKeyPrefixWith",
                self.replace_key_prefix_with.as_deref(),
            )?;
            write_optional_text(w, "ReplaceKeyWith", self.replace_key_with.as_deref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for RoutingRule {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("RoutingRule")
            .write_inner_content(|w| {
                if let Some(ref cond) = self.condition {
                    cond.serialize_xml(w)?;
                }
                self.redirect.serialize_xml(w)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for RedirectAllRequestsTo {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("RedirectAllRequestsTo")
            .write_inner_content(|w| {
                write_text_element(w, "HostName", &self.host_name)?;
                write_optional_enum(w, "Protocol", self.protocol.as_ref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for IndexDocument {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("IndexDocument")
            .write_inner_content(|w| {
                write_text_element(w, "Suffix", &self.suffix)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for ErrorDocument {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("ErrorDocument")
            .write_inner_content(|w| {
                write_text_element(w, "Key", &self.key)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for WebsiteConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref rart) = self.redirect_all_requests_to {
            rart.serialize_xml(writer)?;
        }
        if let Some(ref idx) = self.index_document {
            idx.serialize_xml(writer)?;
        }
        if let Some(ref err) = self.error_document {
            err.serialize_xml(writer)?;
        }
        if !self.routing_rules.is_empty() {
            writer
                .create_element("RoutingRules")
                .write_inner_content(|w| {
                    for rule in &self.routing_rules {
                        rule.serialize_xml(w)?;
                    }
                    Ok(())
                })?;
        }
        Ok(())
    }
}

impl S3Serialize for VersioningConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(writer, "Status", self.status.as_ref())?;
        write_optional_enum(writer, "MfaDelete", self.mfa_delete.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for BucketInfo {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Bucket").write_inner_content(|w| {
            if let Some(ref dr) = self.data_redundancy {
                write_text_element(w, "DataRedundancy", dr.as_str())?;
            }
            if let Some(ref t) = self.r#type {
                write_text_element(w, "Type", t.as_str())?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for LocationInfo {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Location").write_inner_content(|w| {
            write_optional_text(w, "Name", self.name.as_deref())?;
            if let Some(ref t) = self.r#type {
                write_text_element(w, "Type", t.as_str())?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for CreateBucketConfiguration {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(
            writer,
            "LocationConstraint",
            self.location_constraint.as_ref(),
        )?;
        if let Some(ref bucket) = self.bucket {
            bucket.serialize_xml(writer)?;
        }
        if let Some(ref location) = self.location {
            location.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for ObjectIdentifier {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Object").write_inner_content(|w| {
            write_text_element(w, "Key", &self.key)?;
            write_optional_text(w, "VersionId", self.version_id.as_deref())?;
            write_optional_text(w, "ETag", self.e_tag.as_deref())?;
            write_optional_i64(w, "Size", self.size)?;
            write_optional_timestamp(w, "LastModifiedTime", self.last_modified_time.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for Delete {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_bool(writer, "Quiet", self.quiet)?;
        for obj in &self.objects {
            obj.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for CompletedPart {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Part").write_inner_content(|w| {
            write_optional_i32(w, "PartNumber", self.part_number)?;
            write_optional_text(w, "ETag", self.e_tag.as_deref())?;
            write_optional_text(w, "ChecksumCRC32", self.checksum_crc32.as_deref())?;
            write_optional_text(w, "ChecksumCRC32C", self.checksum_crc32c.as_deref())?;
            write_optional_text(w, "ChecksumCRC64NVME", self.checksum_crc64nvme.as_deref())?;
            write_optional_text(w, "ChecksumSHA1", self.checksum_sha1.as_deref())?;
            write_optional_text(w, "ChecksumSHA256", self.checksum_sha256.as_deref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for CompletedMultipartUpload {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for part in &self.parts {
            part.serialize_xml(writer)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Serialization-only types (response output types)
// ---------------------------------------------------------------------------

impl S3Serialize for Bucket {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Bucket").write_inner_content(|w| {
            write_optional_text(w, "Name", self.name.as_deref())?;
            write_optional_timestamp(w, "CreationDate", self.creation_date.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for CommonPrefix {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("CommonPrefixes")
            .write_inner_content(|w| {
                write_optional_text(w, "Prefix", self.prefix.as_deref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for Object {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Contents").write_inner_content(|w| {
            write_optional_text(w, "Key", self.key.as_deref())?;
            write_optional_timestamp(w, "LastModified", self.last_modified.as_ref())?;
            write_optional_text(w, "ETag", self.e_tag.as_deref())?;
            write_optional_i64(w, "Size", self.size)?;
            write_optional_enum(w, "StorageClass", self.storage_class.as_ref())?;
            if let Some(ref owner) = self.owner {
                owner.serialize_xml(w)?;
            }
            for algo in &self.checksum_algorithm {
                write_text_element(w, "ChecksumAlgorithm", algo.as_str())?;
            }
            write_optional_enum(w, "ChecksumType", self.checksum_type.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for ObjectVersion {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Version").write_inner_content(|w| {
            write_optional_text(w, "Key", self.key.as_deref())?;
            write_optional_text(w, "VersionId", self.version_id.as_deref())?;
            write_optional_bool(w, "IsLatest", self.is_latest)?;
            write_optional_timestamp(w, "LastModified", self.last_modified.as_ref())?;
            write_optional_text(w, "ETag", self.e_tag.as_deref())?;
            write_optional_i64(w, "Size", self.size)?;
            write_optional_enum(w, "StorageClass", self.storage_class.as_ref())?;
            if let Some(ref owner) = self.owner {
                owner.serialize_xml(w)?;
            }
            for algo in &self.checksum_algorithm {
                write_text_element(w, "ChecksumAlgorithm", algo.as_str())?;
            }
            write_optional_enum(w, "ChecksumType", self.checksum_type.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for DeleteMarkerEntry {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("DeleteMarker")
            .write_inner_content(|w| {
                write_optional_text(w, "Key", self.key.as_deref())?;
                write_optional_text(w, "VersionId", self.version_id.as_deref())?;
                write_optional_bool(w, "IsLatest", self.is_latest)?;
                write_optional_timestamp(w, "LastModified", self.last_modified.as_ref())?;
                if let Some(ref owner) = self.owner {
                    owner.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for DeletedObject {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Deleted").write_inner_content(|w| {
            write_optional_text(w, "Key", self.key.as_deref())?;
            write_optional_text(
                w,
                "DeleteMarkerVersionId",
                self.delete_marker_version_id.as_deref(),
            )?;
            write_optional_bool(w, "DeleteMarker", self.delete_marker)?;
            write_optional_text(w, "VersionId", self.version_id.as_deref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for Error {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Error").write_inner_content(|w| {
            write_optional_text(w, "Key", self.key.as_deref())?;
            write_optional_text(w, "VersionId", self.version_id.as_deref())?;
            write_optional_text(w, "Code", self.code.as_deref())?;
            write_optional_text(w, "Message", self.message.as_deref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for CopyObjectResult {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "ETag", self.e_tag.as_deref())?;
        write_optional_timestamp(writer, "LastModified", self.last_modified.as_ref())?;
        write_optional_text(writer, "ChecksumCRC32", self.checksum_crc32.as_deref())?;
        write_optional_text(writer, "ChecksumCRC32C", self.checksum_crc32c.as_deref())?;
        write_optional_text(
            writer,
            "ChecksumCRC64NVME",
            self.checksum_crc64nvme.as_deref(),
        )?;
        write_optional_text(writer, "ChecksumSHA1", self.checksum_sha1.as_deref())?;
        write_optional_text(writer, "ChecksumSHA256", self.checksum_sha256.as_deref())?;
        write_optional_enum(writer, "ChecksumType", self.checksum_type.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for Part {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Part").write_inner_content(|w| {
            write_optional_i32(w, "PartNumber", self.part_number)?;
            write_optional_timestamp(w, "LastModified", self.last_modified.as_ref())?;
            write_optional_text(w, "ETag", self.e_tag.as_deref())?;
            write_optional_i64(w, "Size", self.size)?;
            write_optional_text(w, "ChecksumCRC32", self.checksum_crc32.as_deref())?;
            write_optional_text(w, "ChecksumCRC32C", self.checksum_crc32c.as_deref())?;
            write_optional_text(w, "ChecksumCRC64NVME", self.checksum_crc64nvme.as_deref())?;
            write_optional_text(w, "ChecksumSHA1", self.checksum_sha1.as_deref())?;
            write_optional_text(w, "ChecksumSHA256", self.checksum_sha256.as_deref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for ObjectPart {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Part").write_inner_content(|w| {
            write_optional_i32(w, "PartNumber", self.part_number)?;
            write_optional_i64(w, "Size", self.size)?;
            write_optional_text(w, "ChecksumCRC32", self.checksum_crc32.as_deref())?;
            write_optional_text(w, "ChecksumCRC32C", self.checksum_crc32c.as_deref())?;
            write_optional_text(w, "ChecksumCRC64NVME", self.checksum_crc64nvme.as_deref())?;
            write_optional_text(w, "ChecksumSHA1", self.checksum_sha1.as_deref())?;
            write_optional_text(w, "ChecksumSHA256", self.checksum_sha256.as_deref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for Initiator {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("Initiator")
            .write_inner_content(|w| {
                write_optional_text(w, "ID", self.id.as_deref())?;
                write_optional_text(w, "DisplayName", self.display_name.as_deref())?;
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for MultipartUpload {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Upload").write_inner_content(|w| {
            write_optional_text(w, "Key", self.key.as_deref())?;
            write_optional_text(w, "UploadId", self.upload_id.as_deref())?;
            if let Some(ref initiator) = self.initiator {
                initiator.serialize_xml(w)?;
            }
            if let Some(ref owner) = self.owner {
                owner.serialize_xml(w)?;
            }
            write_optional_enum(w, "StorageClass", self.storage_class.as_ref())?;
            write_optional_timestamp(w, "Initiated", self.initiated.as_ref())?;
            write_optional_enum(w, "ChecksumAlgorithm", self.checksum_algorithm.as_ref())?;
            write_optional_enum(w, "ChecksumType", self.checksum_type.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for PolicyStatus {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_bool(writer, "IsPublic", self.is_public)?;
        Ok(())
    }
}

impl S3Serialize for Checksum {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("Checksum").write_inner_content(|w| {
            write_optional_text(w, "ChecksumCRC32", self.checksum_crc32.as_deref())?;
            write_optional_text(w, "ChecksumCRC32C", self.checksum_crc32c.as_deref())?;
            write_optional_text(w, "ChecksumCRC64NVME", self.checksum_crc64nvme.as_deref())?;
            write_optional_text(w, "ChecksumSHA1", self.checksum_sha1.as_deref())?;
            write_optional_text(w, "ChecksumSHA256", self.checksum_sha256.as_deref())?;
            write_optional_enum(w, "ChecksumType", self.checksum_type.as_ref())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for GetObjectAttributesParts {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer
            .create_element("ObjectParts")
            .write_inner_content(|w| {
                write_optional_i32(w, "TotalPartsCount", self.total_parts_count)?;
                write_optional_text(w, "PartNumberMarker", self.part_number_marker.as_deref())?;
                write_optional_text(
                    w,
                    "NextPartNumberMarker",
                    self.next_part_number_marker.as_deref(),
                )?;
                write_optional_i32(w, "MaxParts", self.max_parts)?;
                write_optional_bool(w, "IsTruncated", self.is_truncated)?;
                for part in &self.parts {
                    part.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// S3Serialize implementations for output types
// ---------------------------------------------------------------------------

impl S3Serialize for ListBucketsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref owner) = self.owner {
            owner.serialize_xml(writer)?;
        }
        writer.create_element("Buckets").write_inner_content(|w| {
            for bucket in &self.buckets {
                bucket.serialize_xml(w)?;
            }
            Ok(())
        })?;
        write_optional_text(
            writer,
            "ContinuationToken",
            self.continuation_token.as_deref(),
        )?;
        write_optional_text(writer, "Prefix", self.prefix.as_deref())?;
        Ok(())
    }
}

impl S3Serialize for ListObjectsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "Name", self.name.as_deref())?;
        write_optional_text(writer, "Prefix", self.prefix.as_deref())?;
        write_optional_text(writer, "Marker", self.marker.as_deref())?;
        write_optional_i32(writer, "MaxKeys", self.max_keys)?;
        write_optional_text(writer, "Delimiter", self.delimiter.as_deref())?;
        write_optional_bool(writer, "IsTruncated", self.is_truncated)?;
        write_optional_enum(writer, "EncodingType", self.encoding_type.as_ref())?;
        write_optional_text(writer, "NextMarker", self.next_marker.as_deref())?;
        for obj in &self.contents {
            obj.serialize_xml(writer)?;
        }
        for cp in &self.common_prefixes {
            cp.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for ListObjectsV2Output {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "Name", self.name.as_deref())?;
        write_optional_text(writer, "Prefix", self.prefix.as_deref())?;
        write_optional_i32(writer, "KeyCount", self.key_count)?;
        write_optional_i32(writer, "MaxKeys", self.max_keys)?;
        write_optional_text(writer, "Delimiter", self.delimiter.as_deref())?;
        write_optional_bool(writer, "IsTruncated", self.is_truncated)?;
        write_optional_text(
            writer,
            "ContinuationToken",
            self.continuation_token.as_deref(),
        )?;
        write_optional_text(
            writer,
            "NextContinuationToken",
            self.next_continuation_token.as_deref(),
        )?;
        write_optional_text(writer, "StartAfter", self.start_after.as_deref())?;
        write_optional_enum(writer, "EncodingType", self.encoding_type.as_ref())?;
        for obj in &self.contents {
            obj.serialize_xml(writer)?;
        }
        for cp in &self.common_prefixes {
            cp.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for ListObjectVersionsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "Name", self.name.as_deref())?;
        write_optional_text(writer, "Prefix", self.prefix.as_deref())?;
        write_optional_text(writer, "KeyMarker", self.key_marker.as_deref())?;
        write_optional_text(writer, "VersionIdMarker", self.version_id_marker.as_deref())?;
        write_optional_text(writer, "NextKeyMarker", self.next_key_marker.as_deref())?;
        write_optional_text(
            writer,
            "NextVersionIdMarker",
            self.next_version_id_marker.as_deref(),
        )?;
        write_optional_i32(writer, "MaxKeys", self.max_keys)?;
        write_optional_text(writer, "Delimiter", self.delimiter.as_deref())?;
        write_optional_bool(writer, "IsTruncated", self.is_truncated)?;
        write_optional_enum(writer, "EncodingType", self.encoding_type.as_ref())?;
        for version in &self.versions {
            version.serialize_xml(writer)?;
        }
        for dm in &self.delete_markers {
            dm.serialize_xml(writer)?;
        }
        for cp in &self.common_prefixes {
            cp.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for ListPartsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "Bucket", self.bucket.as_deref())?;
        write_optional_text(writer, "Key", self.key.as_deref())?;
        write_optional_text(writer, "UploadId", self.upload_id.as_deref())?;
        write_optional_text(
            writer,
            "PartNumberMarker",
            self.part_number_marker.as_deref(),
        )?;
        write_optional_text(
            writer,
            "NextPartNumberMarker",
            self.next_part_number_marker.as_deref(),
        )?;
        write_optional_i32(writer, "MaxParts", self.max_parts)?;
        write_optional_bool(writer, "IsTruncated", self.is_truncated)?;
        if let Some(ref initiator) = self.initiator {
            initiator.serialize_xml(writer)?;
        }
        if let Some(ref owner) = self.owner {
            owner.serialize_xml(writer)?;
        }
        write_optional_enum(writer, "StorageClass", self.storage_class.as_ref())?;
        write_optional_enum(
            writer,
            "ChecksumAlgorithm",
            self.checksum_algorithm.as_ref(),
        )?;
        write_optional_enum(writer, "ChecksumType", self.checksum_type.as_ref())?;
        for part in &self.parts {
            part.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for ListMultipartUploadsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "Bucket", self.bucket.as_deref())?;
        write_optional_text(writer, "KeyMarker", self.key_marker.as_deref())?;
        write_optional_text(writer, "UploadIdMarker", self.upload_id_marker.as_deref())?;
        write_optional_text(writer, "NextKeyMarker", self.next_key_marker.as_deref())?;
        write_optional_text(
            writer,
            "NextUploadIdMarker",
            self.next_upload_id_marker.as_deref(),
        )?;
        write_optional_i32(writer, "MaxUploads", self.max_uploads)?;
        write_optional_text(writer, "Delimiter", self.delimiter.as_deref())?;
        write_optional_text(writer, "Prefix", self.prefix.as_deref())?;
        write_optional_bool(writer, "IsTruncated", self.is_truncated)?;
        write_optional_enum(writer, "EncodingType", self.encoding_type.as_ref())?;
        for upload in &self.uploads {
            upload.serialize_xml(writer)?;
        }
        for cp in &self.common_prefixes {
            cp.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for DeleteObjectsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for deleted in &self.deleted {
            deleted.serialize_xml(writer)?;
        }
        for error in &self.errors {
            error.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketVersioningOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(writer, "Status", self.status.as_ref())?;
        write_optional_enum(writer, "MfaDelete", self.mfa_delete.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for GetBucketCorsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for rule in &self.cors_rules {
            rule.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketAclOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref owner) = self.owner {
            owner.serialize_xml(writer)?;
        }
        writer
            .create_element("AccessControlList")
            .write_inner_content(|w| {
                for grant in &self.grants {
                    grant.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for GetBucketEncryptionOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref config) = self.server_side_encryption_configuration {
            config.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketLoggingOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref le) = self.logging_enabled {
            le.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketNotificationConfigurationOutput {
    fn serialize_xml<W: Write>(&self, _writer: &mut Writer<W>) -> io::Result<()> {
        Ok(())
    }
}

impl S3Serialize for GetBucketOwnershipControlsOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref controls) = self.ownership_controls {
            controls.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketPolicyStatusOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref status) = self.policy_status {
            status.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketRequestPaymentOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(writer, "Payer", self.payer.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for GetBucketTaggingOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("TagSet").write_inner_content(|w| {
            for tag in &self.tag_set {
                tag.serialize_xml(w)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for GetBucketWebsiteOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref rart) = self.redirect_all_requests_to {
            rart.serialize_xml(writer)?;
        }
        if let Some(ref idx) = self.index_document {
            idx.serialize_xml(writer)?;
        }
        if let Some(ref err) = self.error_document {
            err.serialize_xml(writer)?;
        }
        if !self.routing_rules.is_empty() {
            writer
                .create_element("RoutingRules")
                .write_inner_content(|w| {
                    for rule in &self.routing_rules {
                        rule.serialize_xml(w)?;
                    }
                    Ok(())
                })?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketLifecycleConfigurationOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        for rule in &self.rules {
            rule.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetBucketAccelerateConfigurationOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_enum(writer, "Status", self.status.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for GetObjectAclOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref owner) = self.owner {
            owner.serialize_xml(writer)?;
        }
        writer
            .create_element("AccessControlList")
            .write_inner_content(|w| {
                for grant in &self.grants {
                    grant.serialize_xml(w)?;
                }
                Ok(())
            })?;
        Ok(())
    }
}

impl S3Serialize for GetObjectLegalHoldOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref hold) = self.legal_hold {
            hold.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetObjectLockConfigurationOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref config) = self.object_lock_configuration {
            config.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetObjectRetentionOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref retention) = self.retention {
            retention.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetPublicAccessBlockOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref config) = self.public_access_block_configuration {
            config.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for GetObjectTaggingOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        writer.create_element("TagSet").write_inner_content(|w| {
            for tag in &self.tag_set {
                tag.serialize_xml(w)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl S3Serialize for GetObjectAttributesOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "ETag", self.e_tag.as_deref())?;
        write_optional_i64(writer, "ObjectSize", self.object_size)?;
        write_optional_enum(writer, "StorageClass", self.storage_class.as_ref())?;
        if let Some(ref checksum) = self.checksum {
            checksum.serialize_xml(writer)?;
        }
        if let Some(ref parts) = self.object_parts {
            parts.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for CopyObjectOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref result) = self.copy_object_result {
            result.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for CreateMultipartUploadOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "Bucket", self.bucket.as_deref())?;
        write_optional_text(writer, "Key", self.key.as_deref())?;
        write_optional_text(writer, "UploadId", self.upload_id.as_deref())?;
        Ok(())
    }
}

impl S3Serialize for CompleteMultipartUploadOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "Location", self.location.as_deref())?;
        write_optional_text(writer, "Bucket", self.bucket.as_deref())?;
        write_optional_text(writer, "Key", self.key.as_deref())?;
        write_optional_text(writer, "ETag", self.e_tag.as_deref())?;
        write_optional_text(writer, "ChecksumCRC32", self.checksum_crc32.as_deref())?;
        write_optional_text(writer, "ChecksumCRC32C", self.checksum_crc32c.as_deref())?;
        write_optional_text(
            writer,
            "ChecksumCRC64NVME",
            self.checksum_crc64nvme.as_deref(),
        )?;
        write_optional_text(writer, "ChecksumSHA1", self.checksum_sha1.as_deref())?;
        write_optional_text(writer, "ChecksumSHA256", self.checksum_sha256.as_deref())?;
        write_optional_enum(writer, "ChecksumType", self.checksum_type.as_ref())?;
        Ok(())
    }
}

impl S3Serialize for UploadPartCopyOutput {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        if let Some(ref result) = self.copy_part_result {
            result.serialize_xml(writer)?;
        }
        Ok(())
    }
}

impl S3Serialize for CopyPartResult {
    fn serialize_xml<W: Write>(&self, writer: &mut Writer<W>) -> io::Result<()> {
        write_optional_text(writer, "ETag", self.e_tag.as_deref())?;
        write_optional_timestamp(writer, "LastModified", self.last_modified.as_ref())?;
        write_optional_text(writer, "ChecksumCRC32", self.checksum_crc32.as_deref())?;
        write_optional_text(writer, "ChecksumCRC32C", self.checksum_crc32c.as_deref())?;
        write_optional_text(
            writer,
            "ChecksumCRC64NVME",
            self.checksum_crc64nvme.as_deref(),
        )?;
        write_optional_text(writer, "ChecksumSHA1", self.checksum_sha1.as_deref())?;
        write_optional_text(writer, "ChecksumSHA256", self.checksum_sha256.as_deref())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_serialize_tagging() {
        let tagging = Tagging {
            tag_set: vec![
                Tag {
                    key: "env".to_string(),
                    value: "prod".to_string(),
                },
                Tag {
                    key: "team".to_string(),
                    value: "backend".to_string(),
                },
            ],
        };

        let xml = to_xml("Tagging", &tagging).expect("serialization should succeed");
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.contains("<Tagging xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">"));
        assert!(xml_str.contains("<TagSet>"));
        assert!(xml_str.contains("<Tag><Key>env</Key><Value>prod</Value></Tag>"));
        assert!(xml_str.contains("</Tagging>"));
    }

    #[test]
    fn test_should_serialize_versioning_configuration() {
        let vc = VersioningConfiguration {
            status: Some(BucketVersioningStatus::Enabled),
            mfa_delete: None,
        };

        let xml = to_xml("VersioningConfiguration", &vc).expect("serialization should succeed");
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.contains("<Status>Enabled</Status>"));
        assert!(!xml_str.contains("MfaDelete"));
    }

    #[test]
    fn test_should_serialize_public_access_block() {
        let config = PublicAccessBlockConfiguration {
            block_public_acls: Some(true),
            block_public_policy: Some(true),
            ignore_public_acls: Some(false),
            restrict_public_buckets: Some(false),
        };

        let xml = to_xml("PublicAccessBlockConfiguration", &config)
            .expect("serialization should succeed");
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.contains("<BlockPublicAcls>true</BlockPublicAcls>"));
        assert!(xml_str.contains("<BlockPublicPolicy>true</BlockPublicPolicy>"));
        assert!(xml_str.contains("<IgnorePublicAcls>false</IgnorePublicAcls>"));
        assert!(xml_str.contains("<RestrictPublicBuckets>false</RestrictPublicBuckets>"));
    }

    #[test]
    fn test_should_serialize_delete_request() {
        let delete = Delete {
            objects: vec![
                ObjectIdentifier {
                    key: "file1.txt".to_string(),
                    ..Default::default()
                },
                ObjectIdentifier {
                    key: "file2.txt".to_string(),
                    version_id: Some("v1".to_string()),
                    ..Default::default()
                },
            ],
            quiet: Some(true),
        };

        let xml = to_xml("Delete", &delete).expect("serialization should succeed");
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.contains("<Quiet>true</Quiet>"));
        assert!(xml_str.contains("<Object><Key>file1.txt</Key></Object>"));
        assert!(xml_str.contains("<Key>file2.txt</Key>"));
        assert!(xml_str.contains("<VersionId>v1</VersionId>"));
    }

    #[test]
    fn test_should_serialize_xml_special_characters() {
        let tagging = Tagging {
            tag_set: vec![Tag {
                key: "key<>".to_string(),
                value: "val&\"".to_string(),
            }],
        };

        let xml = to_xml("Tagging", &tagging).expect("serialization should succeed");
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.contains("key&lt;&gt;"));
        assert!(xml_str.contains("val&amp;&quot;"));
    }
}
