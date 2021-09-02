use super::*;
use crate::control::{ControlModule, ControlStream};
use crate::Result;
pub use iscsiadm::*;
use regex::Regex;
use std::collections::HashMap;
pub use targetcli::*;

mod iscsiadm;
mod targetcli;

impl App {
    pub async fn targetcli(&self) -> Result<TargetCLI> {
        let cmd: ControlModule = self.control_controller().await?;
        let targetcli = cmd.exec_open("targetcli").await?;
        let mut result = TargetCLI {
            app: self.clone(),
            cmd,
            targetcli,
        };
        result.wait_for_prompt().await?;
        Ok(result)
    }

    pub async fn iscsiadm(&self) -> Result<Iscsiadm> {
        let result = self.control_node().await?;
        Ok(result)
    }
}
