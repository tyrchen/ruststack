//! Bucket configuration operation handlers.
//!
//! Implements versioning, encryption, CORS, lifecycle, policy, tagging,
//! notification, logging, public access block, ownership controls,
//! object lock, accelerate, request payment, website, ACL, and
//! policy status operations.

// The s3s DTO module contains dozens of types we reference; wildcard is clearer.
#[allow(clippy::wildcard_imports)]
use s3s::dto::*;
use s3s::{S3Request, S3Response, S3Result};
use tracing::debug;

use crate::cors::CorsRule;
use crate::error::S3ServiceError;
use crate::provider::RustStackS3;
use crate::state::bucket::{
    BucketEncryption, CorsRuleConfig, ObjectLockConfiguration, ObjectLockRule,
    OwnershipControlsConfig, PublicAccessBlockConfig, VersioningStatus,
};
use crate::state::object::{CannedAcl, Owner as InternalOwner};

// These handler methods must remain async to match the s3s::S3 trait interface.
#[allow(clippy::unused_async)]
impl RustStackS3 {
    // -----------------------------------------------------------------------
    // Versioning
    // -----------------------------------------------------------------------

    /// Get the versioning configuration for a bucket.
    pub(crate) async fn handle_get_bucket_versioning(
        &self,
        req: S3Request<GetBucketVersioningInput>,
    ) -> S3Result<S3Response<GetBucketVersioningOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let status = match *bucket.versioning.read() {
            VersioningStatus::Enabled => Some(BucketVersioningStatus::from_static("Enabled")),
            VersioningStatus::Suspended => Some(BucketVersioningStatus::from_static("Suspended")),
            VersioningStatus::Disabled => None,
        };

        let output = GetBucketVersioningOutput {
            mfa_delete: None,
            status,
        };
        Ok(S3Response::new(output))
    }

    /// Set the versioning configuration for a bucket.
    pub(crate) async fn handle_put_bucket_versioning(
        &self,
        req: S3Request<PutBucketVersioningInput>,
    ) -> S3Result<S3Response<PutBucketVersioningOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let config = req.input.versioning_configuration;
        if let Some(status) = config.status {
            match status.as_str() {
                "Enabled" => bucket.enable_versioning(),
                "Suspended" => bucket.suspend_versioning(),
                _ => {
                    return Err(s3s::s3_error!(InvalidArgument, "Invalid versioning status"));
                }
            }
        }

        debug!(bucket = %bucket_name, "put_bucket_versioning completed");
        Ok(S3Response::new(PutBucketVersioningOutput {}))
    }

    // -----------------------------------------------------------------------
    // Encryption
    // -----------------------------------------------------------------------

    /// Get the server-side encryption configuration for a bucket.
    pub(crate) async fn handle_get_bucket_encryption(
        &self,
        req: S3Request<GetBucketEncryptionInput>,
    ) -> S3Result<S3Response<GetBucketEncryptionOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let enc = bucket.encryption.read();
        let enc_config = enc
            .as_ref()
            .ok_or(S3ServiceError::ServerSideEncryptionConfigurationNotFoundError)
            .map_err(S3ServiceError::into_s3_error)?;

        let rule = ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: Some(ServerSideEncryptionByDefault {
                kms_master_key_id: enc_config.kms_master_key_id.clone(),
                sse_algorithm: ServerSideEncryption::from(enc_config.sse_algorithm.clone()),
            }),
            bucket_key_enabled: Some(enc_config.bucket_key_enabled),
        };

        let output = GetBucketEncryptionOutput {
            server_side_encryption_configuration: Some(ServerSideEncryptionConfiguration {
                rules: vec![rule],
            }),
        };
        Ok(S3Response::new(output))
    }

    /// Set the server-side encryption configuration for a bucket.
    pub(crate) async fn handle_put_bucket_encryption(
        &self,
        req: S3Request<PutBucketEncryptionInput>,
    ) -> S3Result<S3Response<PutBucketEncryptionOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let config = req.input.server_side_encryption_configuration;

        let rule = config
            .rules
            .first()
            .ok_or_else(|| s3s::s3_error!(InvalidArgument, "At least one rule is required"))?;

        let default = rule
            .apply_server_side_encryption_by_default
            .as_ref()
            .ok_or_else(|| {
                s3s::s3_error!(
                    InvalidArgument,
                    "ApplyServerSideEncryptionByDefault is required"
                )
            })?;

        let enc = BucketEncryption {
            sse_algorithm: default.sse_algorithm.as_str().to_owned(),
            kms_master_key_id: default.kms_master_key_id.clone(),
            bucket_key_enabled: rule.bucket_key_enabled.unwrap_or(false),
        };

        *bucket.encryption.write() = Some(enc);

        debug!(bucket = %bucket_name, "put_bucket_encryption completed");
        Ok(S3Response::new(PutBucketEncryptionOutput {}))
    }

    /// Delete the server-side encryption configuration for a bucket.
    pub(crate) async fn handle_delete_bucket_encryption(
        &self,
        req: S3Request<DeleteBucketEncryptionInput>,
    ) -> S3Result<S3Response<DeleteBucketEncryptionOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.encryption.write() = None;

        debug!(bucket = %bucket_name, "delete_bucket_encryption completed");
        Ok(S3Response::new(DeleteBucketEncryptionOutput {}))
    }

    // -----------------------------------------------------------------------
    // CORS
    // -----------------------------------------------------------------------

    /// Get CORS configuration for a bucket.
    pub(crate) async fn handle_get_bucket_cors(
        &self,
        req: S3Request<GetBucketCorsInput>,
    ) -> S3Result<S3Response<GetBucketCorsOutput>> {
        let bucket_name = req.input.bucket;

        // Verify bucket exists.
        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let cors_rules = bucket.cors_rules.read();
        let rules = cors_rules
            .as_ref()
            .ok_or(S3ServiceError::NoSuchCorsConfiguration)
            .map_err(S3ServiceError::into_s3_error)?;

        let s3_rules: Vec<CORSRule> = rules.iter().map(cors_config_to_dto).collect();

        let output = GetBucketCorsOutput {
            cors_rules: Some(s3_rules),
        };
        Ok(S3Response::new(output))
    }

    /// Set CORS configuration for a bucket.
    pub(crate) async fn handle_put_bucket_cors(
        &self,
        req: S3Request<PutBucketCorsInput>,
    ) -> S3Result<S3Response<PutBucketCorsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let cors_config = req.input.cors_configuration;

        let configs: Vec<CorsRuleConfig> = cors_config
            .cors_rules
            .iter()
            .map(dto_to_cors_config)
            .collect();
        let index_rules: Vec<CorsRule> = configs
            .iter()
            .map(|c| CorsRule {
                allowed_origins: c.allowed_origins.clone(),
                allowed_methods: c.allowed_methods.clone(),
                allowed_headers: c.allowed_headers.clone(),
                expose_headers: c.expose_headers.clone(),
                max_age_seconds: c.max_age_seconds,
            })
            .collect();

        *bucket.cors_rules.write() = Some(configs);
        self.cors_index.set_rules(&bucket_name, index_rules);

        debug!(bucket = %bucket_name, "put_bucket_cors completed");
        Ok(S3Response::new(PutBucketCorsOutput {}))
    }

    /// Delete CORS configuration for a bucket.
    pub(crate) async fn handle_delete_bucket_cors(
        &self,
        req: S3Request<DeleteBucketCorsInput>,
    ) -> S3Result<S3Response<DeleteBucketCorsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.cors_rules.write() = None;
        self.cors_index.delete_rules(&bucket_name);

        debug!(bucket = %bucket_name, "delete_bucket_cors completed");
        Ok(S3Response::new(DeleteBucketCorsOutput {}))
    }

    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Get lifecycle configuration for a bucket.
    pub(crate) async fn handle_get_bucket_lifecycle_configuration(
        &self,
        req: S3Request<GetBucketLifecycleConfigurationInput>,
    ) -> S3Result<S3Response<GetBucketLifecycleConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let lifecycle = bucket.lifecycle.read();
        if lifecycle.is_none() {
            return Err(S3ServiceError::NoSuchLifecycleConfiguration.into_s3_error());
        }

        // Return empty rules since we store as opaque JSON.
        let output = GetBucketLifecycleConfigurationOutput {
            rules: Some(Vec::new()),
            transition_default_minimum_object_size: None,
        };
        Ok(S3Response::new(output))
    }

    /// Set lifecycle configuration for a bucket.
    pub(crate) async fn handle_put_bucket_lifecycle_configuration(
        &self,
        req: S3Request<PutBucketLifecycleConfigurationInput>,
    ) -> S3Result<S3Response<PutBucketLifecycleConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        // Store as opaque JSON since we don't enforce lifecycle policies.
        *bucket.lifecycle.write() = Some(serde_json::json!({"status": "configured"}));

        debug!(bucket = %bucket_name, "put_bucket_lifecycle_configuration completed");
        Ok(S3Response::new(PutBucketLifecycleConfigurationOutput {
            transition_default_minimum_object_size: None,
        }))
    }

    /// Delete lifecycle configuration for a bucket.
    pub(crate) async fn handle_delete_bucket_lifecycle(
        &self,
        req: S3Request<DeleteBucketLifecycleInput>,
    ) -> S3Result<S3Response<DeleteBucketLifecycleOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.lifecycle.write() = None;

        debug!(bucket = %bucket_name, "delete_bucket_lifecycle completed");
        Ok(S3Response::new(DeleteBucketLifecycleOutput {}))
    }

    // -----------------------------------------------------------------------
    // Policy
    // -----------------------------------------------------------------------

    /// Get the bucket policy.
    pub(crate) async fn handle_get_bucket_policy(
        &self,
        req: S3Request<GetBucketPolicyInput>,
    ) -> S3Result<S3Response<GetBucketPolicyOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let policy = bucket.policy.read();
        let policy_str = policy
            .as_ref()
            .ok_or(S3ServiceError::NoSuchBucketPolicy)
            .map_err(S3ServiceError::into_s3_error)?
            .clone();

        let output = GetBucketPolicyOutput {
            policy: Some(policy_str),
        };
        Ok(S3Response::new(output))
    }

    /// Set the bucket policy.
    pub(crate) async fn handle_put_bucket_policy(
        &self,
        req: S3Request<PutBucketPolicyInput>,
    ) -> S3Result<S3Response<PutBucketPolicyOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let policy = req.input.policy;

        *bucket.policy.write() = Some(policy);

        debug!(bucket = %bucket_name, "put_bucket_policy completed");
        Ok(S3Response::new(PutBucketPolicyOutput {}))
    }

    /// Delete the bucket policy.
    pub(crate) async fn handle_delete_bucket_policy(
        &self,
        req: S3Request<DeleteBucketPolicyInput>,
    ) -> S3Result<S3Response<DeleteBucketPolicyOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.policy.write() = None;

        debug!(bucket = %bucket_name, "delete_bucket_policy completed");
        Ok(S3Response::new(DeleteBucketPolicyOutput {}))
    }

    // -----------------------------------------------------------------------
    // Tagging
    // -----------------------------------------------------------------------

    /// Get the tag set for a bucket.
    pub(crate) async fn handle_get_bucket_tagging(
        &self,
        req: S3Request<GetBucketTaggingInput>,
    ) -> S3Result<S3Response<GetBucketTaggingOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let tags = bucket.tags.read();
        if tags.is_empty() {
            return Err(S3ServiceError::NoSuchTagSet.into_s3_error());
        }

        let tag_set: Vec<Tag> = tags
            .iter()
            .map(|(k, v)| Tag {
                key: Some(k.clone()),
                value: Some(v.clone()),
            })
            .collect();

        let output = GetBucketTaggingOutput { tag_set };
        Ok(S3Response::new(output))
    }

    /// Set the tag set for a bucket.
    pub(crate) async fn handle_put_bucket_tagging(
        &self,
        req: S3Request<PutBucketTaggingInput>,
    ) -> S3Result<S3Response<PutBucketTaggingOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let tagging = req.input.tagging;

        let tags: Vec<(String, String)> = tagging
            .tag_set
            .into_iter()
            .map(|t| (t.key.unwrap_or_default(), t.value.unwrap_or_default()))
            .collect();

        crate::validation::validate_tags(&tags).map_err(S3ServiceError::into_s3_error)?;

        *bucket.tags.write() = tags;

        debug!(bucket = %bucket_name, "put_bucket_tagging completed");
        Ok(S3Response::new(PutBucketTaggingOutput {}))
    }

    /// Delete the tag set for a bucket.
    pub(crate) async fn handle_delete_bucket_tagging(
        &self,
        req: S3Request<DeleteBucketTaggingInput>,
    ) -> S3Result<S3Response<DeleteBucketTaggingOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.tags.write() = Vec::new();

        debug!(bucket = %bucket_name, "delete_bucket_tagging completed");
        Ok(S3Response::new(DeleteBucketTaggingOutput {}))
    }

    // -----------------------------------------------------------------------
    // Notification
    // -----------------------------------------------------------------------

    /// Get notification configuration for a bucket.
    pub(crate) async fn handle_get_bucket_notification_configuration(
        &self,
        req: S3Request<GetBucketNotificationConfigurationInput>,
    ) -> S3Result<S3Response<GetBucketNotificationConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        // Verify bucket exists.
        let _bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        // Return empty configuration (we don't actively deliver notifications).
        let output = GetBucketNotificationConfigurationOutput {
            event_bridge_configuration: None,
            lambda_function_configurations: None,
            queue_configurations: None,
            topic_configurations: None,
        };
        Ok(S3Response::new(output))
    }

    /// Set notification configuration for a bucket.
    pub(crate) async fn handle_put_bucket_notification_configuration(
        &self,
        req: S3Request<PutBucketNotificationConfigurationInput>,
    ) -> S3Result<S3Response<PutBucketNotificationConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        // Store as opaque JSON.
        *bucket.notification_configuration.write() =
            Some(serde_json::json!({"status": "configured"}));

        debug!(bucket = %bucket_name, "put_bucket_notification_configuration completed");
        Ok(S3Response::new(PutBucketNotificationConfigurationOutput {}))
    }

    // -----------------------------------------------------------------------
    // Logging
    // -----------------------------------------------------------------------

    /// Get logging configuration for a bucket.
    pub(crate) async fn handle_get_bucket_logging(
        &self,
        req: S3Request<GetBucketLoggingInput>,
    ) -> S3Result<S3Response<GetBucketLoggingOutput>> {
        let bucket_name = req.input.bucket;

        // Verify bucket exists.
        let _bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let output = GetBucketLoggingOutput {
            logging_enabled: None,
        };
        Ok(S3Response::new(output))
    }

    /// Set logging configuration for a bucket.
    pub(crate) async fn handle_put_bucket_logging(
        &self,
        req: S3Request<PutBucketLoggingInput>,
    ) -> S3Result<S3Response<PutBucketLoggingOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.logging.write() = Some(serde_json::json!({"status": "configured"}));

        debug!(bucket = %bucket_name, "put_bucket_logging completed");
        Ok(S3Response::new(PutBucketLoggingOutput {}))
    }

    // -----------------------------------------------------------------------
    // Public Access Block
    // -----------------------------------------------------------------------

    /// Get public access block configuration for a bucket.
    pub(crate) async fn handle_get_public_access_block(
        &self,
        req: S3Request<GetPublicAccessBlockInput>,
    ) -> S3Result<S3Response<GetPublicAccessBlockOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let pab = bucket.public_access_block.read();
        let config = pab
            .as_ref()
            .ok_or(S3ServiceError::NoSuchPublicAccessBlockConfiguration)
            .map_err(S3ServiceError::into_s3_error)?;

        let output = GetPublicAccessBlockOutput {
            public_access_block_configuration: Some(PublicAccessBlockConfiguration {
                block_public_acls: Some(config.block_public_acls),
                block_public_policy: Some(config.block_public_policy),
                ignore_public_acls: Some(config.ignore_public_acls),
                restrict_public_buckets: Some(config.restrict_public_buckets),
            }),
        };
        Ok(S3Response::new(output))
    }

    /// Set public access block configuration for a bucket.
    pub(crate) async fn handle_put_public_access_block(
        &self,
        req: S3Request<PutPublicAccessBlockInput>,
    ) -> S3Result<S3Response<PutPublicAccessBlockOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let config = req.input.public_access_block_configuration;

        let internal_config = PublicAccessBlockConfig {
            block_public_acls: config.block_public_acls.unwrap_or(false),
            ignore_public_acls: config.ignore_public_acls.unwrap_or(false),
            block_public_policy: config.block_public_policy.unwrap_or(false),
            restrict_public_buckets: config.restrict_public_buckets.unwrap_or(false),
        };

        *bucket.public_access_block.write() = Some(internal_config);

        debug!(bucket = %bucket_name, "put_public_access_block completed");
        Ok(S3Response::new(PutPublicAccessBlockOutput {}))
    }

    /// Delete public access block configuration for a bucket.
    pub(crate) async fn handle_delete_public_access_block(
        &self,
        req: S3Request<DeletePublicAccessBlockInput>,
    ) -> S3Result<S3Response<DeletePublicAccessBlockOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.public_access_block.write() = None;

        debug!(bucket = %bucket_name, "delete_public_access_block completed");
        Ok(S3Response::new(DeletePublicAccessBlockOutput {}))
    }

    // -----------------------------------------------------------------------
    // Ownership Controls
    // -----------------------------------------------------------------------

    /// Get ownership controls configuration for a bucket.
    pub(crate) async fn handle_get_bucket_ownership_controls(
        &self,
        req: S3Request<GetBucketOwnershipControlsInput>,
    ) -> S3Result<S3Response<GetBucketOwnershipControlsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let controls = bucket.ownership_controls.read();
        let config = controls
            .as_ref()
            .ok_or(S3ServiceError::OwnershipControlsNotFoundError)
            .map_err(S3ServiceError::into_s3_error)?;

        let rule = OwnershipControlsRule {
            object_ownership: ObjectOwnership::from(config.object_ownership.clone()),
        };

        let output = GetBucketOwnershipControlsOutput {
            ownership_controls: Some(OwnershipControls { rules: vec![rule] }),
        };
        Ok(S3Response::new(output))
    }

    /// Set ownership controls configuration for a bucket.
    pub(crate) async fn handle_put_bucket_ownership_controls(
        &self,
        req: S3Request<PutBucketOwnershipControlsInput>,
    ) -> S3Result<S3Response<PutBucketOwnershipControlsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let controls = req.input.ownership_controls;

        let rule = controls
            .rules
            .first()
            .ok_or_else(|| s3s::s3_error!(InvalidArgument, "At least one rule is required"))?;

        let ownership = rule.object_ownership.as_str().to_owned();

        *bucket.ownership_controls.write() = Some(OwnershipControlsConfig {
            object_ownership: ownership,
        });

        debug!(bucket = %bucket_name, "put_bucket_ownership_controls completed");
        Ok(S3Response::new(PutBucketOwnershipControlsOutput {}))
    }

    /// Delete ownership controls configuration for a bucket.
    pub(crate) async fn handle_delete_bucket_ownership_controls(
        &self,
        req: S3Request<DeleteBucketOwnershipControlsInput>,
    ) -> S3Result<S3Response<DeleteBucketOwnershipControlsOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.ownership_controls.write() = None;

        debug!(bucket = %bucket_name, "delete_bucket_ownership_controls completed");
        Ok(S3Response::new(DeleteBucketOwnershipControlsOutput {}))
    }

    // -----------------------------------------------------------------------
    // Object Lock Configuration
    // -----------------------------------------------------------------------

    /// Get object lock configuration for a bucket.
    pub(crate) async fn handle_get_object_lock_configuration(
        &self,
        req: S3Request<GetObjectLockConfigurationInput>,
    ) -> S3Result<S3Response<GetObjectLockConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        if !*bucket.object_lock_enabled.read() {
            return Err(S3ServiceError::ObjectLockConfigurationNotFoundError.into_s3_error());
        }

        let lock_config = bucket.object_lock_configuration.read();
        let rule = lock_config.as_ref().and_then(|c| {
            c.rule.as_ref().map(|r| s3s::dto::ObjectLockRule {
                default_retention: r.default_retention.as_ref().map(|dr| {
                    s3s::dto::DefaultRetention {
                        days: dr.days,
                        mode: Some(ObjectLockRetentionMode::from(dr.mode.clone())),
                        years: dr.years,
                    }
                }),
            })
        });

        let output = GetObjectLockConfigurationOutput {
            object_lock_configuration: Some(s3s::dto::ObjectLockConfiguration {
                object_lock_enabled: Some(ObjectLockEnabled::from_static("Enabled")),
                rule,
            }),
        };
        Ok(S3Response::new(output))
    }

    /// Set object lock configuration for a bucket.
    pub(crate) async fn handle_put_object_lock_configuration(
        &self,
        req: S3Request<PutObjectLockConfigurationInput>,
    ) -> S3Result<S3Response<PutObjectLockConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        if let Some(config) = req.input.object_lock_configuration {
            *bucket.object_lock_enabled.write() = true;
            bucket.enable_versioning();

            let internal_config = ObjectLockConfiguration {
                object_lock_enabled: config
                    .object_lock_enabled
                    .as_ref()
                    .map_or_else(|| "Enabled".to_owned(), |e| e.as_str().to_owned()),
                rule: config.rule.map(|r| ObjectLockRule {
                    default_retention: r.default_retention.map(|dr| {
                        crate::state::bucket::DefaultRetention {
                            mode: dr
                                .mode
                                .as_ref()
                                .map(|m| m.as_str().to_owned())
                                .unwrap_or_default(),
                            days: dr.days,
                            years: dr.years,
                        }
                    }),
                }),
            };
            *bucket.object_lock_configuration.write() = Some(internal_config);
        }

        debug!(bucket = %bucket_name, "put_object_lock_configuration completed");
        Ok(S3Response::new(PutObjectLockConfigurationOutput {
            request_charged: None,
        }))
    }

    // -----------------------------------------------------------------------
    // Accelerate
    // -----------------------------------------------------------------------

    /// Get the transfer acceleration configuration for a bucket.
    pub(crate) async fn handle_get_bucket_accelerate_configuration(
        &self,
        req: S3Request<GetBucketAccelerateConfigurationInput>,
    ) -> S3Result<S3Response<GetBucketAccelerateConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let status = bucket
            .accelerate
            .read()
            .as_ref()
            .map(|s| BucketAccelerateStatus::from(s.clone()));

        let output = GetBucketAccelerateConfigurationOutput {
            request_charged: None,
            status,
        };
        Ok(S3Response::new(output))
    }

    /// Set the transfer acceleration configuration for a bucket.
    pub(crate) async fn handle_put_bucket_accelerate_configuration(
        &self,
        req: S3Request<PutBucketAccelerateConfigurationInput>,
    ) -> S3Result<S3Response<PutBucketAccelerateConfigurationOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let config = req.input.accelerate_configuration;
        let status = config.status.map(|s| s.as_str().to_owned());
        *bucket.accelerate.write() = status;

        debug!(bucket = %bucket_name, "put_bucket_accelerate_configuration completed");
        Ok(S3Response::new(PutBucketAccelerateConfigurationOutput {}))
    }

    // -----------------------------------------------------------------------
    // Request Payment
    // -----------------------------------------------------------------------

    /// Get the request payment configuration for a bucket.
    pub(crate) async fn handle_get_bucket_request_payment(
        &self,
        req: S3Request<GetBucketRequestPaymentInput>,
    ) -> S3Result<S3Response<GetBucketRequestPaymentOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let payer = bucket.request_payment.read().clone();

        let output = GetBucketRequestPaymentOutput {
            payer: Some(Payer::from(payer)),
        };
        Ok(S3Response::new(output))
    }

    /// Set the request payment configuration for a bucket.
    pub(crate) async fn handle_put_bucket_request_payment(
        &self,
        req: S3Request<PutBucketRequestPaymentInput>,
    ) -> S3Result<S3Response<PutBucketRequestPaymentOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let config = req.input.request_payment_configuration;
        config
            .payer
            .as_str()
            .clone_into(&mut bucket.request_payment.write());

        debug!(bucket = %bucket_name, "put_bucket_request_payment completed");
        Ok(S3Response::new(PutBucketRequestPaymentOutput {}))
    }

    // -----------------------------------------------------------------------
    // Website
    // -----------------------------------------------------------------------

    /// Get the website configuration for a bucket.
    pub(crate) async fn handle_get_bucket_website(
        &self,
        req: S3Request<GetBucketWebsiteInput>,
    ) -> S3Result<S3Response<GetBucketWebsiteOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let website = bucket.website.read();
        if website.is_none() {
            return Err(S3ServiceError::NoSuchWebsiteConfiguration.into_s3_error());
        }

        let output = GetBucketWebsiteOutput {
            error_document: None,
            index_document: None,
            redirect_all_requests_to: None,
            routing_rules: None,
        };
        Ok(S3Response::new(output))
    }

    /// Set the website configuration for a bucket.
    pub(crate) async fn handle_put_bucket_website(
        &self,
        req: S3Request<PutBucketWebsiteInput>,
    ) -> S3Result<S3Response<PutBucketWebsiteOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.website.write() = Some(serde_json::json!({"status": "configured"}));

        debug!(bucket = %bucket_name, "put_bucket_website completed");
        Ok(S3Response::new(PutBucketWebsiteOutput {}))
    }

    /// Delete the website configuration for a bucket.
    pub(crate) async fn handle_delete_bucket_website(
        &self,
        req: S3Request<DeleteBucketWebsiteInput>,
    ) -> S3Result<S3Response<DeleteBucketWebsiteOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        *bucket.website.write() = None;

        debug!(bucket = %bucket_name, "delete_bucket_website completed");
        Ok(S3Response::new(DeleteBucketWebsiteOutput {}))
    }

    // -----------------------------------------------------------------------
    // Bucket ACL
    // -----------------------------------------------------------------------

    /// Get the ACL for a bucket.
    pub(crate) async fn handle_get_bucket_acl(
        &self,
        req: S3Request<GetBucketAclInput>,
    ) -> S3Result<S3Response<GetBucketAclOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let owner = super::bucket::to_s3_owner(&bucket.owner);
        let acl = *bucket.acl.read();
        let grants = canned_acl_to_grants(&bucket.owner, acl);

        let output = GetBucketAclOutput {
            grants: Some(grants),
            owner: Some(owner),
        };
        Ok(S3Response::new(output))
    }

    /// Set the ACL for a bucket.
    pub(crate) async fn handle_put_bucket_acl(
        &self,
        req: S3Request<PutBucketAclInput>,
    ) -> S3Result<S3Response<PutBucketAclOutput>> {
        let bucket_name = req.input.bucket;

        let bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        if let Some(acl_str) = req.input.acl {
            let acl: CannedAcl = acl_str
                .as_str()
                .parse()
                .map_err(|_| s3s::s3_error!(InvalidArgument, "Invalid canned ACL"))?;
            *bucket.acl.write() = acl;
        }

        debug!(bucket = %bucket_name, "put_bucket_acl completed");
        Ok(S3Response::new(PutBucketAclOutput {}))
    }

    // -----------------------------------------------------------------------
    // Policy Status
    // -----------------------------------------------------------------------

    /// Get the policy status for a bucket.
    pub(crate) async fn handle_get_bucket_policy_status(
        &self,
        req: S3Request<GetBucketPolicyStatusInput>,
    ) -> S3Result<S3Response<GetBucketPolicyStatusOutput>> {
        let bucket_name = req.input.bucket;

        // Verify bucket exists.
        let _bucket = self
            .state
            .get_bucket(&bucket_name)
            .map_err(S3ServiceError::into_s3_error)?;

        let output = GetBucketPolicyStatusOutput {
            policy_status: Some(PolicyStatus {
                is_public: Some(false),
            }),
        };
        Ok(S3Response::new(output))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a [`CorsRuleConfig`] to an s3s [`CORSRule`] DTO.
fn cors_config_to_dto(config: &CorsRuleConfig) -> CORSRule {
    CORSRule {
        allowed_headers: Some(config.allowed_headers.clone()),
        allowed_methods: config.allowed_methods.clone(),
        allowed_origins: config.allowed_origins.clone(),
        expose_headers: Some(config.expose_headers.clone()),
        id: config.id.clone(),
        max_age_seconds: config.max_age_seconds,
    }
}

/// Convert an s3s [`CORSRule`] DTO to a [`CorsRuleConfig`].
fn dto_to_cors_config(rule: &CORSRule) -> CorsRuleConfig {
    CorsRuleConfig {
        id: rule.id.clone(),
        allowed_origins: rule.allowed_origins.clone(),
        allowed_methods: rule.allowed_methods.clone(),
        allowed_headers: rule.allowed_headers.clone().unwrap_or_default(),
        expose_headers: rule.expose_headers.clone().unwrap_or_default(),
        max_age_seconds: rule.max_age_seconds,
    }
}

/// Convert a canned ACL to a list of s3s [`Grant`] DTOs.
fn canned_acl_to_grants(owner: &InternalOwner, acl: CannedAcl) -> Vec<Grant> {
    let owner_grant = Grant {
        grantee: Some(Grantee {
            display_name: Some(owner.display_name.clone()),
            email_address: None,
            id: Some(owner.id.clone()),
            type_: s3s::dto::Type::from_static("CanonicalUser"),
            uri: None,
        }),
        permission: Some(Permission::from_static("FULL_CONTROL")),
    };

    match acl {
        CannedAcl::PublicRead => {
            let public_read = Grant {
                grantee: Some(Grantee {
                    display_name: None,
                    email_address: None,
                    id: None,
                    type_: s3s::dto::Type::from_static("Group"),
                    uri: Some("http://acs.amazonaws.com/groups/global/AllUsers".to_owned()),
                }),
                permission: Some(Permission::from_static("READ")),
            };
            vec![owner_grant, public_read]
        }
        _ => vec![owner_grant],
    }
}
