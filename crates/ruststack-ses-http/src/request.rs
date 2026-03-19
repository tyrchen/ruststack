//! SES `awsQuery` request parameter parsing utilities.
//!
//! SES v1 uses `application/x-www-form-urlencoded` request bodies with
//! dot-notation for nested parameters (e.g., `Destination.ToAddresses.member.1`).

use std::collections::HashMap;

use ruststack_ses_model::error::{SesError, SesErrorCode};

/// Parse a URL-encoded body into a list of key-value pairs.
#[must_use]
pub fn parse_form_params(body: &[u8]) -> Vec<(String, String)> {
    form_urlencoded::parse(body)
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

/// Get a required parameter value.
///
/// Returns an error if the parameter is not present.
pub fn get_required_param<'a>(
    params: &'a [(String, String)],
    key: &str,
) -> Result<&'a str, SesError> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::InvalidParameterValue,
                format!("Missing required parameter: {key}"),
            )
        })
}

/// Get an optional parameter value.
#[must_use]
pub fn get_optional_param<'a>(params: &'a [(String, String)], key: &str) -> Option<&'a str> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// Get an optional boolean parameter.
///
/// Parses `"true"` / `"false"` (case-insensitive). Returns `None` if
/// the parameter is absent.
#[must_use]
pub fn get_optional_bool(params: &[(String, String)], key: &str) -> Option<bool> {
    get_optional_param(params, key).map(|v| v.eq_ignore_ascii_case("true"))
}

/// Get an optional i32 parameter.
#[must_use]
pub fn get_optional_i32(params: &[(String, String)], key: &str) -> Option<i32> {
    get_optional_param(params, key).and_then(|v| v.parse().ok())
}

/// Parse a `member.N` list from form parameters.
///
/// Given parameters like `Prefix.member.1=value1`, `Prefix.member.2=value2`,
/// collects them into a `Vec<String>`.
#[must_use]
pub fn parse_member_list(params: &[(String, String)], prefix: &str) -> Vec<String> {
    let member_prefix = format!("{prefix}.member.");
    let mut items: Vec<(u32, String)> = Vec::new();
    for (k, v) in params {
        if let Some(rest) = k.strip_prefix(&member_prefix) {
            if let Ok(idx) = rest.parse::<u32>() {
                items.push((idx, v.clone()));
            }
        }
    }
    items.sort_by_key(|(idx, _)| *idx);
    items.into_iter().map(|(_, v)| v).collect()
}

/// Parse message tags from form parameters.
///
/// Tags follow the pattern:
/// `Tags.member.N.Name=key`, `Tags.member.N.Value=value`
#[must_use]
pub fn parse_tag_list(params: &[(String, String)], prefix: &str) -> Vec<(String, String)> {
    let member_prefix = format!("{prefix}.member.");
    let indices = collect_indices(params, &member_prefix);
    let mut tags = Vec::new();
    for idx in indices {
        let name_key = format!("{member_prefix}{idx}.Name");
        let value_key = format!("{member_prefix}{idx}.Value");
        if let (Some(name), Some(value)) = (
            get_optional_param(params, &name_key),
            get_optional_param(params, &value_key),
        ) {
            tags.push((name.to_owned(), value.to_owned()));
        }
    }
    tags
}

/// Parse an attributes map from `Prefix.entry.N.key` / `Prefix.entry.N.value`.
pub fn parse_attributes_map(
    params: &[(String, String)],
    prefix: &str,
) -> Result<HashMap<String, String>, SesError> {
    let mut result = HashMap::new();
    let entry_prefix = format!("{prefix}.entry.");
    let indices = collect_indices(params, &entry_prefix);

    for idx in indices {
        let key_param = format!("{entry_prefix}{idx}.key");
        let value_param = format!("{entry_prefix}{idx}.value");

        let key = get_required_param(params, &key_param)?;
        let value = get_optional_param(params, &value_param).unwrap_or("");
        result.insert(key.to_owned(), value.to_owned());
    }
    Ok(result)
}

/// Parse query parameters from a URI query string.
#[must_use]
pub fn parse_query_params(query: Option<&str>) -> HashMap<String, String> {
    let mut params = HashMap::new();
    if let Some(q) = query {
        for (k, v) in form_urlencoded::parse(q.as_bytes()) {
            params.insert(k.into_owned(), v.into_owned());
        }
    }
    params
}

/// Collect unique numeric indices from params matching a prefix pattern.
fn collect_indices(params: &[(String, String)], prefix: &str) -> Vec<u32> {
    let mut indices: Vec<u32> = Vec::new();
    for (k, _) in params {
        if let Some(rest) = k.strip_prefix(prefix) {
            if let Some(idx_str) = rest.split('.').next() {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    if !indices.contains(&idx) {
                        indices.push(idx);
                    }
                }
            }
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_form_params() {
        let body = b"Action=SendEmail&Source=sender%40example.com&Version=2010-12-01";
        let params = parse_form_params(body);
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], ("Action".to_owned(), "SendEmail".to_owned()));
        assert_eq!(
            params[1],
            ("Source".to_owned(), "sender@example.com".to_owned())
        );
    }

    #[test]
    fn test_should_get_required_param() {
        let params = vec![("Source".to_owned(), "test@example.com".to_owned())];
        assert_eq!(
            get_required_param(&params, "Source").unwrap(),
            "test@example.com"
        );
    }

    #[test]
    fn test_should_error_on_missing_required_param() {
        let params: Vec<(String, String)> = vec![];
        let err = get_required_param(&params, "Source").unwrap_err();
        assert!(err.message.contains("Source"));
    }

    #[test]
    fn test_should_parse_member_list() {
        let params = vec![
            (
                "Destination.ToAddresses.member.1".to_owned(),
                "a@example.com".to_owned(),
            ),
            (
                "Destination.ToAddresses.member.2".to_owned(),
                "b@example.com".to_owned(),
            ),
        ];
        let list = parse_member_list(&params, "Destination.ToAddresses");
        assert_eq!(list, vec!["a@example.com", "b@example.com"]);
    }

    #[test]
    fn test_should_parse_tag_list() {
        let params = vec![
            ("Tags.member.1.Name".to_owned(), "campaign".to_owned()),
            ("Tags.member.1.Value".to_owned(), "welcome".to_owned()),
            ("Tags.member.2.Name".to_owned(), "env".to_owned()),
            ("Tags.member.2.Value".to_owned(), "test".to_owned()),
        ];
        let tags = parse_tag_list(&params, "Tags");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], ("campaign".to_owned(), "welcome".to_owned()));
        assert_eq!(tags[1], ("env".to_owned(), "test".to_owned()));
    }

    #[test]
    fn test_should_parse_query_params() {
        let params = parse_query_params(Some("id=abc&email=test@example.com"));
        assert_eq!(params.get("id").unwrap(), "abc");
        assert_eq!(params.get("email").unwrap(), "test@example.com");
    }

    #[test]
    fn test_should_parse_empty_query_params() {
        let params = parse_query_params(None);
        assert!(params.is_empty());
    }
}
