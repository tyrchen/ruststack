//! SES handler implementation bridging HTTP to business logic.
//!
//! Parses form-urlencoded request bodies, dispatches to the provider,
//! and serializes XML responses following the awsQuery protocol.
//!
//! Covers Phase 0-3: identity management, email sending, templates,
//! configuration sets, event destinations, receipt rules, identity
//! notification/DKIM/mail-from configuration, and sending authorization.

use std::{future::Future, pin::Pin, sync::Arc};

use base64::Engine;
use bytes::Bytes;
use ruststack_ses_http::{
    body::SesResponseBody,
    dispatch::SesHandler,
    request::{
        get_optional_bool, get_optional_param, get_required_param, parse_form_params,
        parse_member_list, parse_tag_list,
    },
    response::{XmlWriter, xml_response},
};
use ruststack_ses_model::{
    error::SesError,
    input::{
        CloneReceiptRuleSetInput, CreateConfigurationSetEventDestinationInput,
        CreateConfigurationSetInput, CreateReceiptRuleInput, CreateReceiptRuleSetInput,
        CreateTemplateInput, DeleteConfigurationSetEventDestinationInput,
        DeleteConfigurationSetInput, DeleteIdentityInput, DeleteIdentityPolicyInput,
        DeleteReceiptRuleInput, DeleteReceiptRuleSetInput, DeleteTemplateInput,
        DeleteVerifiedEmailAddressInput, DescribeActiveReceiptRuleSetInput,
        DescribeConfigurationSetInput, DescribeReceiptRuleSetInput, GetIdentityDkimAttributesInput,
        GetIdentityMailFromDomainAttributesInput, GetIdentityNotificationAttributesInput,
        GetIdentityPoliciesInput, GetIdentityVerificationAttributesInput, GetTemplateInput,
        ListConfigurationSetsInput, ListIdentitiesInput, ListIdentityPoliciesInput,
        ListTemplatesInput, PutIdentityPolicyInput, SendEmailInput, SendRawEmailInput,
        SendTemplatedEmailInput, SetActiveReceiptRuleSetInput,
        SetIdentityFeedbackForwardingEnabledInput, SetIdentityMailFromDomainInput,
        SetIdentityNotificationTopicInput, UpdateConfigurationSetEventDestinationInput,
        UpdateTemplateInput, VerifyDomainDkimInput, VerifyDomainIdentityInput,
        VerifyEmailAddressInput, VerifyEmailIdentityInput,
    },
    operations::SesOperation,
    types::{
        BehaviorOnMXFailure, Body, ConfigurationSet, Content, Destination, EventDestination,
        IdentityType, Message, MessageTag, NotificationType, RawMessage, ReceiptRule, Template,
    },
};

use crate::provider::RustStackSes;

/// Handler that bridges the HTTP layer to the SES provider.
#[derive(Debug)]
pub struct RustStackSesHandler {
    provider: Arc<RustStackSes>,
}

impl RustStackSesHandler {
    /// Create a new handler wrapping a provider.
    #[must_use]
    pub fn new(provider: Arc<RustStackSes>) -> Self {
        Self { provider }
    }

    /// Get access to the underlying provider.
    #[must_use]
    pub fn provider(&self) -> &RustStackSes {
        &self.provider
    }
}

impl SesHandler for RustStackSesHandler {
    fn handle_operation(
        &self,
        op: SesOperation,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SesResponseBody>, SesError>> + Send>>
    {
        let provider = Arc::clone(&self.provider);
        Box::pin(async move { dispatch(&provider, op, &body) })
    }

    fn handle_v2_operation(
        &self,
        _method: http::Method,
        _path: String,
        _body: Bytes,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SesResponseBody>, SesError>> + Send>>
    {
        Box::pin(async { Err(SesError::not_implemented("SES v2")) })
    }

    fn query_emails(&self, filter_id: Option<&str>, filter_source: Option<&str>) -> String {
        let messages = self.provider.emails.query(filter_id, filter_source);
        let body = serde_json::json!({ "messages": messages });
        body.to_string()
    }

    fn clear_emails(&self, filter_id: Option<&str>) {
        if let Some(id) = filter_id {
            self.provider.emails.remove(id);
        } else {
            self.provider.emails.clear();
        }
    }
}

/// Dispatch an SES operation to the appropriate handler method.
#[allow(clippy::too_many_lines)]
fn dispatch(
    provider: &RustStackSes,
    op: SesOperation,
    body: &[u8],
) -> Result<http::Response<SesResponseBody>, SesError> {
    let params = parse_form_params(body);
    let request_id = uuid::Uuid::new_v4().to_string();

    match op {
        // ---------------------------------------------------------------
        // Phase 0: Core Sending + Identities
        // ---------------------------------------------------------------
        SesOperation::VerifyEmailIdentity => {
            let input = VerifyEmailIdentityInput {
                email_address: get_required_param(&params, "EmailAddress")?.to_owned(),
            };
            provider.verify_email_identity(input)?;
            Ok(empty_xml_response("VerifyEmailIdentity", &request_id))
        }

        SesOperation::VerifyDomainIdentity => {
            let input = VerifyDomainIdentityInput {
                domain: get_required_param(&params, "Domain")?.to_owned(),
            };
            let output = provider.verify_domain_identity(input)?;
            let mut w = XmlWriter::new();
            w.start_response("VerifyDomainIdentity");
            w.start_result("VerifyDomainIdentity");
            w.write_element("VerificationToken", &output.verification_token);
            w.end_element("VerifyDomainIdentityResult");
            w.write_response_metadata(&request_id);
            w.end_element("VerifyDomainIdentityResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::ListIdentities => {
            let identity_type = get_optional_param(&params, "IdentityType").map(|s| {
                if s == "EmailAddress" {
                    IdentityType::EmailAddress
                } else {
                    IdentityType::Domain
                }
            });
            let input = ListIdentitiesInput {
                identity_type,
                max_items: None,
                next_token: None,
            };
            let output = provider.list_identities(input)?;
            let mut w = XmlWriter::new();
            w.start_response("ListIdentities");
            w.start_result("ListIdentities");
            w.raw("<Identities>");
            for identity in &output.identities {
                w.write_element("member", identity);
            }
            w.raw("</Identities>");
            w.write_optional_element("NextToken", output.next_token.as_deref());
            w.end_element("ListIdentitiesResult");
            w.write_response_metadata(&request_id);
            w.end_element("ListIdentitiesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::DeleteIdentity => {
            let input = DeleteIdentityInput {
                identity: get_required_param(&params, "Identity")?.to_owned(),
            };
            provider.delete_identity(input)?;
            Ok(empty_xml_response("DeleteIdentity", &request_id))
        }

        SesOperation::GetIdentityVerificationAttributes => {
            let identities = parse_member_list(&params, "Identities");
            let input = GetIdentityVerificationAttributesInput { identities };
            let output = provider.get_identity_verification_attributes(input)?;
            let mut w = XmlWriter::new();
            w.start_response("GetIdentityVerificationAttributes");
            w.start_result("GetIdentityVerificationAttributes");
            w.raw("<VerificationAttributes>");
            for (identity, attrs) in &output.verification_attributes {
                w.raw("<entry>");
                w.write_element("key", identity);
                w.raw("<value>");
                w.write_element("VerificationStatus", attrs.verification_status.as_str());
                w.write_optional_element("VerificationToken", attrs.verification_token.as_deref());
                w.raw("</value>");
                w.raw("</entry>");
            }
            w.raw("</VerificationAttributes>");
            w.end_element("GetIdentityVerificationAttributesResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetIdentityVerificationAttributesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::VerifyEmailAddress => {
            let input = VerifyEmailAddressInput {
                email_address: get_required_param(&params, "EmailAddress")?.to_owned(),
            };
            provider.verify_email_address(input)?;
            Ok(empty_xml_response("VerifyEmailAddress", &request_id))
        }

        SesOperation::DeleteVerifiedEmailAddress => {
            let input = DeleteVerifiedEmailAddressInput {
                email_address: get_required_param(&params, "EmailAddress")?.to_owned(),
            };
            provider.delete_verified_email_address(input)?;
            Ok(empty_xml_response(
                "DeleteVerifiedEmailAddress",
                &request_id,
            ))
        }

        SesOperation::ListVerifiedEmailAddresses => {
            let output = provider.list_verified_email_addresses()?;
            let mut w = XmlWriter::new();
            w.start_response("ListVerifiedEmailAddresses");
            w.start_result("ListVerifiedEmailAddresses");
            w.raw("<VerifiedEmailAddresses>");
            for email in &output.verified_email_addresses {
                w.write_element("member", email);
            }
            w.raw("</VerifiedEmailAddresses>");
            w.end_element("ListVerifiedEmailAddressesResult");
            w.write_response_metadata(&request_id);
            w.end_element("ListVerifiedEmailAddressesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::SendEmail => {
            let input = deserialize_send_email(&params)?;
            let output = provider.send_email(input)?;
            let mut w = XmlWriter::new();
            w.start_response("SendEmail");
            w.start_result("SendEmail");
            w.write_element("MessageId", &output.message_id);
            w.end_element("SendEmailResult");
            w.write_response_metadata(&request_id);
            w.end_element("SendEmailResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::SendRawEmail => {
            let input = deserialize_send_raw_email(&params)?;
            let output = provider.send_raw_email(input)?;
            let mut w = XmlWriter::new();
            w.start_response("SendRawEmail");
            w.start_result("SendRawEmail");
            w.write_element("MessageId", &output.message_id);
            w.end_element("SendRawEmailResult");
            w.write_response_metadata(&request_id);
            w.end_element("SendRawEmailResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::GetSendQuota => {
            let output = provider.get_send_quota()?;
            let mut w = XmlWriter::new();
            w.start_response("GetSendQuota");
            w.start_result("GetSendQuota");
            if let Some(v) = output.max24_hour_send {
                w.write_f64_element("Max24HourSend", v);
            }
            if let Some(v) = output.max_send_rate {
                w.write_f64_element("MaxSendRate", v);
            }
            if let Some(v) = output.sent_last24_hours {
                w.write_f64_element("SentLast24Hours", v);
            }
            w.end_element("GetSendQuotaResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetSendQuotaResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::GetSendStatistics => {
            let output = provider.get_send_statistics()?;
            let mut w = XmlWriter::new();
            w.start_response("GetSendStatistics");
            w.start_result("GetSendStatistics");
            w.raw("<SendDataPoints>");
            for dp in &output.send_data_points {
                w.raw("<member>");
                if let Some(v) = dp.delivery_attempts {
                    w.write_i64_element("DeliveryAttempts", v);
                }
                if let Some(v) = dp.bounces {
                    w.write_i64_element("Bounces", v);
                }
                if let Some(v) = dp.complaints {
                    w.write_i64_element("Complaints", v);
                }
                if let Some(v) = dp.rejects {
                    w.write_i64_element("Rejects", v);
                }
                if let Some(ts) = &dp.timestamp {
                    w.write_element("Timestamp", &ts.to_rfc3339());
                }
                w.raw("</member>");
            }
            w.raw("</SendDataPoints>");
            w.end_element("GetSendStatisticsResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetSendStatisticsResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        // ---------------------------------------------------------------
        // Phase 1: Templates + Configuration Sets
        // ---------------------------------------------------------------
        SesOperation::CreateTemplate => {
            let input = deserialize_create_template(&params)?;
            provider.create_template(input)?;
            Ok(empty_xml_response("CreateTemplate", &request_id))
        }

        SesOperation::GetTemplate => {
            let input = GetTemplateInput {
                template_name: get_required_param(&params, "TemplateName")?.to_owned(),
            };
            let output = provider.get_template(input)?;
            let mut w = XmlWriter::new();
            w.start_response("GetTemplate");
            w.start_result("GetTemplate");
            if let Some(t) = &output.template {
                w.raw("<Template>");
                w.write_element("TemplateName", &t.template_name);
                w.write_optional_element("SubjectPart", t.subject_part.as_deref());
                w.write_optional_element("TextPart", t.text_part.as_deref());
                w.write_optional_element("HtmlPart", t.html_part.as_deref());
                w.raw("</Template>");
            }
            w.end_element("GetTemplateResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetTemplateResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::UpdateTemplate => {
            let input = deserialize_update_template(&params)?;
            provider.update_template(input)?;
            Ok(empty_xml_response("UpdateTemplate", &request_id))
        }

        SesOperation::DeleteTemplate => {
            let input = DeleteTemplateInput {
                template_name: get_required_param(&params, "TemplateName")?.to_owned(),
            };
            provider.delete_template(input)?;
            Ok(empty_xml_response("DeleteTemplate", &request_id))
        }

        SesOperation::ListTemplates => {
            let input = ListTemplatesInput {
                max_items: None,
                next_token: None,
            };
            let output = provider.list_templates(input)?;
            let mut w = XmlWriter::new();
            w.start_response("ListTemplates");
            w.start_result("ListTemplates");
            w.raw("<TemplatesMetadata>");
            for meta in &output.templates_metadata {
                w.raw("<member>");
                w.write_optional_element("Name", meta.name.as_deref());
                if let Some(ts) = &meta.created_timestamp {
                    w.write_element("CreatedTimestamp", &ts.to_rfc3339());
                }
                w.raw("</member>");
            }
            w.raw("</TemplatesMetadata>");
            w.write_optional_element("NextToken", output.next_token.as_deref());
            w.end_element("ListTemplatesResult");
            w.write_response_metadata(&request_id);
            w.end_element("ListTemplatesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::SendTemplatedEmail => {
            let input = deserialize_send_templated_email(&params)?;
            let output = provider.send_templated_email(input)?;
            let mut w = XmlWriter::new();
            w.start_response("SendTemplatedEmail");
            w.start_result("SendTemplatedEmail");
            w.write_element("MessageId", &output.message_id);
            w.end_element("SendTemplatedEmailResult");
            w.write_response_metadata(&request_id);
            w.end_element("SendTemplatedEmailResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::CreateConfigurationSet => {
            let name = get_required_param(&params, "ConfigurationSet.Name")?.to_owned();
            let input = CreateConfigurationSetInput {
                configuration_set: ConfigurationSet { name },
            };
            provider.create_configuration_set(input)?;
            Ok(empty_xml_response("CreateConfigurationSet", &request_id))
        }

        SesOperation::DeleteConfigurationSet => {
            let input = DeleteConfigurationSetInput {
                configuration_set_name: get_required_param(&params, "ConfigurationSetName")?
                    .to_owned(),
            };
            provider.delete_configuration_set(input)?;
            Ok(empty_xml_response("DeleteConfigurationSet", &request_id))
        }

        SesOperation::DescribeConfigurationSet => {
            let input = DescribeConfigurationSetInput {
                configuration_set_name: get_required_param(&params, "ConfigurationSetName")?
                    .to_owned(),
                configuration_set_attribute_names: Vec::new(),
            };
            let output = provider.describe_configuration_set(input)?;
            let mut w = XmlWriter::new();
            w.start_response("DescribeConfigurationSet");
            w.start_result("DescribeConfigurationSet");
            if let Some(cs) = &output.configuration_set {
                w.raw("<ConfigurationSet>");
                w.write_element("Name", &cs.name);
                w.raw("</ConfigurationSet>");
            }
            w.raw("<EventDestinations>");
            for dest in &output.event_destinations {
                write_event_destination_xml(&mut w, dest);
            }
            w.raw("</EventDestinations>");
            w.end_element("DescribeConfigurationSetResult");
            w.write_response_metadata(&request_id);
            w.end_element("DescribeConfigurationSetResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::ListConfigurationSets => {
            let input = ListConfigurationSetsInput {
                max_items: None,
                next_token: None,
            };
            let output = provider.list_configuration_sets(input)?;
            let mut w = XmlWriter::new();
            w.start_response("ListConfigurationSets");
            w.start_result("ListConfigurationSets");
            w.raw("<ConfigurationSets>");
            for cs in &output.configuration_sets {
                w.raw("<member>");
                w.write_element("Name", &cs.name);
                w.raw("</member>");
            }
            w.raw("</ConfigurationSets>");
            w.write_optional_element("NextToken", output.next_token.as_deref());
            w.end_element("ListConfigurationSetsResult");
            w.write_response_metadata(&request_id);
            w.end_element("ListConfigurationSetsResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        // ---------------------------------------------------------------
        // Phase 2: Event Destinations + Receipt Rules
        // ---------------------------------------------------------------
        SesOperation::CreateConfigurationSetEventDestination => {
            let input = deserialize_create_config_set_event_dest(&params)?;
            provider.create_configuration_set_event_destination(input)?;
            Ok(empty_xml_response(
                "CreateConfigurationSetEventDestination",
                &request_id,
            ))
        }

        SesOperation::UpdateConfigurationSetEventDestination => {
            let input = deserialize_update_config_set_event_dest(&params)?;
            provider.update_configuration_set_event_destination(input)?;
            Ok(empty_xml_response(
                "UpdateConfigurationSetEventDestination",
                &request_id,
            ))
        }

        SesOperation::DeleteConfigurationSetEventDestination => {
            let input = DeleteConfigurationSetEventDestinationInput {
                configuration_set_name: get_required_param(&params, "ConfigurationSetName")?
                    .to_owned(),
                event_destination_name: get_required_param(&params, "EventDestinationName")?
                    .to_owned(),
            };
            provider.delete_configuration_set_event_destination(input)?;
            Ok(empty_xml_response(
                "DeleteConfigurationSetEventDestination",
                &request_id,
            ))
        }

        SesOperation::CreateReceiptRuleSet => {
            let input = CreateReceiptRuleSetInput {
                rule_set_name: get_required_param(&params, "RuleSetName")?.to_owned(),
            };
            provider.create_receipt_rule_set(input)?;
            Ok(empty_xml_response("CreateReceiptRuleSet", &request_id))
        }

        SesOperation::DeleteReceiptRuleSet => {
            let input = DeleteReceiptRuleSetInput {
                rule_set_name: get_required_param(&params, "RuleSetName")?.to_owned(),
            };
            provider.delete_receipt_rule_set(input)?;
            Ok(empty_xml_response("DeleteReceiptRuleSet", &request_id))
        }

        SesOperation::CreateReceiptRule => {
            let input = deserialize_create_receipt_rule(&params)?;
            provider.create_receipt_rule(input)?;
            Ok(empty_xml_response("CreateReceiptRule", &request_id))
        }

        SesOperation::DeleteReceiptRule => {
            let input = DeleteReceiptRuleInput {
                rule_set_name: get_required_param(&params, "RuleSetName")?.to_owned(),
                rule_name: get_required_param(&params, "RuleName")?.to_owned(),
            };
            provider.delete_receipt_rule(input)?;
            Ok(empty_xml_response("DeleteReceiptRule", &request_id))
        }

        SesOperation::DescribeReceiptRuleSet => {
            let input = DescribeReceiptRuleSetInput {
                rule_set_name: get_required_param(&params, "RuleSetName")?.to_owned(),
            };
            let output = provider.describe_receipt_rule_set(input)?;
            let mut w = XmlWriter::new();
            w.start_response("DescribeReceiptRuleSet");
            w.start_result("DescribeReceiptRuleSet");
            if let Some(meta) = &output.metadata {
                w.raw("<Metadata>");
                w.write_optional_element("Name", meta.name.as_deref());
                if let Some(ts) = &meta.created_timestamp {
                    w.write_element("CreatedTimestamp", &ts.to_rfc3339());
                }
                w.raw("</Metadata>");
            }
            w.raw("<Rules>");
            for rule in &output.rules {
                write_receipt_rule_xml(&mut w, rule);
            }
            w.raw("</Rules>");
            w.end_element("DescribeReceiptRuleSetResult");
            w.write_response_metadata(&request_id);
            w.end_element("DescribeReceiptRuleSetResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::CloneReceiptRuleSet => {
            let input = CloneReceiptRuleSetInput {
                original_rule_set_name: get_required_param(&params, "OriginalRuleSetName")?
                    .to_owned(),
                rule_set_name: get_required_param(&params, "RuleSetName")?.to_owned(),
            };
            provider.clone_receipt_rule_set(input)?;
            Ok(empty_xml_response("CloneReceiptRuleSet", &request_id))
        }

        SesOperation::DescribeActiveReceiptRuleSet => {
            let output =
                provider.describe_active_receipt_rule_set(DescribeActiveReceiptRuleSetInput {})?;
            let mut w = XmlWriter::new();
            w.start_response("DescribeActiveReceiptRuleSet");
            w.start_result("DescribeActiveReceiptRuleSet");
            if let Some(meta) = &output.metadata {
                w.raw("<Metadata>");
                w.write_optional_element("Name", meta.name.as_deref());
                if let Some(ts) = &meta.created_timestamp {
                    w.write_element("CreatedTimestamp", &ts.to_rfc3339());
                }
                w.raw("</Metadata>");
            }
            w.raw("<Rules>");
            for rule in &output.rules {
                write_receipt_rule_xml(&mut w, rule);
            }
            w.raw("</Rules>");
            w.end_element("DescribeActiveReceiptRuleSetResult");
            w.write_response_metadata(&request_id);
            w.end_element("DescribeActiveReceiptRuleSetResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::SetActiveReceiptRuleSet => {
            let input = SetActiveReceiptRuleSetInput {
                rule_set_name: get_optional_param(&params, "RuleSetName").map(str::to_owned),
            };
            provider.set_active_receipt_rule_set(input)?;
            Ok(empty_xml_response("SetActiveReceiptRuleSet", &request_id))
        }

        // ---------------------------------------------------------------
        // Phase 3: Identity Configuration + Sending Authorization
        // ---------------------------------------------------------------
        SesOperation::SetIdentityNotificationTopic => {
            let notification_type_str = get_required_param(&params, "NotificationType")?;
            let notification_type = NotificationType::from(notification_type_str);
            let input = SetIdentityNotificationTopicInput {
                identity: get_required_param(&params, "Identity")?.to_owned(),
                notification_type,
                sns_topic: get_optional_param(&params, "SnsTopic").map(str::to_owned),
            };
            provider.set_identity_notification_topic(input)?;
            Ok(empty_xml_response(
                "SetIdentityNotificationTopic",
                &request_id,
            ))
        }

        SesOperation::SetIdentityFeedbackForwardingEnabled => {
            let input = SetIdentityFeedbackForwardingEnabledInput {
                identity: get_required_param(&params, "Identity")?.to_owned(),
                forwarding_enabled: get_optional_bool(&params, "ForwardingEnabled").unwrap_or(true),
            };
            provider.set_identity_feedback_forwarding_enabled(input)?;
            Ok(empty_xml_response(
                "SetIdentityFeedbackForwardingEnabled",
                &request_id,
            ))
        }

        SesOperation::GetIdentityNotificationAttributes => {
            let identities = parse_member_list(&params, "Identities");
            let input = GetIdentityNotificationAttributesInput { identities };
            let output = provider.get_identity_notification_attributes(input)?;
            let mut w = XmlWriter::new();
            w.start_response("GetIdentityNotificationAttributes");
            w.start_result("GetIdentityNotificationAttributes");
            w.raw("<NotificationAttributes>");
            for (identity, attrs) in &output.notification_attributes {
                w.raw("<entry>");
                w.write_element("key", identity);
                w.raw("<value>");
                w.write_element("BounceTopic", &attrs.bounce_topic);
                w.write_element("ComplaintTopic", &attrs.complaint_topic);
                w.write_element("DeliveryTopic", &attrs.delivery_topic);
                w.write_bool_element("ForwardingEnabled", attrs.forwarding_enabled);
                w.write_bool_element(
                    "HeadersInBounceNotificationsEnabled",
                    attrs
                        .headers_in_bounce_notifications_enabled
                        .unwrap_or(false),
                );
                w.write_bool_element(
                    "HeadersInComplaintNotificationsEnabled",
                    attrs
                        .headers_in_complaint_notifications_enabled
                        .unwrap_or(false),
                );
                w.write_bool_element(
                    "HeadersInDeliveryNotificationsEnabled",
                    attrs
                        .headers_in_delivery_notifications_enabled
                        .unwrap_or(false),
                );
                w.raw("</value>");
                w.raw("</entry>");
            }
            w.raw("</NotificationAttributes>");
            w.end_element("GetIdentityNotificationAttributesResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetIdentityNotificationAttributesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::VerifyDomainDkim => {
            let input = VerifyDomainDkimInput {
                domain: get_required_param(&params, "Domain")?.to_owned(),
            };
            let output = provider.verify_domain_dkim(input)?;
            let mut w = XmlWriter::new();
            w.start_response("VerifyDomainDkim");
            w.start_result("VerifyDomainDkim");
            w.raw("<DkimTokens>");
            for token in &output.dkim_tokens {
                w.write_element("member", token);
            }
            w.raw("</DkimTokens>");
            w.end_element("VerifyDomainDkimResult");
            w.write_response_metadata(&request_id);
            w.end_element("VerifyDomainDkimResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::GetIdentityDkimAttributes => {
            let identities = parse_member_list(&params, "Identities");
            let input = GetIdentityDkimAttributesInput { identities };
            let output = provider.get_identity_dkim_attributes(input)?;
            let mut w = XmlWriter::new();
            w.start_response("GetIdentityDkimAttributes");
            w.start_result("GetIdentityDkimAttributes");
            w.raw("<DkimAttributes>");
            for (identity, attrs) in &output.dkim_attributes {
                w.raw("<entry>");
                w.write_element("key", identity);
                w.raw("<value>");
                w.write_bool_element("DkimEnabled", attrs.dkim_enabled);
                w.write_element(
                    "DkimVerificationStatus",
                    attrs.dkim_verification_status.as_str(),
                );
                w.raw("<DkimTokens>");
                for token in &attrs.dkim_tokens {
                    w.write_element("member", token);
                }
                w.raw("</DkimTokens>");
                w.raw("</value>");
                w.raw("</entry>");
            }
            w.raw("</DkimAttributes>");
            w.end_element("GetIdentityDkimAttributesResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetIdentityDkimAttributesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::SetIdentityMailFromDomain => {
            let behavior =
                get_optional_param(&params, "BehaviorOnMXFailure").map(BehaviorOnMXFailure::from);
            let input = SetIdentityMailFromDomainInput {
                identity: get_required_param(&params, "Identity")?.to_owned(),
                mail_from_domain: get_optional_param(&params, "MailFromDomain").map(str::to_owned),
                behavior_on_mx_failure: behavior,
            };
            provider.set_identity_mail_from_domain(input)?;
            Ok(empty_xml_response("SetIdentityMailFromDomain", &request_id))
        }

        SesOperation::GetIdentityMailFromDomainAttributes => {
            let identities = parse_member_list(&params, "Identities");
            let input = GetIdentityMailFromDomainAttributesInput { identities };
            let output = provider.get_identity_mail_from_domain_attributes(input)?;
            let mut w = XmlWriter::new();
            w.start_response("GetIdentityMailFromDomainAttributes");
            w.start_result("GetIdentityMailFromDomainAttributes");
            w.raw("<MailFromDomainAttributes>");
            for (identity, attrs) in &output.mail_from_domain_attributes {
                w.raw("<entry>");
                w.write_element("key", identity);
                w.raw("<value>");
                w.write_element("MailFromDomain", &attrs.mail_from_domain);
                w.write_element(
                    "MailFromDomainStatus",
                    attrs.mail_from_domain_status.as_str(),
                );
                w.write_element("BehaviorOnMXFailure", attrs.behavior_on_mx_failure.as_str());
                w.raw("</value>");
                w.raw("</entry>");
            }
            w.raw("</MailFromDomainAttributes>");
            w.end_element("GetIdentityMailFromDomainAttributesResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetIdentityMailFromDomainAttributesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::GetIdentityPolicies => {
            let identity = get_required_param(&params, "Identity")?.to_owned();
            let policy_names = parse_member_list(&params, "PolicyNames");
            let input = GetIdentityPoliciesInput {
                identity,
                policy_names,
            };
            let output = provider.get_identity_policies(input)?;
            let mut w = XmlWriter::new();
            w.start_response("GetIdentityPolicies");
            w.start_result("GetIdentityPolicies");
            w.raw("<Policies>");
            for (name, policy) in &output.policies {
                w.raw("<entry>");
                w.write_element("key", name);
                w.write_element("value", policy);
                w.raw("</entry>");
            }
            w.raw("</Policies>");
            w.end_element("GetIdentityPoliciesResult");
            w.write_response_metadata(&request_id);
            w.end_element("GetIdentityPoliciesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }

        SesOperation::PutIdentityPolicy => {
            let input = PutIdentityPolicyInput {
                identity: get_required_param(&params, "Identity")?.to_owned(),
                policy_name: get_required_param(&params, "PolicyName")?.to_owned(),
                policy: get_required_param(&params, "Policy")?.to_owned(),
            };
            provider.put_identity_policy(input)?;
            Ok(empty_xml_response("PutIdentityPolicy", &request_id))
        }

        SesOperation::DeleteIdentityPolicy => {
            let input = DeleteIdentityPolicyInput {
                identity: get_required_param(&params, "Identity")?.to_owned(),
                policy_name: get_required_param(&params, "PolicyName")?.to_owned(),
            };
            provider.delete_identity_policy(input)?;
            Ok(empty_xml_response("DeleteIdentityPolicy", &request_id))
        }

        SesOperation::ListIdentityPolicies => {
            let input = ListIdentityPoliciesInput {
                identity: get_required_param(&params, "Identity")?.to_owned(),
            };
            let output = provider.list_identity_policies(input)?;
            let mut w = XmlWriter::new();
            w.start_response("ListIdentityPolicies");
            w.start_result("ListIdentityPolicies");
            w.raw("<PolicyNames>");
            for name in &output.policy_names {
                w.write_element("member", name);
            }
            w.raw("</PolicyNames>");
            w.end_element("ListIdentityPoliciesResult");
            w.write_response_metadata(&request_id);
            w.end_element("ListIdentityPoliciesResponse");
            Ok(xml_response(w.into_string(), &request_id))
        }
    }
}

// ---------------------------------------------------------------
// Helper: empty XML response
// ---------------------------------------------------------------

fn empty_xml_response(operation: &str, request_id: &str) -> http::Response<SesResponseBody> {
    let mut w = XmlWriter::new();
    w.start_response(operation);
    w.start_result(operation);
    w.end_element(&format!("{operation}Result"));
    w.write_response_metadata(request_id);
    w.end_element(&format!("{operation}Response"));
    xml_response(w.into_string(), request_id)
}

// ---------------------------------------------------------------
// Deserialization helpers for complex input types
// ---------------------------------------------------------------

fn deserialize_send_email(params: &[(String, String)]) -> Result<SendEmailInput, SesError> {
    let source = get_required_param(params, "Source")?.to_owned();
    let to = parse_member_list(params, "Destination.ToAddresses");
    let cc = parse_member_list(params, "Destination.CcAddresses");
    let bcc = parse_member_list(params, "Destination.BccAddresses");
    let subject_data = get_optional_param(params, "Message.Subject.Data")
        .unwrap_or("")
        .to_owned();
    let subject_charset = get_optional_param(params, "Message.Subject.Charset").map(str::to_owned);
    let text_data = get_optional_param(params, "Message.Body.Text.Data").map(str::to_owned);
    let html_data = get_optional_param(params, "Message.Body.Html.Data").map(str::to_owned);

    let tags = parse_tag_list(params, "Tags");

    Ok(SendEmailInput {
        source,
        destination: Destination {
            to_addresses: to,
            cc_addresses: cc,
            bcc_addresses: bcc,
        },
        message: Message {
            subject: Content {
                data: subject_data,
                charset: subject_charset,
            },
            body: Body {
                text: text_data.map(|d| Content {
                    data: d,
                    charset: get_optional_param(params, "Message.Body.Text.Charset")
                        .map(str::to_owned),
                }),
                html: html_data.map(|d| Content {
                    data: d,
                    charset: get_optional_param(params, "Message.Body.Html.Charset")
                        .map(str::to_owned),
                }),
            },
        },
        tags: tags
            .into_iter()
            .map(|(n, v)| MessageTag { name: n, value: v })
            .collect(),
        configuration_set_name: get_optional_param(params, "ConfigurationSetName")
            .map(str::to_owned),
        return_path: get_optional_param(params, "ReturnPath").map(str::to_owned),
        return_path_arn: get_optional_param(params, "ReturnPathArn").map(str::to_owned),
        source_arn: get_optional_param(params, "SourceArn").map(str::to_owned),
        reply_to_addresses: parse_member_list(params, "ReplyToAddresses"),
    })
}

fn deserialize_send_raw_email(params: &[(String, String)]) -> Result<SendRawEmailInput, SesError> {
    let raw_data = get_required_param(params, "RawMessage.Data")?.to_owned();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(raw_data.as_bytes())
        .map_err(|e| {
            SesError::invalid_parameter_value(format!("RawMessage.Data is not valid base64: {e}"))
        })?;
    let tags = parse_tag_list(params, "Tags");
    Ok(SendRawEmailInput {
        raw_message: RawMessage { data: decoded },
        source: get_optional_param(params, "Source").map(str::to_owned),
        destinations: parse_member_list(params, "Destinations"),
        tags: tags
            .into_iter()
            .map(|(n, v)| MessageTag { name: n, value: v })
            .collect(),
        configuration_set_name: get_optional_param(params, "ConfigurationSetName")
            .map(str::to_owned),
        from_arn: get_optional_param(params, "FromArn").map(str::to_owned),
        return_path_arn: get_optional_param(params, "ReturnPathArn").map(str::to_owned),
        source_arn: get_optional_param(params, "SourceArn").map(str::to_owned),
    })
}

fn deserialize_send_templated_email(
    params: &[(String, String)],
) -> Result<SendTemplatedEmailInput, SesError> {
    let source = get_required_param(params, "Source")?.to_owned();
    let to = parse_member_list(params, "Destination.ToAddresses");
    let cc = parse_member_list(params, "Destination.CcAddresses");
    let bcc = parse_member_list(params, "Destination.BccAddresses");
    let template = get_required_param(params, "Template")?.to_owned();
    let template_data = get_required_param(params, "TemplateData")?.to_owned();
    let tags = parse_tag_list(params, "Tags");

    Ok(SendTemplatedEmailInput {
        source,
        destination: Destination {
            to_addresses: to,
            cc_addresses: cc,
            bcc_addresses: bcc,
        },
        template,
        template_data,
        tags: tags
            .into_iter()
            .map(|(n, v)| MessageTag { name: n, value: v })
            .collect(),
        configuration_set_name: get_optional_param(params, "ConfigurationSetName")
            .map(str::to_owned),
        return_path: get_optional_param(params, "ReturnPath").map(str::to_owned),
        return_path_arn: get_optional_param(params, "ReturnPathArn").map(str::to_owned),
        source_arn: get_optional_param(params, "SourceArn").map(str::to_owned),
        template_arn: get_optional_param(params, "TemplateArn").map(str::to_owned),
        reply_to_addresses: parse_member_list(params, "ReplyToAddresses"),
    })
}

fn deserialize_create_template(
    params: &[(String, String)],
) -> Result<CreateTemplateInput, SesError> {
    Ok(CreateTemplateInput {
        template: Template {
            template_name: get_required_param(params, "Template.TemplateName")?.to_owned(),
            subject_part: get_optional_param(params, "Template.SubjectPart").map(str::to_owned),
            text_part: get_optional_param(params, "Template.TextPart").map(str::to_owned),
            html_part: get_optional_param(params, "Template.HtmlPart").map(str::to_owned),
        },
    })
}

fn deserialize_update_template(
    params: &[(String, String)],
) -> Result<UpdateTemplateInput, SesError> {
    Ok(UpdateTemplateInput {
        template: Template {
            template_name: get_required_param(params, "Template.TemplateName")?.to_owned(),
            subject_part: get_optional_param(params, "Template.SubjectPart").map(str::to_owned),
            text_part: get_optional_param(params, "Template.TextPart").map(str::to_owned),
            html_part: get_optional_param(params, "Template.HtmlPart").map(str::to_owned),
        },
    })
}

fn deserialize_create_config_set_event_dest(
    params: &[(String, String)],
) -> Result<CreateConfigurationSetEventDestinationInput, SesError> {
    let config_set_name = get_required_param(params, "ConfigurationSetName")?.to_owned();
    let dest_name = get_required_param(params, "EventDestination.Name")?.to_owned();
    let enabled = get_optional_bool(params, "EventDestination.Enabled");
    let matching_types = parse_member_list(params, "EventDestination.MatchingEventTypes");

    Ok(CreateConfigurationSetEventDestinationInput {
        configuration_set_name: config_set_name,
        event_destination: EventDestination {
            name: dest_name,
            enabled,
            matching_event_types: matching_types
                .into_iter()
                .map(|s| s.as_str().into())
                .collect(),
            ..EventDestination::default()
        },
    })
}

fn deserialize_update_config_set_event_dest(
    params: &[(String, String)],
) -> Result<UpdateConfigurationSetEventDestinationInput, SesError> {
    let config_set_name = get_required_param(params, "ConfigurationSetName")?.to_owned();
    let dest_name = get_required_param(params, "EventDestination.Name")?.to_owned();
    let enabled = get_optional_bool(params, "EventDestination.Enabled");
    let matching_types = parse_member_list(params, "EventDestination.MatchingEventTypes");

    Ok(UpdateConfigurationSetEventDestinationInput {
        configuration_set_name: config_set_name,
        event_destination: EventDestination {
            name: dest_name,
            enabled,
            matching_event_types: matching_types
                .into_iter()
                .map(|s| s.as_str().into())
                .collect(),
            ..EventDestination::default()
        },
    })
}

fn deserialize_create_receipt_rule(
    params: &[(String, String)],
) -> Result<CreateReceiptRuleInput, SesError> {
    let rule_set_name = get_required_param(params, "RuleSetName")?.to_owned();
    let after = get_optional_param(params, "After").map(str::to_owned);
    let rule_name = get_required_param(params, "Rule.Name")?.to_owned();
    let enabled = get_optional_bool(params, "Rule.Enabled");
    let scan_enabled = get_optional_bool(params, "Rule.ScanEnabled");
    let recipients = parse_member_list(params, "Rule.Recipients");

    Ok(CreateReceiptRuleInput {
        rule_set_name,
        after,
        rule: ReceiptRule {
            name: rule_name,
            enabled,
            scan_enabled,
            recipients,
            actions: Vec::new(),
            tls_policy: None,
        },
    })
}

// ---------------------------------------------------------------
// XML serialization helpers
// ---------------------------------------------------------------

fn write_event_destination_xml(w: &mut XmlWriter, dest: &EventDestination) {
    w.raw("<member>");
    w.write_element("Name", &dest.name);
    if let Some(enabled) = dest.enabled {
        w.write_bool_element("Enabled", enabled);
    }
    w.raw("<MatchingEventTypes>");
    for et in &dest.matching_event_types {
        w.write_element("member", et.as_str());
    }
    w.raw("</MatchingEventTypes>");
    w.raw("</member>");
}

fn write_receipt_rule_xml(w: &mut XmlWriter, rule: &ReceiptRule) {
    w.raw("<member>");
    w.write_element("Name", &rule.name);
    if let Some(enabled) = rule.enabled {
        w.write_bool_element("Enabled", enabled);
    }
    if let Some(scan) = rule.scan_enabled {
        w.write_bool_element("ScanEnabled", scan);
    }
    w.raw("<Recipients>");
    for r in &rule.recipients {
        w.write_element("member", r);
    }
    w.raw("</Recipients>");
    w.raw("<Actions>");
    for _action in &rule.actions {
        w.raw("<member/>");
    }
    w.raw("</Actions>");
    w.raw("</member>");
}
