#[macro_use]
extern crate log;

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

pub use app::App;
use error::{AppError, Result};
use flexi_logger::{AdaptiveFormat, Logger};
use std::str::FromStr;

mod app;
mod args;
mod config;
mod control;
mod csi;
mod error;
mod iscsi;
mod sock;
mod util;
mod zfs;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::new();
    let mut builder = flexi_logger::LogSpecification::builder();
    builder.default(flexi_logger::LevelFilter::Debug).module(
        "zed_csi",
        flexi_logger::LevelFilter::from_str(&args.log_level.as_str())?,
    );
    Logger::with(builder.build())
        .adaptive_format_for_stderr(AdaptiveFormat::Default)
        .set_palette("196;208;31;8;59".into())
        .start()?;

    let app = App::new(args)?;

    match app.run().await {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(e) => {
            error!("Failed: {}", e);
            std::process::exit(1);
        }
    }
}
