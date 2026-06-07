use vps_selector::config::parse_config;

#[test]
fn parses_chinese_toml_config() {
    let input = include_str!("fixtures/valid-config.toml");
    let config = parse_config(input).unwrap();
    assert_eq!(config.targets[0].ip, "203.0.113.10");
    assert_eq!(config.targets[0].city, "东京");
    assert!((config.weights.stability - 0.5).abs() < 0.0001);
}

#[test]
fn rejects_invalid_weight_sum() {
    let input = include_str!("fixtures/invalid-weight.toml");
    let error = parse_config(input).unwrap_err().to_string();
    assert!(error.contains("总权重"));
}
