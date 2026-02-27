//! S3 error XML formatting and error types.
//!
//! This module provides the `XmlError` type for XML serialization/deserialization
//! errors and the `error_to_xml` function for formatting S3 error responses.

use std::io;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesText, Event};

/// Errors that can occur during S3 XML serialization or deserialization.
#[derive(Debug, thiserror::Error)]
pub enum XmlError {
    /// An I/O error during XML writing.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// An error from the underlying quick-xml library.
    #[error("XML processing error: {0}")]
    QuickXml(#[from] quick_xml::Error),

    /// An error from quick-xml attribute handling.
    #[error("XML attribute error: {0}")]
    Attribute(#[from] quick_xml::events::attributes::AttrError),

    /// A required XML element was missing.
    #[error("missing required XML element: {0}")]
    MissingElement(String),

    /// An unexpected XML element was encountered.
    #[error("unexpected XML element: {0}")]
    UnexpectedElement(String),

    /// An error parsing a value from XML text content.
    #[error("failed to parse value: {0}")]
    ParseError(String),
}

/// Format an S3 error as XML.
///
/// S3 uses `noErrorWrapping: true`, so errors are formatted as a flat `<Error>` element
/// without an outer `<ErrorResponse>` wrapper.
///
/// # Example output
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <Error>
///   <Code>NoSuchBucket</Code>
///   <Message>The specified bucket does not exist</Message>
///   <Resource>/mybucket</Resource>
///   <RequestId>tx00000...</RequestId>
/// </Error>
/// ```
pub fn error_to_xml(
    code: &str,
    message: &str,
    resource: Option<&str>,
    request_id: &str,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(256);
    // Writing to Vec<u8> is infallible; if this fails it means a logic error.
    if let Err(e) = write_error_xml(&mut buf, code, message, resource, request_id) {
        tracing::error!(error = %e, "failed to serialize S3 error XML");
        buf.clear();
    }
    buf
}

fn write_error_xml(
    buf: &mut Vec<u8>,
    code: &str,
    message: &str,
    resource: Option<&str>,
    request_id: &str,
) -> io::Result<()> {
    let mut writer = Writer::new(buf);

    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    writer.create_element("Error").write_inner_content(|w| {
        w.create_element("Code")
            .write_text_content(BytesText::new(code))?;
        w.create_element("Message")
            .write_text_content(BytesText::new(message))?;
        if let Some(res) = resource {
            w.create_element("Resource")
                .write_text_content(BytesText::new(res))?;
        }
        w.create_element("RequestId")
            .write_text_content(BytesText::new(request_id))?;
        Ok(())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_format_error_with_resource() {
        let xml = error_to_xml(
            "NoSuchBucket",
            "The specified bucket does not exist",
            Some("/mybucket"),
            "tx000001",
        );
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml_str.contains("<Code>NoSuchBucket</Code>"));
        assert!(xml_str.contains("<Message>The specified bucket does not exist</Message>"));
        assert!(xml_str.contains("<Resource>/mybucket</Resource>"));
        assert!(xml_str.contains("<RequestId>tx000001</RequestId>"));
    }

    #[test]
    fn test_should_format_error_without_resource() {
        let xml = error_to_xml("InternalError", "Internal server error", None, "tx000002");
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.contains("<Code>InternalError</Code>"));
        assert!(!xml_str.contains("<Resource>"));
        assert!(xml_str.contains("<RequestId>tx000002</RequestId>"));
    }

    #[test]
    fn test_should_escape_special_characters() {
        let xml = error_to_xml(
            "InvalidArgument",
            "Value must be < 1024 & > 0",
            Some("/my&bucket"),
            "tx000003",
        );
        let xml_str = std::str::from_utf8(&xml).expect("valid UTF-8");

        assert!(xml_str.contains("Value must be &lt; 1024 &amp; &gt; 0"));
        assert!(xml_str.contains("/my&amp;bucket"));
    }
}
