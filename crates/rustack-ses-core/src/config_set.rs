//! Configuration set and event destination management.
//!
//! Stores configuration sets and their associated event destinations
//! using `DashMap` for concurrent access.

use dashmap::{DashMap, mapref::entry::Entry};
use rustack_ses_model::{
    error::{SesError, SesErrorCode},
    types::EventDestination,
};

/// Internal configuration set record with event destinations.
#[derive(Debug, Clone)]
pub struct ConfigSetRecord {
    /// Configuration set name.
    pub name: String,
    /// Event destinations attached to this configuration set.
    pub event_destinations: Vec<EventDestination>,
}

/// Store for configuration sets and their event destinations.
#[derive(Debug)]
pub struct ConfigurationSetStore {
    /// Configuration sets keyed by name.
    config_sets: DashMap<String, ConfigSetRecord>,
}

impl Default for ConfigurationSetStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigurationSetStore {
    /// Create a new empty configuration set store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_sets: DashMap::new(),
        }
    }

    /// Create a new configuration set.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationSetAlreadyExistsException` if the name is taken.
    pub fn create(&self, name: &str) -> Result<(), SesError> {
        match self.config_sets.entry(name.to_owned()) {
            Entry::Occupied(_) => Err(SesError::with_message(
                SesErrorCode::ConfigurationSetAlreadyExistsException,
                format!("Configuration set <{name}> already exists."),
            )),
            Entry::Vacant(e) => {
                e.insert(ConfigSetRecord {
                    name: name.to_owned(),
                    event_destinations: Vec::new(),
                });
                Ok(())
            }
        }
    }

    /// Delete a configuration set.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationSetDoesNotExistException` if not found.
    pub fn delete(&self, name: &str) -> Result<(), SesError> {
        self.config_sets.remove(name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::ConfigurationSetDoesNotExistException,
                format!("Configuration set <{name}> does not exist."),
            )
        })?;
        Ok(())
    }

    /// Describe a configuration set including its event destinations.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationSetDoesNotExistException` if not found.
    pub fn describe(&self, name: &str) -> Result<ConfigSetRecord, SesError> {
        self.config_sets
            .get(name)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| {
                SesError::with_message(
                    SesErrorCode::ConfigurationSetDoesNotExistException,
                    format!("Configuration set <{name}> does not exist."),
                )
            })
    }

    /// List all configuration set names.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.config_sets.iter().map(|e| e.key().clone()).collect()
    }

    /// Check if a configuration set exists.
    #[must_use]
    pub fn exists(&self, name: &str) -> bool {
        self.config_sets.contains_key(name)
    }

    /// Add an event destination to a configuration set.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationSetDoesNotExistException` if the config set is not found.
    /// Returns `EventDestinationAlreadyExistsException` if the destination name is taken.
    pub fn add_event_destination(
        &self,
        config_set_name: &str,
        destination: EventDestination,
    ) -> Result<(), SesError> {
        let mut entry = self.config_sets.get_mut(config_set_name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::ConfigurationSetDoesNotExistException,
                format!("Configuration set <{config_set_name}> does not exist."),
            )
        })?;
        if entry
            .event_destinations
            .iter()
            .any(|d| d.name == destination.name)
        {
            return Err(SesError::with_message(
                SesErrorCode::EventDestinationAlreadyExistsException,
                format!(
                    "Event destination {} already exists in configuration set {config_set_name}.",
                    destination.name
                ),
            ));
        }
        entry.event_destinations.push(destination);
        Ok(())
    }

    /// Update an event destination within a configuration set.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationSetDoesNotExistException` if the config set is not found.
    /// Returns `EventDestinationDoesNotExistException` if the destination is not found.
    pub fn update_event_destination(
        &self,
        config_set_name: &str,
        destination: EventDestination,
    ) -> Result<(), SesError> {
        let mut entry = self.config_sets.get_mut(config_set_name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::ConfigurationSetDoesNotExistException,
                format!("Configuration set <{config_set_name}> does not exist."),
            )
        })?;
        let pos = entry
            .event_destinations
            .iter()
            .position(|d| d.name == destination.name)
            .ok_or_else(|| {
                SesError::with_message(
                    SesErrorCode::EventDestinationDoesNotExistException,
                    format!("Event destination {} does not exist.", destination.name),
                )
            })?;
        entry.event_destinations[pos] = destination;
        Ok(())
    }

    /// Delete an event destination from a configuration set.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationSetDoesNotExistException` if the config set is not found.
    pub fn delete_event_destination(
        &self,
        config_set_name: &str,
        destination_name: &str,
    ) -> Result<(), SesError> {
        let mut entry = self.config_sets.get_mut(config_set_name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::ConfigurationSetDoesNotExistException,
                format!("Configuration set <{config_set_name}> does not exist."),
            )
        })?;
        entry
            .event_destinations
            .retain(|d| d.name != destination_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_config_set() {
        let store = ConfigurationSetStore::new();
        assert!(store.create("my-set").is_ok());
        assert!(store.exists("my-set"));
    }

    #[test]
    fn test_should_reject_duplicate_config_set() {
        let store = ConfigurationSetStore::new();
        store.create("dup").unwrap_or_default();
        assert!(store.create("dup").is_err());
    }

    #[test]
    fn test_should_delete_config_set() {
        let store = ConfigurationSetStore::new();
        store.create("del").unwrap_or_default();
        assert!(store.delete("del").is_ok());
        assert!(!store.exists("del"));
    }

    #[test]
    fn test_should_return_error_on_delete_nonexistent() {
        let store = ConfigurationSetStore::new();
        assert!(store.delete("nope").is_err());
    }

    #[test]
    fn test_should_describe_config_set() {
        let store = ConfigurationSetStore::new();
        store.create("desc").unwrap_or_default();
        let record = store.describe("desc");
        assert!(record.is_ok());
        assert_eq!(
            record
                .unwrap_or_else(|_| ConfigSetRecord {
                    name: String::new(),
                    event_destinations: Vec::new(),
                })
                .name,
            "desc"
        );
    }

    #[test]
    fn test_should_list_config_sets() {
        let store = ConfigurationSetStore::new();
        store.create("a").unwrap_or_default();
        store.create("b").unwrap_or_default();
        let list = store.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_should_add_event_destination() {
        let store = ConfigurationSetStore::new();
        store.create("set1").unwrap_or_default();
        let dest = EventDestination {
            name: "my-dest".to_owned(),
            enabled: Some(true),
            ..EventDestination::default()
        };
        assert!(store.add_event_destination("set1", dest).is_ok());
        let record = store.describe("set1").unwrap_or_else(|_| ConfigSetRecord {
            name: String::new(),
            event_destinations: Vec::new(),
        });
        assert_eq!(record.event_destinations.len(), 1);
    }

    #[test]
    fn test_should_reject_duplicate_event_destination() {
        let store = ConfigurationSetStore::new();
        store.create("set2").unwrap_or_default();
        let dest = EventDestination {
            name: "dup-dest".to_owned(),
            ..EventDestination::default()
        };
        store
            .add_event_destination("set2", dest.clone())
            .unwrap_or_default();
        assert!(store.add_event_destination("set2", dest).is_err());
    }

    #[test]
    fn test_should_delete_event_destination() {
        let store = ConfigurationSetStore::new();
        store.create("set3").unwrap_or_default();
        let dest = EventDestination {
            name: "del-dest".to_owned(),
            ..EventDestination::default()
        };
        store
            .add_event_destination("set3", dest)
            .unwrap_or_default();
        assert!(store.delete_event_destination("set3", "del-dest").is_ok());
        let record = store.describe("set3").unwrap_or_else(|_| ConfigSetRecord {
            name: String::new(),
            event_destinations: Vec::new(),
        });
        assert!(record.event_destinations.is_empty());
    }

    #[test]
    fn test_should_update_event_destination() {
        let store = ConfigurationSetStore::new();
        store.create("set4").unwrap_or_default();
        let dest = EventDestination {
            name: "upd-dest".to_owned(),
            enabled: Some(false),
            ..EventDestination::default()
        };
        store
            .add_event_destination("set4", dest)
            .unwrap_or_default();
        let updated = EventDestination {
            name: "upd-dest".to_owned(),
            enabled: Some(true),
            ..EventDestination::default()
        };
        assert!(store.update_event_destination("set4", updated).is_ok());
    }
}
