use vps_selector::probe::{parse_ping_avg_latency, parse_ping_packet_loss, parse_traceroute_hops};

#[test]
fn parses_macos_ping_latency() {
    let output = "round-trip min/avg/max/stddev = 10.234/12.345/15.678/2.345 ms";

    assert_eq!(parse_ping_avg_latency(output), Some(12.345));
}

#[test]
fn parses_packet_loss() {
    assert_eq!(
        parse_ping_packet_loss("5 packets transmitted, 5 packets received, 0.0% packet loss"),
        Some(0.0)
    );
    assert_eq!(
        parse_ping_packet_loss("5 packets transmitted, 0 packets received, 100.0% packet loss"),
        Some(1.0)
    );
}

#[test]
fn parses_traceroute_hops() {
    let output = "traceroute to 203.0.113.10\n 1  192.168.1.1  1.2 ms\n 2  10.0.0.1  3.4 ms\n 3  203.0.113.10  20.1 ms";

    assert_eq!(parse_traceroute_hops(output), Some(3));
}
