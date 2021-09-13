use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "zfs-csi", rename_all = "kebab_case")]
pub struct Args {
    #[structopt(short, long, default_value = "info")]
    /// Log level to use [trace, debug, info, warn, error]
    pub log_level: flexi_logger::LevelFilter,

    #[structopt(short("c"), long("config"), default_value = "/etc/zed-csi.yml")]
    /// Configuration file path
    pub config_path: PathBuf,

    #[structopt(long, default_value = "/plugin/csi.sock")]
    /// Configuration file path
    pub csi_path: PathBuf,

    #[structopt(long, default_value = "/plugin/metadata.db")]
    /// Metadata file path, this file will be created if it does not already exist
    pub metadata_db: PathBuf,

    #[structopt(long)]
    /// The name of the node this instance is running on
    pub node_id: String,

    #[structopt(long)]
    /// The name of the CSI Driver
    pub csi_name: String,
}

impl Args {
    pub fn new() -> Self {
        Args::from_args()
    }
}
