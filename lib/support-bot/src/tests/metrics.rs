use super::*;
use prometheus_client::encoding::text::encode;

#[test]
fn encodes_separate_bot_labels() {
    let mut registry = Registry::default();
    let metrics = SupportBotMetrics::register(&mut registry);

    metrics
        .for_bot("support_a")
        .record_acp_request("success", Duration::from_millis(25));
    metrics
        .for_bot("support_b")
        .record_acp_request("error", Duration::from_millis(50));

    let mut output = String::new();
    encode(&mut output, &registry).unwrap();

    assert!(
        output.contains("support_bot_acp_requests_total{bot=\"support_a\",outcome=\"success\"} 1")
    );
    assert!(
        output.contains("support_bot_acp_requests_total{bot=\"support_b\",outcome=\"error\"} 1")
    );
}
