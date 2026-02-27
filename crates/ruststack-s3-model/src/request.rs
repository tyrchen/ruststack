//! Auto-generated from AWS S3 Smithy model. DO NOT EDIT.

/// A wrapper around `bytes::Bytes` for streaming blob data.
///
/// In the future, this may be replaced with an actual streaming type.
#[derive(Debug, Clone, Default)]
pub struct StreamingBlob {
    /// The underlying bytes data.
    pub data: bytes::Bytes,
}

impl StreamingBlob {
    /// Create a new `StreamingBlob` from bytes.
    #[must_use]
    pub fn new(data: impl Into<bytes::Bytes>) -> Self {
        Self { data: data.into() }
    }

    /// Returns true if the blob is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the length of the blob.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl From<bytes::Bytes> for StreamingBlob {
    fn from(data: bytes::Bytes) -> Self {
        Self { data }
    }
}

impl From<Vec<u8>> for StreamingBlob {
    fn from(data: Vec<u8>) -> Self {
        Self { data: data.into() }
    }
}

impl From<&[u8]> for StreamingBlob {
    fn from(data: &[u8]) -> Self {
        Self {
            data: bytes::Bytes::copy_from_slice(data),
        }
    }
}

/// AWS credentials for request authentication.
#[derive(Clone, Default)]
pub struct Credentials {
    /// The AWS access key ID.
    pub access_key_id: String,
    /// The AWS secret access key.
    pub secret_access_key: String,
    /// Optional session token for temporary credentials.
    pub session_token: Option<String>,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &"[REDACTED]")
            .field(
                "session_token",
                &self.session_token.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

/// An S3 request wrapping an input type with credentials and headers.
#[derive(Debug, Clone)]
pub struct S3Request<T> {
    /// The input payload.
    pub input: T,
    /// Optional credentials for the request.
    pub credentials: Option<Credentials>,
    /// Additional HTTP headers.
    pub headers: http::HeaderMap,
}

impl<T: Default> Default for S3Request<T> {
    fn default() -> Self {
        Self {
            input: T::default(),
            credentials: None,
            headers: http::HeaderMap::new(),
        }
    }
}

impl<T> S3Request<T> {
    /// Create a new S3Request with the given input.
    #[must_use]
    pub fn new(input: T) -> Self {
        Self {
            input,
            credentials: None,
            headers: http::HeaderMap::new(),
        }
    }

    /// Set the credentials for this request.
    #[must_use]
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Map the input type to a different type.
    pub fn map_input<U>(self, f: impl FnOnce(T) -> U) -> S3Request<U> {
        S3Request {
            input: f(self.input),
            credentials: self.credentials,
            headers: self.headers,
        }
    }
}
