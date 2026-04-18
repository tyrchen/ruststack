//! Integration tests for the CloudFront service.
//!
//! These tests require a running Rustack server at `localhost:4566`.
//! They are marked `#[ignore = "requires running Rustack server"]` so they
//! don't run during normal `cargo test`.

#![allow(unused_imports)]

use aws_sdk_cloudfront::types::{
    AllowedMethods, CachedMethods, DefaultCacheBehavior, DistributionConfig, GeoRestriction,
    GeoRestrictionType, HttpVersion, Method as CfMethod, Origin, OriginAccessControlConfig,
    OriginAccessControlOriginTypes, OriginAccessControlSigningBehaviors,
    OriginAccessControlSigningProtocols, Origins, PriceClass, Restrictions, S3OriginConfig,
    ViewerCertificate, ViewerProtocolPolicy,
};

use crate::cloudfront_client;

#[allow(dead_code)]
fn build_min_distribution_config(caller_ref: &str, bucket_domain: &str) -> DistributionConfig {
    let origin = Origin::builder()
        .id("primary")
        .domain_name(bucket_domain)
        .s3_origin_config(S3OriginConfig::builder().origin_access_identity("").build())
        .build()
        .expect("build Origin");
    let default_cb = DefaultCacheBehavior::builder()
        .target_origin_id("primary")
        .viewer_protocol_policy(ViewerProtocolPolicy::AllowAll)
        .allowed_methods(
            AllowedMethods::builder()
                .quantity(2)
                .items(CfMethod::Get)
                .items(CfMethod::Head)
                .cached_methods(
                    CachedMethods::builder()
                        .quantity(2)
                        .items(CfMethod::Get)
                        .items(CfMethod::Head)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        )
        .cache_policy_id("658327ea-f89d-4fab-a63d-7e88639e58f6")
        .compress(false)
        .build()
        .expect("build DefaultCacheBehavior");

    DistributionConfig::builder()
        .caller_reference(caller_ref)
        .comment("test distribution")
        .enabled(false)
        .default_root_object("index.html")
        .price_class(PriceClass::PriceClassAll)
        .http_version(HttpVersion::Http2)
        .is_ipv6_enabled(false)
        .origins(
            Origins::builder()
                .quantity(1)
                .items(origin)
                .build()
                .unwrap(),
        )
        .default_cache_behavior(default_cb)
        .viewer_certificate(
            ViewerCertificate::builder()
                .cloud_front_default_certificate(true)
                .build(),
        )
        .restrictions(
            Restrictions::builder()
                .geo_restriction(
                    GeoRestriction::builder()
                        .quantity(0)
                        .restriction_type(GeoRestrictionType::None)
                        .build()
                        .unwrap(),
                )
                .build(),
        )
        .build()
        .expect("build DistributionConfig")
}

#[tokio::test]
#[ignore = "requires running Rustack server"]
async fn test_should_create_get_and_delete_distribution() {
    let client = cloudfront_client();
    let caller_ref = format!("test-{}", uuid::Uuid::new_v4());
    let cfg = build_min_distribution_config(&caller_ref, "test-bucket.s3.us-east-1.amazonaws.com");

    let created = client
        .create_distribution()
        .distribution_config(cfg)
        .send()
        .await
        .expect("create distribution");
    let dist = created.distribution().expect("distribution in response");
    let id = dist.id().to_owned();
    assert!(
        id.starts_with('E'),
        "distribution id should start with E: {id}"
    );
    assert!(!dist.domain_name().is_empty());

    let got = client
        .get_distribution()
        .id(&id)
        .send()
        .await
        .expect("get distribution");
    assert_eq!(got.distribution().unwrap().id(), id);

    let etag = got.e_tag().expect("etag").to_owned();
    client
        .delete_distribution()
        .id(&id)
        .if_match(&etag)
        .send()
        .await
        .expect("delete disabled distribution");
}

#[tokio::test]
#[ignore = "requires running Rustack server"]
async fn test_should_enforce_etag_if_match() {
    let client = cloudfront_client();
    let caller_ref = format!("test-{}", uuid::Uuid::new_v4());
    let cfg = build_min_distribution_config(&caller_ref, "test-bucket.s3.us-east-1.amazonaws.com");

    let created = client
        .create_distribution()
        .distribution_config(cfg)
        .send()
        .await
        .expect("create");
    let id = created.distribution().unwrap().id().to_owned();

    // Delete without If-Match should fail.
    let err = client
        .delete_distribution()
        .id(&id)
        .send()
        .await
        .err()
        .expect("expected error");
    let msg = err.to_string();
    assert!(
        msg.contains("If-Match") || msg.contains("Precondition") || msg.contains("Missing"),
        "unexpected error: {msg}"
    );

    let got = client.get_distribution().id(&id).send().await.unwrap();
    let etag = got.e_tag().unwrap().to_owned();
    client
        .delete_distribution()
        .id(&id)
        .if_match(&etag)
        .send()
        .await
        .expect("delete");
}

#[tokio::test]
#[ignore = "requires running Rustack server"]
async fn test_should_create_origin_access_control() {
    let client = cloudfront_client();
    let name = format!("oac-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let cfg = OriginAccessControlConfig::builder()
        .name(&name)
        .description("test OAC")
        .signing_protocol(OriginAccessControlSigningProtocols::Sigv4)
        .signing_behavior(OriginAccessControlSigningBehaviors::Always)
        .origin_access_control_origin_type(OriginAccessControlOriginTypes::S3)
        .build()
        .unwrap();

    let created = client
        .create_origin_access_control()
        .origin_access_control_config(cfg)
        .send()
        .await
        .expect("create oac");
    let oac = created.origin_access_control().unwrap();
    assert!(oac.id().starts_with('E'));
    let id = oac.id().to_owned();

    let got = client
        .get_origin_access_control()
        .id(&id)
        .send()
        .await
        .expect("get oac");
    let etag = got.e_tag().unwrap().to_owned();
    client
        .delete_origin_access_control()
        .id(&id)
        .if_match(&etag)
        .send()
        .await
        .expect("delete oac");
}

#[tokio::test]
#[ignore = "requires running Rustack server"]
async fn test_should_list_managed_cache_policies() {
    let client = cloudfront_client();
    let resp = client
        .list_cache_policies()
        .send()
        .await
        .expect("list cache policies");
    let list = resp.cache_policy_list().expect("policy list");
    let items = list.items();
    assert!(
        items.iter().any(|p| p
            .cache_policy()
            .is_some_and(|cp| cp.id() == "658327ea-f89d-4fab-a63d-7e88639e58f6")),
        "managed CachingOptimized policy should be present"
    );
}
