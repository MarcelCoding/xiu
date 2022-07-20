use anyhow::Result;
use config::{Config, Environment};
use log::{LevelFilter, set_max_level};
use simplelog::{ColorChoice, ConfigBuilder, LevelPadding, TerminalMode, TermLogger};
use time::macros::format_description;
use tokio;
use tokio::signal;

use hls::rtmp_event_processor::RtmpEventProcessor;
use hls::server as hls_server;
use httpflv::server as httpflv_server;
use rtmp::channels::channels::ChannelsManager;
use rtmp::relay::pull_client::PullClient;
use rtmp::relay::push_client::PushClient;
use rtmp::rtmp::RtmpServer;

use crate::xiu_config::{HlsConfig, HttpFlvConfig, RtmpConfig, XiuConfig};

mod xiu_config;

#[tokio::main]
async fn main() -> Result<()> {
  TermLogger::init(
    LevelFilter::Info,
    ConfigBuilder::new()
      .set_level_padding(LevelPadding::Right)
      .set_time_format_custom(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
      ))
      .build(),
    TerminalMode::Mixed,
    ColorChoice::Auto,
  )?;

  let config = Config::builder()
    .add_source(Environment::with_prefix("XIU").separator("_"))
    .build()?
    .try_deserialize::<XiuConfig>()?;

  if let Some(log) = &config.log {
    let filter = &log.level.into();
    set_max_level(*filter);
  }

  let mut service = Xiu::new(config);
  service.run().await?;

  shutdown_signal().await;

  Ok(())
}

async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  {
    let terminate = async {
      signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("failed to install signal handler")
        .recv()
        .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
  }

  #[cfg(not(unix))]
  ctrl_c.await;

  log::info!("signal received, starting graceful shutdown");
}

pub struct Xiu {
  config: XiuConfig,
}

impl Xiu {
  pub fn new(config: XiuConfig) -> Self {
    Xiu { config }
  }

  async fn run(&mut self) -> Result<()> {
    let mut channel = ChannelsManager::new();

    if let Some(httpflv_config) = &self.config.httpflv {
      if httpflv_config.enabled {
        Self::start_httpflv(&httpflv_config, &mut channel)?;
      }
    }

    if let Some(hls_config) = &self.config.hls {
      if hls_config.enabled {
        Self::start_hls(&hls_config, &mut channel)?;
      }
    }

    if let Some(rtmp_config) = &self.config.rtmp {
      if rtmp_config.enabled {
        Self::start_rtmp(rtmp_config, &mut channel)?;
      }
    }

    tokio::spawn(async move { channel.run().await });

    Ok(())
  }

  fn start_httpflv(config: &HttpFlvConfig, channel: &mut ChannelsManager) -> Result<()> {
    let port = config.port.unwrap_or(8081);
    let event_producer = channel.get_session_event_producer().clone();

    tokio::spawn(async move {
      if let Err(err) = httpflv_server::run(event_producer, port).await {
        //print!("push client error {}\n", err);
        log::error!("httpflv server error: {}\n", err);
      }
    });

    Ok(())
  }

  fn start_hls(config: &HlsConfig, channel: &mut ChannelsManager) -> Result<()> {
    let event_producer = channel.get_session_event_producer().clone();
    let cient_event_consumer = channel.get_client_event_consumer();
    let mut rtmp_event_processor = RtmpEventProcessor::new(cient_event_consumer, event_producer);

    tokio::spawn(async move {
      if let Err(err) = rtmp_event_processor.run().await {
        // print!("push client error {}\n", err);
        log::error!("rtmp event processor error: {}\n", err);
      }
    });

    let port = config.port;

    tokio::spawn(async move {
      if let Err(err) = hls_server::run(port).await {
        //print!("push client error {}\n", err);
        log::error!("hls server error: {}\n", err);
      }
    });

    channel.set_hls_enabled(true);

    Ok(())
  }

  fn start_rtmp(config: &RtmpConfig, channel: &mut ChannelsManager) -> Result<()> {
    let producer = channel.get_session_event_producer();

    /*static push */
    if let Some(push_cfg_values) = &config.push {
      for push_value in push_cfg_values {
        if !push_value.enabled {
          continue;
        }
        log::info!("start rtmp push client..");
        let address = format!(
          "{ip}:{port}",
          ip = push_value.address,
          port = push_value.port
        );

        let mut push_client = PushClient::new(
          address,
          channel.get_client_event_consumer(),
          producer.clone(),
        );
        tokio::spawn(async move {
          if let Err(err) = push_client.run().await {
            log::error!("push client error {}\n", err);
          }
        });

        channel.set_rtmp_push_enabled(true);
      }
    }

    /*static pull*/
    if let Some(pull_cfg_value) = &config.pull {
      if pull_cfg_value.enabled {
        let address = format!(
          "{ip}:{port}",
          ip = pull_cfg_value.address,
          port = pull_cfg_value.port
        );
        log::info!("start rtmp pull client from address: {}", address);
        let mut pull_client = PullClient::new(
          address,
          channel.get_client_event_consumer(),
          producer.clone(),
        );

        tokio::spawn(async move {
          if let Err(err) = pull_client.run().await {
            log::error!("pull client error {}\n", err);
          }
        });

        channel.set_rtmp_pull_enabled(true);
      }
    }

    let listen_port = config.port.unwrap_or(1935);
    let address = format!("0.0.0.0:{port}", port = listen_port);

    let mut rtmp_server = RtmpServer::new(address, producer.clone());
    tokio::spawn(async move {
      if let Err(err) = rtmp_server.run().await {
        //print!("rtmp server  error {}\n", err);
        log::error!("rtmp server error: {}\n", err);
      }
    });

    Ok(())
  }
}
