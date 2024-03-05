use std::{net::SocketAddr, str::FromStr, time::Duration};

use metrics::gauge;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::time::sleep;
use tracing::{debug, info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use carbonex::{
    types::{self, Region},
    Config,
};

fn setup_exporter(listen: impl Into<SocketAddr>) -> color_eyre::Result<()> {
    let builder = PrometheusBuilder::new().with_http_listener(listen.into());
    builder
        .install()
        .expect("failed to install recorder/exporter");
    Ok(())
}

fn get_config() -> color_eyre::Result<Config> {
    Ok(Config {
        postcodes: vec!["RG1".to_string(), "CB25".to_string()],
        listen_addr: "[::]:9993".parse().unwrap(),
    })
}

#[instrument(level = "INFO")]
async fn run_exporter_loop(postcode: String) {
    let forecast_metric_name = "carbon_regional_intensity_forecast_gco2e";
    let forecast_intensity = gauge!(forecast_metric_name,"postcode" => postcode.clone());
    let index_metric_name = "carbon_regional_intensity_index";
    let index_intensity = gauge!(index_metric_name,"postcode" => postcode.clone());
    // let generation_mix: BTreeMap<String,Gauge> = PowerType::iter().map(|t| {
    // gauge!("carbon_regional_power_x");
    // });
    let url = format!("https://api.carbonintensity.org.uk/regional/postcode/{postcode}");
    loop {
        info!("collecting metrics");
        let body = reqwest::get(&url)
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
        // println!("got {:?}", body.bytes().await.expect("x"));
        let dat: types::Data<Region> = body.json().await.unwrap();
        let region = dat.first().unwrap();
        debug!(
            region_data_count = region.region_data.len(),
            "got region data"
        );
        forecast_intensity.set(region.region_data.first().unwrap().intensity.forecast);
        index_intensity.set(
            region
                .region_data
                .first()
                .unwrap()
                .intensity
                .index
                .metric_value() as f64,
        );
        // println!("{dat:?}");
        sleep(Duration::from_secs(60 * 10)).await;
    }
}

fn setup_tracing() -> color_eyre::Result<()> {
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::from_str("INFO").unwrap());
    let fmt_layer = fmt::layer();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    Ok(())
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    setup_tracing()?;
    let cfg = get_config()?;
    setup_exporter(cfg.listen_addr)?;

    info!(listen_addr = ?cfg.listen_addr, "setting up listen socket");
    for postcode in cfg.postcodes {
        tokio::spawn(run_exporter_loop(postcode));
    }

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
