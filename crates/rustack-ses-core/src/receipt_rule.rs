//! Receipt rule set management.
//!
//! Receipt rules are accepted and stored but never actually process
//! incoming email in the local development emulator. This is for
//! API compatibility only.

use dashmap::{DashMap, mapref::entry::Entry};
use parking_lot::RwLock;
use rustack_ses_model::{
    error::{SesError, SesErrorCode},
    types::{ReceiptRule, ReceiptRuleSetMetadata},
};

/// Internal receipt rule set record.
#[derive(Debug, Clone)]
pub struct ReceiptRuleSetRecord {
    /// Rule set name.
    pub name: String,
    /// Rules within this rule set.
    pub rules: Vec<ReceiptRule>,
    /// Creation timestamp.
    pub created_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Store for receipt rule sets and rules.
///
/// Receipt rules are accepted and stored but never actually process
/// incoming email. This is for API compatibility only.
#[derive(Debug)]
pub struct ReceiptRuleSetStore {
    rule_sets: DashMap<String, ReceiptRuleSetRecord>,
    active_rule_set: RwLock<Option<String>>,
}

impl Default for ReceiptRuleSetStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceiptRuleSetStore {
    /// Create a new empty receipt rule set store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rule_sets: DashMap::new(),
            active_rule_set: RwLock::new(None),
        }
    }

    /// Create a new receipt rule set.
    ///
    /// # Errors
    ///
    /// Returns `AlreadyExistsException` if a rule set with the same name exists.
    pub fn create_rule_set(&self, name: &str) -> Result<(), SesError> {
        match self.rule_sets.entry(name.to_owned()) {
            Entry::Occupied(_) => Err(SesError::with_message(
                SesErrorCode::AlreadyExistsException,
                format!("Receipt rule set <{name}> already exists."),
            )),
            Entry::Vacant(e) => {
                e.insert(ReceiptRuleSetRecord {
                    name: name.to_owned(),
                    rules: Vec::new(),
                    created_timestamp: chrono::Utc::now(),
                });
                Ok(())
            }
        }
    }

    /// Delete a receipt rule set.
    ///
    /// # Errors
    ///
    /// Returns `RuleSetDoesNotExistException` if not found.
    pub fn delete_rule_set(&self, name: &str) -> Result<(), SesError> {
        self.rule_sets.remove(name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::RuleSetDoesNotExistException,
                format!("Receipt rule set <{name}> does not exist."),
            )
        })?;
        // If the deleted rule set was active, clear the active rule set
        let mut active = self.active_rule_set.write();
        if active.as_deref() == Some(name) {
            *active = None;
        }
        Ok(())
    }

    /// Describe a receipt rule set.
    ///
    /// # Errors
    ///
    /// Returns `RuleSetDoesNotExistException` if not found.
    pub fn describe_rule_set(&self, name: &str) -> Result<ReceiptRuleSetRecord, SesError> {
        self.rule_sets
            .get(name)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| {
                SesError::with_message(
                    SesErrorCode::RuleSetDoesNotExistException,
                    format!("Receipt rule set <{name}> does not exist."),
                )
            })
    }

    /// Create a receipt rule within a rule set.
    ///
    /// # Errors
    ///
    /// Returns `RuleSetDoesNotExistException` if the rule set is not found.
    pub fn create_rule(
        &self,
        rule_set_name: &str,
        rule: ReceiptRule,
        after: Option<&str>,
    ) -> Result<(), SesError> {
        let mut entry = self.rule_sets.get_mut(rule_set_name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::RuleSetDoesNotExistException,
                format!("Receipt rule set <{rule_set_name}> does not exist."),
            )
        })?;

        if let Some(after_name) = after {
            let pos = entry
                .rules
                .iter()
                .position(|r| r.name == after_name)
                .map_or(entry.rules.len(), |p| p + 1);
            entry.rules.insert(pos, rule);
        } else {
            entry.rules.push(rule);
        }
        Ok(())
    }

    /// Delete a receipt rule from a rule set.
    ///
    /// # Errors
    ///
    /// Returns `RuleSetDoesNotExistException` if the rule set is not found.
    pub fn delete_rule(&self, rule_set_name: &str, rule_name: &str) -> Result<(), SesError> {
        let mut entry = self.rule_sets.get_mut(rule_set_name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::RuleSetDoesNotExistException,
                format!("Receipt rule set <{rule_set_name}> does not exist."),
            )
        })?;
        entry.rules.retain(|r| r.name != rule_name);
        Ok(())
    }

    /// Clone a receipt rule set to a new name.
    ///
    /// # Errors
    ///
    /// Returns `RuleSetDoesNotExistException` if the source does not exist.
    /// Returns `AlreadyExistsException` if the destination already exists.
    pub fn clone_rule_set(&self, source_name: &str, dest_name: &str) -> Result<(), SesError> {
        let source = self.describe_rule_set(source_name)?;
        match self.rule_sets.entry(dest_name.to_owned()) {
            Entry::Occupied(_) => Err(SesError::with_message(
                SesErrorCode::AlreadyExistsException,
                format!("Receipt rule set <{dest_name}> already exists."),
            )),
            Entry::Vacant(e) => {
                e.insert(ReceiptRuleSetRecord {
                    name: dest_name.to_owned(),
                    rules: source.rules.clone(),
                    created_timestamp: chrono::Utc::now(),
                });
                Ok(())
            }
        }
    }

    /// Get the active receipt rule set name and metadata.
    #[must_use]
    pub fn get_active_rule_set(&self) -> Option<(ReceiptRuleSetMetadata, Vec<ReceiptRule>)> {
        let active = self.active_rule_set.read();
        let name = active.as_ref()?;
        let record = self.rule_sets.get(name)?;
        Some((
            ReceiptRuleSetMetadata {
                name: Some(record.name.clone()),
                created_timestamp: Some(record.created_timestamp),
            },
            record.rules.clone(),
        ))
    }

    /// Set the active receipt rule set.
    ///
    /// # Errors
    ///
    /// Returns `RuleSetDoesNotExistException` if the rule set is not found
    /// (when a name is provided).
    pub fn set_active_rule_set(&self, name: Option<&str>) -> Result<(), SesError> {
        if let Some(name) = name {
            if !self.rule_sets.contains_key(name) {
                return Err(SesError::with_message(
                    SesErrorCode::RuleSetDoesNotExistException,
                    format!("Receipt rule set <{name}> does not exist."),
                ));
            }
            *self.active_rule_set.write() = Some(name.to_owned());
        } else {
            *self.active_rule_set.write() = None;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_rule_set() {
        let store = ReceiptRuleSetStore::new();
        assert!(store.create_rule_set("my-rules").is_ok());
    }

    #[test]
    fn test_should_reject_duplicate_rule_set() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("dup").unwrap_or_default();
        assert!(store.create_rule_set("dup").is_err());
    }

    #[test]
    fn test_should_delete_rule_set() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("del").unwrap_or_default();
        assert!(store.delete_rule_set("del").is_ok());
        assert!(store.describe_rule_set("del").is_err());
    }

    #[test]
    fn test_should_describe_rule_set() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("desc").unwrap_or_default();
        let record = store.describe_rule_set("desc");
        assert!(record.is_ok());
        assert_eq!(
            record
                .unwrap_or_else(|_| ReceiptRuleSetRecord {
                    name: String::new(),
                    rules: Vec::new(),
                    created_timestamp: chrono::Utc::now(),
                })
                .name,
            "desc"
        );
    }

    #[test]
    fn test_should_create_rule_in_set() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("set1").unwrap_or_default();
        let rule = ReceiptRule {
            name: "rule1".to_owned(),
            enabled: Some(true),
            ..ReceiptRule::default()
        };
        assert!(store.create_rule("set1", rule, None).is_ok());
        let record = store
            .describe_rule_set("set1")
            .unwrap_or_else(|_| ReceiptRuleSetRecord {
                name: String::new(),
                rules: Vec::new(),
                created_timestamp: chrono::Utc::now(),
            });
        assert_eq!(record.rules.len(), 1);
    }

    #[test]
    fn test_should_create_rule_after_existing() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("set2").unwrap_or_default();
        let rule1 = ReceiptRule {
            name: "rule1".to_owned(),
            ..ReceiptRule::default()
        };
        let rule2 = ReceiptRule {
            name: "rule2".to_owned(),
            ..ReceiptRule::default()
        };
        let rule3 = ReceiptRule {
            name: "rule3".to_owned(),
            ..ReceiptRule::default()
        };
        store.create_rule("set2", rule1, None).unwrap_or_default();
        store.create_rule("set2", rule2, None).unwrap_or_default();
        store
            .create_rule("set2", rule3, Some("rule1"))
            .unwrap_or_default();
        let record = store
            .describe_rule_set("set2")
            .unwrap_or_else(|_| ReceiptRuleSetRecord {
                name: String::new(),
                rules: Vec::new(),
                created_timestamp: chrono::Utc::now(),
            });
        assert_eq!(record.rules[0].name, "rule1");
        assert_eq!(record.rules[1].name, "rule3");
        assert_eq!(record.rules[2].name, "rule2");
    }

    #[test]
    fn test_should_delete_rule() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("set3").unwrap_or_default();
        let rule = ReceiptRule {
            name: "rule1".to_owned(),
            ..ReceiptRule::default()
        };
        store.create_rule("set3", rule, None).unwrap_or_default();
        assert!(store.delete_rule("set3", "rule1").is_ok());
        let record = store
            .describe_rule_set("set3")
            .unwrap_or_else(|_| ReceiptRuleSetRecord {
                name: String::new(),
                rules: Vec::new(),
                created_timestamp: chrono::Utc::now(),
            });
        assert!(record.rules.is_empty());
    }

    #[test]
    fn test_should_clone_rule_set() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("source").unwrap_or_default();
        let rule = ReceiptRule {
            name: "rule1".to_owned(),
            ..ReceiptRule::default()
        };
        store.create_rule("source", rule, None).unwrap_or_default();
        assert!(store.clone_rule_set("source", "dest").is_ok());
        let record = store
            .describe_rule_set("dest")
            .unwrap_or_else(|_| ReceiptRuleSetRecord {
                name: String::new(),
                rules: Vec::new(),
                created_timestamp: chrono::Utc::now(),
            });
        assert_eq!(record.rules.len(), 1);
        assert_eq!(record.rules[0].name, "rule1");
    }

    #[test]
    fn test_should_set_and_get_active_rule_set() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("active-set").unwrap_or_default();
        assert!(store.get_active_rule_set().is_none());
        assert!(store.set_active_rule_set(Some("active-set")).is_ok());
        let active = store.get_active_rule_set();
        assert!(active.is_some());
        let (metadata, _) = active.unwrap_or_default();
        assert_eq!(metadata.name, Some("active-set".to_owned()));
    }

    #[test]
    fn test_should_clear_active_on_delete() {
        let store = ReceiptRuleSetStore::new();
        store.create_rule_set("doomed").unwrap_or_default();
        store
            .set_active_rule_set(Some("doomed"))
            .unwrap_or_default();
        store.delete_rule_set("doomed").unwrap_or_default();
        assert!(store.get_active_rule_set().is_none());
    }
}
