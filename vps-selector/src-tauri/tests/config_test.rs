use vps_selector::config::parse_config;

#[test]
fn parses_chinese_toml_config() {
    let input = include_str!("fixtures/valid-config.toml");
    let config = parse_config(input).unwrap();
    assert_eq!(config.targets[0].ip, "192.3.81.8");
    assert_eq!(config.targets[0].city, "纽约");
    assert_eq!(config.targets[1].ip, "107.174.51.158");
    assert_eq!(config.targets[1].city, "洛杉矶 DCO3");
    assert_eq!(config.targets[2].ip, "198.23.228.15");
    assert_eq!(config.targets[2].city, "伊利诺伊州芝加哥");
    assert!((config.weights.stability - 0.5).abs() < 0.0001);
    assert!((config.stability_weights.consecutive_failure - 0.45).abs() < 0.0001);
    assert!((config.stability_weights.packet_loss - 0.40).abs() < 0.0001);
    assert!((config.stability_weights.jitter - 0.15).abs() < 0.0001);
}

#[test]
fn rejects_invalid_weight_sum() {
    let input = include_str!("fixtures/invalid-weight.toml");
    let error = parse_config(input).unwrap_err().to_string();
    assert!(error.contains("总权重"));
}
