use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "zfs-csi", rename_all = "kebab_case")]
pub struct Args {
    #[structopt(short, long, default_value = "info")]
    /// Log level to use [trace, debug, info, warn, error]
    pub log_level: flexi_logger::LevelFilter,

    #[structopt(short("c"), long("config"), default_value = "/etc/zfscsi.yml")]
    /// Configuration file path
    pub config_path: PathBuf,
}

impl Args {
    pub fn new() -> Self {
        Args::from_args()
    }
}
