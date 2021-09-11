use super::*;
use crate::control::{ControlModule, ControlStream};
use crate::Result;
pub use iscsiadm::*;
use regex::Regex;
use std::collections::HashMap;
pub use targetcli::*;

mod iscsiadm;
mod targetcli;

impl ControlModule {
    pub async fn targetcli(&self) -> Result<TargetCLI> {
        // let cmd: ControlModule = self.control_controller().await?;
        self.connect().await?;
        let targetcli = self.exec_open("targetcli").await?;
        let mut result = TargetCLI {
            cmd: self.clone(),
            targetcli,
        };
        result.wait_for_prompt().await?;
        Ok(result)
    }

    pub async fn iscsiadm(&self) -> Result<Iscsiadm> {
        self.connect().await?;
        Ok(self.clone().into())
    }
}
