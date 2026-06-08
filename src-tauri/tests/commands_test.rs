use vps_selector::commands::validate_config_text;

#[test]
fn validate_config_text_reports_candidate_count() {
    let message =
        validate_config_text(include_str!("fixtures/valid-config.toml").to_string()).unwrap();

    assert_eq!(message, "配置校验通过：3 个候选 IP");
}

#[test]
fn validate_config_text_returns_chinese_error() {
    let error =
        validate_config_text(include_str!("fixtures/invalid-weight.toml").to_string()).unwrap_err();

    assert!(error.contains("配置错误"));
    assert!(error.contains("总权重"));
}
