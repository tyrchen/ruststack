//! Template store and rendering for SES email templates.
//!
//! Templates contain `{{variable}}` placeholders that are substituted
//! with values from a JSON data object during rendering.

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use ruststack_ses_model::error::{SesError, SesErrorCode};
use ruststack_ses_model::types::{Template, TemplateMetadata};

/// Store for email templates.
///
/// Templates are keyed by name and contain subject, text body, and HTML body
/// with `{{variable}}` placeholders for substitution.
#[derive(Debug)]
pub struct TemplateStore {
    templates: DashMap<String, StoredTemplate>,
}

/// An email template as stored internally.
#[derive(Debug, Clone)]
pub struct StoredTemplate {
    /// The model template data.
    pub template: Template,
    /// Creation timestamp.
    pub created_timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for TemplateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateStore {
    /// Create a new empty template store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            templates: DashMap::new(),
        }
    }

    /// Create a new template.
    ///
    /// # Errors
    ///
    /// Returns `AlreadyExistsException` if a template with the same name already exists.
    pub fn create(&self, template: Template) -> Result<(), SesError> {
        let name = template.template_name.clone();
        match self.templates.entry(name) {
            Entry::Occupied(_) => Err(SesError::with_message(
                SesErrorCode::AlreadyExistsException,
                format!("Template {} already exists.", template.template_name),
            )),
            Entry::Vacant(e) => {
                e.insert(StoredTemplate {
                    template,
                    created_timestamp: chrono::Utc::now(),
                });
                Ok(())
            }
        }
    }

    /// Get a template by name.
    ///
    /// # Errors
    ///
    /// Returns `TemplateDoesNotExistException` if the template is not found.
    pub fn get(&self, name: &str) -> Result<Template, SesError> {
        self.templates
            .get(name)
            .map(|entry| entry.template.clone())
            .ok_or_else(|| {
                SesError::with_message(
                    SesErrorCode::TemplateDoesNotExistException,
                    format!("Template {name} does not exist."),
                )
            })
    }

    /// Update an existing template.
    ///
    /// # Errors
    ///
    /// Returns `TemplateDoesNotExistException` if the template is not found.
    pub fn update(&self, template: Template) -> Result<(), SesError> {
        let name = template.template_name.clone();
        let mut entry = self.templates.get_mut(&name).ok_or_else(|| {
            SesError::with_message(
                SesErrorCode::TemplateDoesNotExistException,
                format!("Template {name} does not exist."),
            )
        })?;
        entry.template = template;
        Ok(())
    }

    /// Delete a template by name. No error if the template does not exist.
    pub fn delete(&self, name: &str) {
        self.templates.remove(name);
    }

    /// List all templates as metadata.
    #[must_use]
    pub fn list(&self) -> Vec<TemplateMetadata> {
        self.templates
            .iter()
            .map(|entry| TemplateMetadata {
                name: Some(entry.template.template_name.clone()),
                created_timestamp: Some(entry.created_timestamp),
            })
            .collect()
    }
}

/// Render a template by substituting `{{variable}}` placeholders
/// with values from the `template_data` JSON.
///
/// Uses simple Mustache-style substitution. Does not support conditionals,
/// loops, or any advanced Handlebars features.
///
/// # Errors
///
/// Returns `InvalidTemplateException` if the template data is not valid JSON
/// or is not a JSON object.
pub fn render_template(template_text: &str, template_data: &str) -> Result<String, SesError> {
    let data: serde_json::Value = serde_json::from_str(template_data).map_err(|e| {
        SesError::with_message(
            SesErrorCode::InvalidTemplateException,
            format!("Invalid template data JSON: {e}"),
        )
    })?;

    let data_map = data.as_object().ok_or_else(|| {
        SesError::with_message(
            SesErrorCode::InvalidTemplateException,
            "Template data must be a JSON object",
        )
    })?;

    let mut result = template_text.to_owned();
    for (key, value) in data_map {
        let placeholder = format!("{{{{{key}}}}}");
        let replacement = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };
        result = result.replace(&placeholder, &replacement);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_template(name: &str) -> Template {
        Template {
            template_name: name.to_owned(),
            subject_part: Some("Hello {{name}}".to_owned()),
            text_part: Some("Dear {{name}}, welcome!".to_owned()),
            html_part: Some("<p>Dear {{name}}, welcome!</p>".to_owned()),
        }
    }

    #[test]
    fn test_should_create_and_get_template() {
        let store = TemplateStore::new();
        let tmpl = make_template("welcome");
        store.create(tmpl).unwrap_or_default();
        let retrieved = store.get("welcome");
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap_or_default().template_name, "welcome");
    }

    #[test]
    fn test_should_reject_duplicate_template() {
        let store = TemplateStore::new();
        store.create(make_template("dup")).unwrap_or_default();
        let result = store.create(make_template("dup"));
        assert!(result.is_err());
    }

    #[test]
    fn test_should_update_template() {
        let store = TemplateStore::new();
        store.create(make_template("upd")).unwrap_or_default();
        let mut updated = make_template("upd");
        updated.subject_part = Some("Updated {{name}}".to_owned());
        store.update(updated).unwrap_or_default();
        let retrieved = store.get("upd").unwrap_or_default();
        assert_eq!(retrieved.subject_part, Some("Updated {{name}}".to_owned()));
    }

    #[test]
    fn test_should_return_error_on_update_nonexistent() {
        let store = TemplateStore::new();
        let result = store.update(make_template("nope"));
        assert!(result.is_err());
    }

    #[test]
    fn test_should_delete_template() {
        let store = TemplateStore::new();
        store.create(make_template("del")).unwrap_or_default();
        store.delete("del");
        assert!(store.get("del").is_err());
    }

    #[test]
    fn test_should_list_templates() {
        let store = TemplateStore::new();
        store.create(make_template("a")).unwrap_or_default();
        store.create(make_template("b")).unwrap_or_default();
        let list = store.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_should_render_simple_template() {
        let result = render_template("Hello {{name}}", r#"{"name":"World"}"#);
        assert_eq!(result.unwrap_or_default(), "Hello World");
    }

    #[test]
    fn test_should_render_multiple_variables() {
        let result = render_template(
            "{{greeting}} {{name}}!",
            r#"{"greeting":"Hi","name":"Alice"}"#,
        );
        assert_eq!(result.unwrap_or_default(), "Hi Alice!");
    }

    #[test]
    fn test_should_leave_unmatched_placeholders() {
        let result = render_template("Hello {{name}} {{missing}}", r#"{"name":"World"}"#);
        assert_eq!(result.unwrap_or_default(), "Hello World {{missing}}");
    }

    #[test]
    fn test_should_handle_null_values() {
        let result = render_template("Value: {{x}}", r#"{"x":null}"#);
        assert_eq!(result.unwrap_or_default(), "Value: ");
    }

    #[test]
    fn test_should_handle_numeric_values() {
        let result = render_template("Count: {{n}}", r#"{"n":42}"#);
        assert_eq!(result.unwrap_or_default(), "Count: 42");
    }

    #[test]
    fn test_should_reject_invalid_json() {
        let result = render_template("Hello", "not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_reject_non_object_json() {
        let result = render_template("Hello", "[1,2,3]");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_handle_empty_data() {
        let result = render_template("Hello {{name}}", "{}");
        assert_eq!(result.unwrap_or_default(), "Hello {{name}}");
    }
}
