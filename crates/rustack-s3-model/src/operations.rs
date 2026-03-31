//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

/// All supported S3 operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum S3Operation {
    /// The CreateBucket operation.
    CreateBucket,
    /// The DeleteBucket operation.
    DeleteBucket,
    /// The HeadBucket operation.
    HeadBucket,
    /// The ListBuckets operation.
    ListBuckets,
    /// The GetBucketLocation operation.
    GetBucketLocation,
    /// The GetBucketVersioning operation.
    GetBucketVersioning,
    /// The PutBucketVersioning operation.
    PutBucketVersioning,
    /// The GetBucketEncryption operation.
    GetBucketEncryption,
    /// The PutBucketEncryption operation.
    PutBucketEncryption,
    /// The DeleteBucketEncryption operation.
    DeleteBucketEncryption,
    /// The GetBucketCors operation.
    GetBucketCors,
    /// The PutBucketCors operation.
    PutBucketCors,
    /// The DeleteBucketCors operation.
    DeleteBucketCors,
    /// The GetBucketLifecycleConfiguration operation.
    GetBucketLifecycleConfiguration,
    /// The PutBucketLifecycleConfiguration operation.
    PutBucketLifecycleConfiguration,
    /// The DeleteBucketLifecycle operation.
    DeleteBucketLifecycle,
    /// The GetBucketPolicy operation.
    GetBucketPolicy,
    /// The PutBucketPolicy operation.
    PutBucketPolicy,
    /// The DeleteBucketPolicy operation.
    DeleteBucketPolicy,
    /// The GetBucketTagging operation.
    GetBucketTagging,
    /// The PutBucketTagging operation.
    PutBucketTagging,
    /// The DeleteBucketTagging operation.
    DeleteBucketTagging,
    /// The GetBucketNotificationConfiguration operation.
    GetBucketNotificationConfiguration,
    /// The PutBucketNotificationConfiguration operation.
    PutBucketNotificationConfiguration,
    /// The GetBucketLogging operation.
    GetBucketLogging,
    /// The PutBucketLogging operation.
    PutBucketLogging,
    /// The GetPublicAccessBlock operation.
    GetPublicAccessBlock,
    /// The PutPublicAccessBlock operation.
    PutPublicAccessBlock,
    /// The DeletePublicAccessBlock operation.
    DeletePublicAccessBlock,
    /// The GetBucketOwnershipControls operation.
    GetBucketOwnershipControls,
    /// The PutBucketOwnershipControls operation.
    PutBucketOwnershipControls,
    /// The DeleteBucketOwnershipControls operation.
    DeleteBucketOwnershipControls,
    /// The GetObjectLockConfiguration operation.
    GetObjectLockConfiguration,
    /// The PutObjectLockConfiguration operation.
    PutObjectLockConfiguration,
    /// The GetBucketAccelerateConfiguration operation.
    GetBucketAccelerateConfiguration,
    /// The PutBucketAccelerateConfiguration operation.
    PutBucketAccelerateConfiguration,
    /// The GetBucketRequestPayment operation.
    GetBucketRequestPayment,
    /// The PutBucketRequestPayment operation.
    PutBucketRequestPayment,
    /// The GetBucketWebsite operation.
    GetBucketWebsite,
    /// The PutBucketWebsite operation.
    PutBucketWebsite,
    /// The DeleteBucketWebsite operation.
    DeleteBucketWebsite,
    /// The GetBucketAcl operation.
    GetBucketAcl,
    /// The PutBucketAcl operation.
    PutBucketAcl,
    /// The GetBucketPolicyStatus operation.
    GetBucketPolicyStatus,
    /// The PutObject operation.
    PutObject,
    /// The GetObject operation.
    GetObject,
    /// The HeadObject operation.
    HeadObject,
    /// The DeleteObject operation.
    DeleteObject,
    /// The DeleteObjects operation.
    DeleteObjects,
    /// The CopyObject operation.
    CopyObject,
    /// The GetObjectTagging operation.
    GetObjectTagging,
    /// The PutObjectTagging operation.
    PutObjectTagging,
    /// The DeleteObjectTagging operation.
    DeleteObjectTagging,
    /// The GetObjectAcl operation.
    GetObjectAcl,
    /// The PutObjectAcl operation.
    PutObjectAcl,
    /// The GetObjectRetention operation.
    GetObjectRetention,
    /// The PutObjectRetention operation.
    PutObjectRetention,
    /// The GetObjectLegalHold operation.
    GetObjectLegalHold,
    /// The PutObjectLegalHold operation.
    PutObjectLegalHold,
    /// The GetObjectAttributes operation.
    GetObjectAttributes,
    /// The CreateMultipartUpload operation.
    CreateMultipartUpload,
    /// The UploadPart operation.
    UploadPart,
    /// The UploadPartCopy operation.
    UploadPartCopy,
    /// The CompleteMultipartUpload operation.
    CompleteMultipartUpload,
    /// The AbortMultipartUpload operation.
    AbortMultipartUpload,
    /// The ListParts operation.
    ListParts,
    /// The ListMultipartUploads operation.
    ListMultipartUploads,
    /// The ListObjects operation.
    ListObjects,
    /// The ListObjectsV2 operation.
    ListObjectsV2,
    /// The ListObjectVersions operation.
    ListObjectVersions,
    /// The PostObject (browser-based upload) operation.
    PostObject,
}

impl S3Operation {
    /// Returns the AWS operation name string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreateBucket => "CreateBucket",
            Self::DeleteBucket => "DeleteBucket",
            Self::HeadBucket => "HeadBucket",
            Self::ListBuckets => "ListBuckets",
            Self::GetBucketLocation => "GetBucketLocation",
            Self::GetBucketVersioning => "GetBucketVersioning",
            Self::PutBucketVersioning => "PutBucketVersioning",
            Self::GetBucketEncryption => "GetBucketEncryption",
            Self::PutBucketEncryption => "PutBucketEncryption",
            Self::DeleteBucketEncryption => "DeleteBucketEncryption",
            Self::GetBucketCors => "GetBucketCors",
            Self::PutBucketCors => "PutBucketCors",
            Self::DeleteBucketCors => "DeleteBucketCors",
            Self::GetBucketLifecycleConfiguration => "GetBucketLifecycleConfiguration",
            Self::PutBucketLifecycleConfiguration => "PutBucketLifecycleConfiguration",
            Self::DeleteBucketLifecycle => "DeleteBucketLifecycle",
            Self::GetBucketPolicy => "GetBucketPolicy",
            Self::PutBucketPolicy => "PutBucketPolicy",
            Self::DeleteBucketPolicy => "DeleteBucketPolicy",
            Self::GetBucketTagging => "GetBucketTagging",
            Self::PutBucketTagging => "PutBucketTagging",
            Self::DeleteBucketTagging => "DeleteBucketTagging",
            Self::GetBucketNotificationConfiguration => "GetBucketNotificationConfiguration",
            Self::PutBucketNotificationConfiguration => "PutBucketNotificationConfiguration",
            Self::GetBucketLogging => "GetBucketLogging",
            Self::PutBucketLogging => "PutBucketLogging",
            Self::GetPublicAccessBlock => "GetPublicAccessBlock",
            Self::PutPublicAccessBlock => "PutPublicAccessBlock",
            Self::DeletePublicAccessBlock => "DeletePublicAccessBlock",
            Self::GetBucketOwnershipControls => "GetBucketOwnershipControls",
            Self::PutBucketOwnershipControls => "PutBucketOwnershipControls",
            Self::DeleteBucketOwnershipControls => "DeleteBucketOwnershipControls",
            Self::GetObjectLockConfiguration => "GetObjectLockConfiguration",
            Self::PutObjectLockConfiguration => "PutObjectLockConfiguration",
            Self::GetBucketAccelerateConfiguration => "GetBucketAccelerateConfiguration",
            Self::PutBucketAccelerateConfiguration => "PutBucketAccelerateConfiguration",
            Self::GetBucketRequestPayment => "GetBucketRequestPayment",
            Self::PutBucketRequestPayment => "PutBucketRequestPayment",
            Self::GetBucketWebsite => "GetBucketWebsite",
            Self::PutBucketWebsite => "PutBucketWebsite",
            Self::DeleteBucketWebsite => "DeleteBucketWebsite",
            Self::GetBucketAcl => "GetBucketAcl",
            Self::PutBucketAcl => "PutBucketAcl",
            Self::GetBucketPolicyStatus => "GetBucketPolicyStatus",
            Self::PutObject => "PutObject",
            Self::GetObject => "GetObject",
            Self::HeadObject => "HeadObject",
            Self::DeleteObject => "DeleteObject",
            Self::DeleteObjects => "DeleteObjects",
            Self::CopyObject => "CopyObject",
            Self::GetObjectTagging => "GetObjectTagging",
            Self::PutObjectTagging => "PutObjectTagging",
            Self::DeleteObjectTagging => "DeleteObjectTagging",
            Self::GetObjectAcl => "GetObjectAcl",
            Self::PutObjectAcl => "PutObjectAcl",
            Self::GetObjectRetention => "GetObjectRetention",
            Self::PutObjectRetention => "PutObjectRetention",
            Self::GetObjectLegalHold => "GetObjectLegalHold",
            Self::PutObjectLegalHold => "PutObjectLegalHold",
            Self::GetObjectAttributes => "GetObjectAttributes",
            Self::CreateMultipartUpload => "CreateMultipartUpload",
            Self::UploadPart => "UploadPart",
            Self::UploadPartCopy => "UploadPartCopy",
            Self::CompleteMultipartUpload => "CompleteMultipartUpload",
            Self::AbortMultipartUpload => "AbortMultipartUpload",
            Self::ListParts => "ListParts",
            Self::ListMultipartUploads => "ListMultipartUploads",
            Self::ListObjects => "ListObjects",
            Self::ListObjectsV2 => "ListObjectsV2",
            Self::ListObjectVersions => "ListObjectVersions",
            Self::PostObject => "PostObject",
        }
    }

    /// Parse an operation name string into an S3Operation.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "CreateBucket" => Some(Self::CreateBucket),
            "DeleteBucket" => Some(Self::DeleteBucket),
            "HeadBucket" => Some(Self::HeadBucket),
            "ListBuckets" => Some(Self::ListBuckets),
            "GetBucketLocation" => Some(Self::GetBucketLocation),
            "GetBucketVersioning" => Some(Self::GetBucketVersioning),
            "PutBucketVersioning" => Some(Self::PutBucketVersioning),
            "GetBucketEncryption" => Some(Self::GetBucketEncryption),
            "PutBucketEncryption" => Some(Self::PutBucketEncryption),
            "DeleteBucketEncryption" => Some(Self::DeleteBucketEncryption),
            "GetBucketCors" => Some(Self::GetBucketCors),
            "PutBucketCors" => Some(Self::PutBucketCors),
            "DeleteBucketCors" => Some(Self::DeleteBucketCors),
            "GetBucketLifecycleConfiguration" => Some(Self::GetBucketLifecycleConfiguration),
            "PutBucketLifecycleConfiguration" => Some(Self::PutBucketLifecycleConfiguration),
            "DeleteBucketLifecycle" => Some(Self::DeleteBucketLifecycle),
            "GetBucketPolicy" => Some(Self::GetBucketPolicy),
            "PutBucketPolicy" => Some(Self::PutBucketPolicy),
            "DeleteBucketPolicy" => Some(Self::DeleteBucketPolicy),
            "GetBucketTagging" => Some(Self::GetBucketTagging),
            "PutBucketTagging" => Some(Self::PutBucketTagging),
            "DeleteBucketTagging" => Some(Self::DeleteBucketTagging),
            "GetBucketNotificationConfiguration" => Some(Self::GetBucketNotificationConfiguration),
            "PutBucketNotificationConfiguration" => Some(Self::PutBucketNotificationConfiguration),
            "GetBucketLogging" => Some(Self::GetBucketLogging),
            "PutBucketLogging" => Some(Self::PutBucketLogging),
            "GetPublicAccessBlock" => Some(Self::GetPublicAccessBlock),
            "PutPublicAccessBlock" => Some(Self::PutPublicAccessBlock),
            "DeletePublicAccessBlock" => Some(Self::DeletePublicAccessBlock),
            "GetBucketOwnershipControls" => Some(Self::GetBucketOwnershipControls),
            "PutBucketOwnershipControls" => Some(Self::PutBucketOwnershipControls),
            "DeleteBucketOwnershipControls" => Some(Self::DeleteBucketOwnershipControls),
            "GetObjectLockConfiguration" => Some(Self::GetObjectLockConfiguration),
            "PutObjectLockConfiguration" => Some(Self::PutObjectLockConfiguration),
            "GetBucketAccelerateConfiguration" => Some(Self::GetBucketAccelerateConfiguration),
            "PutBucketAccelerateConfiguration" => Some(Self::PutBucketAccelerateConfiguration),
            "GetBucketRequestPayment" => Some(Self::GetBucketRequestPayment),
            "PutBucketRequestPayment" => Some(Self::PutBucketRequestPayment),
            "GetBucketWebsite" => Some(Self::GetBucketWebsite),
            "PutBucketWebsite" => Some(Self::PutBucketWebsite),
            "DeleteBucketWebsite" => Some(Self::DeleteBucketWebsite),
            "GetBucketAcl" => Some(Self::GetBucketAcl),
            "PutBucketAcl" => Some(Self::PutBucketAcl),
            "GetBucketPolicyStatus" => Some(Self::GetBucketPolicyStatus),
            "PutObject" => Some(Self::PutObject),
            "GetObject" => Some(Self::GetObject),
            "HeadObject" => Some(Self::HeadObject),
            "DeleteObject" => Some(Self::DeleteObject),
            "DeleteObjects" => Some(Self::DeleteObjects),
            "CopyObject" => Some(Self::CopyObject),
            "GetObjectTagging" => Some(Self::GetObjectTagging),
            "PutObjectTagging" => Some(Self::PutObjectTagging),
            "DeleteObjectTagging" => Some(Self::DeleteObjectTagging),
            "GetObjectAcl" => Some(Self::GetObjectAcl),
            "PutObjectAcl" => Some(Self::PutObjectAcl),
            "GetObjectRetention" => Some(Self::GetObjectRetention),
            "PutObjectRetention" => Some(Self::PutObjectRetention),
            "GetObjectLegalHold" => Some(Self::GetObjectLegalHold),
            "PutObjectLegalHold" => Some(Self::PutObjectLegalHold),
            "GetObjectAttributes" => Some(Self::GetObjectAttributes),
            "CreateMultipartUpload" => Some(Self::CreateMultipartUpload),
            "UploadPart" => Some(Self::UploadPart),
            "UploadPartCopy" => Some(Self::UploadPartCopy),
            "CompleteMultipartUpload" => Some(Self::CompleteMultipartUpload),
            "AbortMultipartUpload" => Some(Self::AbortMultipartUpload),
            "ListParts" => Some(Self::ListParts),
            "ListMultipartUploads" => Some(Self::ListMultipartUploads),
            "ListObjects" => Some(Self::ListObjects),
            "ListObjectsV2" => Some(Self::ListObjectsV2),
            "ListObjectVersions" => Some(Self::ListObjectVersions),
            "PostObject" => Some(Self::PostObject),
            _ => None,
        }
    }
}

impl std::fmt::Display for S3Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
