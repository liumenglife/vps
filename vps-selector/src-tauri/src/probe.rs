use std::net::{IpAddr, SocketAddr, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::error::AppError;
use crate::models::TcpProbeResult;

pub fn parse_ping_avg_latency(output: &str) -> Option<f64> {
    output.lines().find_map(|line| {
        let (_, values) = line.split_once("=")?;
        let mut parts = values.trim().split('/');
        parts.next()?;
        parts.next()?.trim().parse::<f64>().ok()
    })
}

pub fn parse_ping_packet_loss(output: &str) -> Option<f64> {
    output.lines().find_map(|line| {
        let packet_loss = line.find("% packet loss")?;
        let before_percent = &line[..packet_loss];
        let value = before_percent.rsplit_once(' ').map_or(before_percent, |(_, value)| value);
        value.trim().parse::<f64>().ok().map(|percent| percent / 100.0)
    })
}

pub fn parse_traceroute_hops(output: &str) -> Option<u32> {
    let hops = output
        .lines()
        .filter(|line| {
            line.trim_start()
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_digit())
        })
        .count() as u32;

    (hops > 0).then_some(hops)
}

pub fn tcp_connect(ip: &str, port: u16, timeout_ms: u64) -> TcpProbeResult {
    let parsed_ip = match ip.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(error) => {
            return TcpProbeResult {
                port,
                success: false,
                latency_ms: None,
                error: Some(format!("IP 格式错误：{error}")),
            };
        }
    };

    let address = SocketAddr::new(parsed_ip, port);
    let start = Instant::now();
    match TcpStream::connect_timeout(&address, Duration::from_millis(timeout_ms)) {
        Ok(_) => TcpProbeResult {
            port,
            success: true,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            error: None,
        },
        Err(error) => TcpProbeResult {
            port,
            success: false,
            latency_ms: None,
            error: Some(error.to_string()),
        },
    }
}

pub fn run_ping_once(
    ip: &str,
    timeout_ms: u64,
) -> Result<(Option<f64>, bool, Vec<String>), AppError> {
    let output = Command::new("ping")
        .args(["-c", "1", "-W", &timeout_ms.to_string(), ip])
        .output()?;
    let text = command_text(&output);
    let latency = parse_ping_avg_latency(&text);
    let packet_loss = parse_ping_packet_loss(&text);
    let success = matches!(packet_loss, Some(loss) if loss == 0.0) && latency.is_some();
    let errors = command_errors(&output, "ping");

    Ok((latency, success, errors))
}

pub fn run_traceroute_once(ip: &str) -> Result<(Option<u32>, Vec<String>), AppError> {
    let output = Command::new("traceroute").arg(ip).output()?;
    let text = command_text(&output);
    let hops = parse_traceroute_hops(&text);
    let errors = command_errors(&output, "traceroute");

    Ok((hops, errors))
}

fn command_text(output: &std::process::Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text
}

fn command_errors(output: &std::process::Output, command: &str) -> Vec<String> {
    if output.status.success() {
        return Vec::new();
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        vec![format!("{command} 命令执行失败")]
    } else {
        vec![format!("{command} 命令执行失败：{stderr}")]
    }
}

#[cfg(test)]
mod tests {
    use super::command_errors;
    use std::process::{ExitStatus, Output};

    #[cfg(unix)]
    fn exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;

        ExitStatus::from_raw(code << 8)
    }

    #[test]
    fn command_errors_prefixes_stderr_with_chinese_context() {
        let output = Output {
            status: exit_status(1),
            stdout: Vec::new(),
            stderr: b"unknown host".to_vec(),
        };

        assert_eq!(
            command_errors(&output, "ping"),
            vec!["ping 命令执行失败：unknown host".to_string()]
        );
    }
}
