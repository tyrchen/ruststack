//! Filter evaluation for `DescribeParameters`.
//!
//! Implements the `ParameterStringFilter` matching logic used by the
//! `DescribeParameters` operation. Each filter key supports specific
//! comparison options as documented in the AWS SSM API reference.

use rustack_ssm_model::{
    error::{SsmError, SsmErrorCode},
    types::ParameterStringFilter,
};

use crate::storage::ParameterRecord;

/// Valid filter keys for `DescribeParameters`.
const VALID_FILTER_KEYS: &[&str] = &["Name", "Type", "KeyId", "Path", "Tier", "DataType", "Label"];

/// Evaluate whether a parameter record matches all the given filters.
///
/// Returns `true` if all filters match (AND logic). If the filters slice
/// is empty, all parameters match.
#[must_use]
pub fn matches_filters(record: &ParameterRecord, filters: &[ParameterStringFilter]) -> bool {
    filters.iter().all(|f| matches_single_filter(record, f))
}

/// Validate all filters before evaluation.
///
/// # Errors
///
/// Returns an error if a filter key is invalid, the option is not supported
/// for the given key, or the values list is empty.
pub fn validate_filters(filters: &[ParameterStringFilter]) -> Result<(), SsmError> {
    for filter in filters {
        validate_single_filter(filter)?;
    }
    Ok(())
}

/// Validate a single filter.
fn validate_single_filter(filter: &ParameterStringFilter) -> Result<(), SsmError> {
    let key = filter.key.as_str();

    // Check for tag: prefix
    if let Some(tag_key) = key.strip_prefix("tag:") {
        if tag_key.is_empty() {
            return Err(SsmError::with_message(
                SsmErrorCode::InvalidFilterKey,
                "The filter key 'tag:' requires a tag key name after the prefix.",
            ));
        }
        // tag: filters only support Equals
        if let Some(ref opt) = filter.option {
            if opt != "Equals" {
                return Err(SsmError::with_message(
                    SsmErrorCode::InvalidFilterOption,
                    format!(
                        "The filter option '{opt}' is not valid for key '{key}'. Valid options: \
                         Equals."
                    ),
                ));
            }
        }
        if filter.values.is_empty() {
            return Err(SsmError::with_message(
                SsmErrorCode::InvalidFilterValue,
                format!("The filter for key '{key}' must have at least one value."),
            ));
        }
        return Ok(());
    }

    // Check standard filter keys
    if !VALID_FILTER_KEYS.contains(&key) {
        return Err(SsmError::with_message(
            SsmErrorCode::InvalidFilterKey,
            format!(
                "The filter key '{key}' is not valid. Valid filter keys: Name, Type, KeyId, Path, \
                 Tier, DataType, Label, tag:<key>."
            ),
        ));
    }

    // Validate option per key
    if let Some(ref opt) = filter.option {
        let valid_options = match key {
            "Name" => &["Equals", "BeginsWith"][..],
            "Path" => &["Recursive", "OneLevel"][..],
            "Type" | "KeyId" | "Tier" | "DataType" | "Label" => &["Equals"][..],
            _ => &[][..],
        };
        if !valid_options.contains(&opt.as_str()) {
            return Err(SsmError::with_message(
                SsmErrorCode::InvalidFilterOption,
                format!(
                    "The filter option '{opt}' is not valid for key '{key}'. Valid options: \
                     {valid_options:?}."
                ),
            ));
        }
    }

    if filter.values.is_empty() {
        return Err(SsmError::with_message(
            SsmErrorCode::InvalidFilterValue,
            format!("The filter for key '{key}' must have at least one value."),
        ));
    }

    Ok(())
}

/// Evaluate a single filter against a parameter record.
fn matches_single_filter(record: &ParameterRecord, filter: &ParameterStringFilter) -> bool {
    let key = filter.key.as_str();

    // Handle tag: prefix filters
    if let Some(tag_key) = key.strip_prefix("tag:") {
        return match record.tags.get(tag_key) {
            Some(tag_value) => filter.values.iter().any(|v| v == tag_value),
            None => false,
        };
    }

    match key {
        "Name" => {
            let option = filter.option.as_deref().unwrap_or("Equals");
            match option {
                "Equals" => filter.values.iter().any(|v| v == &record.name),
                "BeginsWith" => filter
                    .values
                    .iter()
                    .any(|v| record.name.starts_with(v.as_str())),
                _ => false,
            }
        }
        "Type" => {
            let type_str = record.parameter_type.as_str();
            filter.values.iter().any(|v| v == type_str)
        }
        "KeyId" => match &record.key_id {
            Some(kid) => filter.values.iter().any(|v| v == kid),
            None => false,
        },
        "Path" => {
            let option = filter.option.as_deref().unwrap_or("OneLevel");
            filter.values.iter().any(|path_prefix| {
                let normalized = if path_prefix.ends_with('/') {
                    path_prefix.clone()
                } else {
                    format!("{path_prefix}/")
                };
                if !record.name.starts_with(&normalized) {
                    return false;
                }
                let remainder = &record.name[normalized.len()..];
                if remainder.is_empty() {
                    return false;
                }
                match option {
                    "Recursive" => true,
                    _ => !remainder.contains('/'),
                }
            })
        }
        "Tier" => {
            // Use latest version's tier
            if let Some(ver) = record.versions.get(&record.current_version) {
                let tier_str = ver.tier.as_str();
                filter.values.iter().any(|v| v == tier_str)
            } else {
                false
            }
        }
        "DataType" => {
            if let Some(ver) = record.versions.get(&record.current_version) {
                filter.values.iter().any(|v| v == &ver.data_type)
            } else {
                false
            }
        }
        "Label" => {
            // Match if any version has a matching label
            filter.values.iter().any(|label| {
                record
                    .versions
                    .values()
                    .any(|ver| ver.labels.contains(label.as_str()))
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap, HashSet};

    use rustack_ssm_model::types::{ParameterStringFilter, ParameterTier, ParameterType};

    use super::*;
    use crate::storage::ParameterVersion;

    fn make_record(name: &str) -> ParameterRecord {
        let version = ParameterVersion {
            version: 1,
            value: "test".to_owned(),
            description: None,
            allowed_pattern: None,
            data_type: "text".to_owned(),
            tier: ParameterTier::Standard,
            labels: HashSet::new(),
            policies: vec![],
            last_modified_date: 0.0,
            last_modified_user: "arn:aws:iam::root".to_owned(),
        };
        let mut versions = BTreeMap::new();
        versions.insert(1, version);
        ParameterRecord {
            name: name.to_owned(),
            current_version: 1,
            versions,
            tags: HashMap::new(),
            parameter_type: ParameterType::String,
            key_id: None,
        }
    }

    #[test]
    fn test_should_match_name_equals() {
        let record = make_record("/app/db/host");
        let filter = ParameterStringFilter {
            key: "Name".to_owned(),
            option: Some("Equals".to_owned()),
            values: vec!["/app/db/host".to_owned()],
        };
        assert!(matches_filters(&record, &[filter]));
    }

    #[test]
    fn test_should_match_name_begins_with() {
        let record = make_record("/app/db/host");
        let filter = ParameterStringFilter {
            key: "Name".to_owned(),
            option: Some("BeginsWith".to_owned()),
            values: vec!["/app/db".to_owned()],
        };
        assert!(matches_filters(&record, &[filter]));
    }

    #[test]
    fn test_should_not_match_name_wrong_value() {
        let record = make_record("/app/db/host");
        let filter = ParameterStringFilter {
            key: "Name".to_owned(),
            option: Some("Equals".to_owned()),
            values: vec!["/other/param".to_owned()],
        };
        assert!(!matches_filters(&record, &[filter]));
    }

    #[test]
    fn test_should_match_type() {
        let record = make_record("/test");
        let filter = ParameterStringFilter {
            key: "Type".to_owned(),
            option: Some("Equals".to_owned()),
            values: vec!["String".to_owned()],
        };
        assert!(matches_filters(&record, &[filter]));
    }

    #[test]
    fn test_should_match_path_one_level() {
        let record = make_record("/app/db/host");
        let filter = ParameterStringFilter {
            key: "Path".to_owned(),
            option: Some("OneLevel".to_owned()),
            values: vec!["/app/db".to_owned()],
        };
        assert!(matches_filters(&record, std::slice::from_ref(&filter)));

        // Nested should not match one-level
        let deep = make_record("/app/db/sub/host");
        assert!(!matches_filters(&deep, &[filter]));
    }

    #[test]
    fn test_should_match_path_recursive() {
        let record = make_record("/app/db/sub/host");
        let filter = ParameterStringFilter {
            key: "Path".to_owned(),
            option: Some("Recursive".to_owned()),
            values: vec!["/app".to_owned()],
        };
        assert!(matches_filters(&record, &[filter]));
    }

    #[test]
    fn test_should_match_tag() {
        let mut record = make_record("/test");
        record
            .tags
            .insert("env".to_owned(), "production".to_owned());
        let filter = ParameterStringFilter {
            key: "tag:env".to_owned(),
            option: Some("Equals".to_owned()),
            values: vec!["production".to_owned()],
        };
        assert!(matches_filters(&record, &[filter]));
    }

    #[test]
    fn test_should_reject_invalid_filter_key() {
        let filter = ParameterStringFilter {
            key: "Invalid".to_owned(),
            option: None,
            values: vec!["val".to_owned()],
        };
        assert!(validate_filters(&[filter]).is_err());
    }

    #[test]
    fn test_should_reject_invalid_filter_option() {
        let filter = ParameterStringFilter {
            key: "Type".to_owned(),
            option: Some("BeginsWith".to_owned()),
            values: vec!["String".to_owned()],
        };
        let err = validate_filters(&[filter]).unwrap_err();
        assert_eq!(err.code, SsmErrorCode::InvalidFilterOption);
    }

    #[test]
    fn test_should_reject_empty_filter_values() {
        let filter = ParameterStringFilter {
            key: "Name".to_owned(),
            option: None,
            values: vec![],
        };
        let err = validate_filters(&[filter]).unwrap_err();
        assert_eq!(err.code, SsmErrorCode::InvalidFilterValue);
    }

    #[test]
    fn test_should_match_empty_filters() {
        let record = make_record("/test");
        assert!(matches_filters(&record, &[]));
    }
}
