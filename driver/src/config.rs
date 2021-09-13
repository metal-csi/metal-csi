use crate::args::Args;
use crate::Result;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[derive(Debug, Deref, Default, Clone)]
pub struct Configuration(Arc<InnerConfiguration>);

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct InnerConfiguration {
    pub node: NodeOptions,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct NodeOptions {
    pub control_mode: ControlMode,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMode {
    Local {
        #[serde(default)]
        sudo: bool,
    },

    Chroot {
        #[serde(default)]
        sudo: bool,
        #[serde(default)]
        path: String,
    },

    #[serde(rename = "ssh")]
    SSH {
        #[serde(default)]
        sudo: bool,
        #[serde(default)]
        user: String,
        #[serde(default)]
        private_key: String,
        #[serde(default)]
        host: String,
        #[serde(default)]
        port: u16,
    },
}

impl Default for ControlMode {
    fn default() -> Self {
        Self::Local { sudo: false }
    }
}

impl Configuration {
    pub fn new(args: Args) -> Result<Self> {
        let file = File::open(args.config_path)?;
        let reader = BufReader::new(file);
        let res = serde_yaml::from_reader(reader)?;
        Ok(Self(Arc::new(res)))
    }
}
