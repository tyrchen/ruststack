//! SES XML response formatting and error serialization.
//!
//! SES v1 responses use `text/xml` content type following the awsQuery protocol.
//! SES v2 responses use `application/json`.
//!
//! All SES v1 responses follow the pattern:
//!
//! ```xml
//! <{Operation}Response xmlns="http://ses.amazonaws.com/doc/2010-12-01/">
//!   <{Operation}Result>
//!     ...fields...
//!   </{Operation}Result>
//!   <ResponseMetadata>
//!     <RequestId>{uuid}</RequestId>
//!   </ResponseMetadata>
//! </{Operation}Response>
//! ```

use rustack_ses_model::error::SesError;

use crate::body::SesResponseBody;

/// Content type for SES v1 XML responses.
pub const XML_CONTENT_TYPE: &str = "text/xml";

/// Content type for SES v2 JSON responses.
pub const JSON_CONTENT_TYPE: &str = "application/json";

/// The SES v1 XML namespace.
const XML_NS: &str = "http://ses.amazonaws.com/doc/2010-12-01/";

/// Build a success XML response with the given body and request ID.
#[must_use]
pub fn xml_response(xml: String, request_id: &str) -> http::Response<SesResponseBody> {
    let body = SesResponseBody::from_xml(xml.into_bytes());
    http::Response::builder()
        .status(http::StatusCode::OK)
        .header("content-type", XML_CONTENT_TYPE)
        .header("x-amzn-requestid", request_id)
        .body(body)
        .expect("valid XML response")
}

/// Build a success JSON response for SES v2.
#[must_use]
pub fn json_response(json: String, status: http::StatusCode) -> http::Response<SesResponseBody> {
    let body = SesResponseBody::from_json(json);
    http::Response::builder()
        .status(status)
        .header("content-type", JSON_CONTENT_TYPE)
        .body(body)
        .expect("valid JSON response")
}

/// Serialize an SES error into an XML error response body string.
#[must_use]
pub fn error_to_xml(error: &SesError, request_id: &str) -> String {
    format!(
        "<ErrorResponse \
         xmlns=\"{XML_NS}\"><Error><Type>{}</Type><Code>{}</Code><Message>{}</Message></\
         Error><RequestId>{}</RequestId></ErrorResponse>",
        error.code.fault(),
        error.code.code(),
        xml_escape(&error.message),
        xml_escape(request_id),
    )
}

/// Convert an `SesError` into a complete HTTP error response (XML for v1).
#[must_use]
pub fn error_to_response(error: &SesError, request_id: &str) -> http::Response<SesResponseBody> {
    let xml = error_to_xml(error, request_id);
    let body = SesResponseBody::from_xml(xml.into_bytes());
    http::Response::builder()
        .status(error.status_code)
        .header("content-type", XML_CONTENT_TYPE)
        .header("x-amzn-requestid", request_id)
        .body(body)
        .expect("valid error response")
}

/// Convert an `SesError` into a JSON error response (for v2).
#[must_use]
pub fn error_to_json_response(error: &SesError) -> http::Response<SesResponseBody> {
    let json = serde_json::json!({
        "__type": error.code.code(),
        "message": error.message,
    });
    let body = SesResponseBody::from_json(json.to_string());
    http::Response::builder()
        .status(error.status_code)
        .header("content-type", JSON_CONTENT_TYPE)
        .body(body)
        .expect("valid JSON error response")
}

/// XML-escape a string value.
///
/// Replaces the five XML special characters with their entity references.
#[must_use]
pub fn xml_escape(s: &str) -> String {
    if !s.contains(['&', '<', '>', '"', '\'']) {
        return s.to_owned();
    }

    let mut result = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(ch),
        }
    }
    result
}

/// Simple XML writer for building SES response XML.
#[derive(Debug)]
pub struct XmlWriter {
    buf: String,
}

impl Default for XmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlWriter {
    /// Create a new `XmlWriter`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(512),
        }
    }

    /// Start the response envelope: `<{op}Response xmlns="...">`.
    pub fn start_response(&mut self, operation: &str) {
        self.buf.push('<');
        self.buf.push_str(operation);
        self.buf.push_str("Response xmlns=\"");
        self.buf.push_str(XML_NS);
        self.buf.push_str("\">");
    }

    /// Start the result element: `<{op}Result>`.
    pub fn start_result(&mut self, operation: &str) {
        self.buf.push('<');
        self.buf.push_str(operation);
        self.buf.push_str("Result>");
    }

    /// End an element: `</{name}>`.
    pub fn end_element(&mut self, name: &str) {
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push('>');
    }

    /// Write a simple text element: `<Name>Value</Name>`.
    pub fn write_element(&mut self, name: &str, value: &str) {
        self.buf.push('<');
        self.buf.push_str(name);
        self.buf.push('>');
        self.buf.push_str(&xml_escape(value));
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push('>');
    }

    /// Write an optional element (skip if `None`).
    pub fn write_optional_element(&mut self, name: &str, value: Option<&str>) {
        if let Some(v) = value {
            self.write_element(name, v);
        }
    }

    /// Write a boolean element: `<name>true</name>` or `<name>false</name>`.
    pub fn write_bool_element(&mut self, name: &str, value: bool) {
        self.write_element(name, if value { "true" } else { "false" });
    }

    /// Write a float element.
    pub fn write_f64_element(&mut self, name: &str, value: f64) {
        self.write_element(name, &value.to_string());
    }

    /// Write an i64 element.
    pub fn write_i64_element(&mut self, name: &str, value: i64) {
        self.write_element(name, &value.to_string());
    }

    /// Write the `<ResponseMetadata>` block with a `<RequestId>`.
    pub fn write_response_metadata(&mut self, request_id: &str) {
        self.buf.push_str("<ResponseMetadata>");
        self.write_element("RequestId", request_id);
        self.buf.push_str("</ResponseMetadata>");
    }

    /// Write raw XML content without escaping.
    pub fn raw(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    /// Consume the writer and return the final XML string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_escape_xml_special_chars() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_should_format_error_xml() {
        let err = SesError::message_rejected("Email address is not verified.");
        let xml = error_to_xml(&err, "req-123");
        assert!(xml.contains("<Code>MessageRejected</Code>"));
        assert!(xml.contains("Email address is not verified."));
        assert!(xml.contains("<RequestId>req-123</RequestId>"));
    }

    #[test]
    fn test_should_build_error_response_with_correct_status() {
        let err = SesError::template_does_not_exist("my-template");
        let resp = error_to_response(&err, "test-req");
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_should_build_xml_with_writer() {
        let mut w = XmlWriter::new();
        w.start_response("SendEmail");
        w.start_result("SendEmail");
        w.write_element("MessageId", "test-id-123");
        w.end_element("SendEmailResult");
        w.write_response_metadata("req-789");
        w.end_element("SendEmailResponse");
        let xml = w.into_string();
        assert!(xml.contains("<MessageId>test-id-123</MessageId>"));
        assert!(xml.contains("<RequestId>req-789</RequestId>"));
        assert!(xml.contains("xmlns=\"http://ses.amazonaws.com/doc/2010-12-01/\""));
    }

    #[test]
    fn test_should_build_json_error_response() {
        let err = SesError::internal_error("Something went wrong");
        let resp = error_to_json_response(&err);
        assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            JSON_CONTENT_TYPE,
        );
    }

    #[test]
    fn test_should_build_success_json_response() {
        let resp = json_response("{\"ok\":true}".to_owned(), http::StatusCode::OK);
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            JSON_CONTENT_TYPE,
        );
    }
}
