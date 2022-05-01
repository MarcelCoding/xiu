use std::fs;
use std::vec::Vec;

use log::{Level, LevelFilter};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct XiuConfig {
  pub rtmp: Option<RtmpConfig>,
  pub httpflv: Option<HttpFlvConfig>,
  pub hls: Option<HlsConfig>,
  pub log: Option<LogConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RtmpConfig {
  pub enabled: bool,
  pub port: Option<u32>,
  pub pull: Option<RtmpPullConfig>,
  pub push: Option<Vec<RtmpPushConfig>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RtmpPullConfig {
  pub enabled: bool,
  pub address: String,
  pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RtmpPushConfig {
  pub enabled: bool,
  pub address: String,
  pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpFlvConfig {
  pub enabled: bool,
  pub port: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HlsConfig {
  pub enabled: bool,
  pub port: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
  pub level: LogLevel,
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LogLevel {
  Off,
  Error,
  Warn,
  Info,
  Debug,
  Trace,
}

impl Into<LevelFilter> for LogLevel {
  fn into(self) -> LevelFilter {
    match self {
      LogLevel::Off => LevelFilter::Off,
      LogLevel::Error => LevelFilter::Error,
      LogLevel::Warn => LevelFilter::Warn,
      LogLevel::Info => LevelFilter::Info,
      LogLevel::Debug => LevelFilter::Debug,
      LogLevel::Trace => LevelFilter::Trace,
    }
  }
}
