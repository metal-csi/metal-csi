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
    pub async fn get_targetcli(&self) -> Result<TargetCLI> {
        self.connect().await?;
        let targetcli = self.exec_open("targetcli").await?;
        let mut result = TargetCLI {
            cmd: self.clone(),
            targetcli,
        };
        result.wait_for_prompt().await?;
        Ok(result)
    }

    pub async fn get_iscsiadm(&self) -> Result<Iscsiadm> {
        self.connect().await?;
        Ok(self.clone().into())
    }
}
