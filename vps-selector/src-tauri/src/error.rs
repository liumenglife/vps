use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置错误：{0}")]
    Config(String),
    #[error("探测错误：{0}")]
    Probe(String),
    #[error("报告错误：{0}")]
    Report(String),
    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
}
