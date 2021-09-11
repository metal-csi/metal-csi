use super::*;

lazy_static! {
    static ref TARGETCLI_PROMPT: Regex = Regex::new("^/(\\S)*>").unwrap();
    static ref IQN_LINE: Regex =
        Regex::new("o-\\s+(?P<iqn>\\S+)\\s\\.+\\s\\[TPGs: (?P<tpgs>\\d+)]").unwrap();
    static ref TPG_ATTRIBUTE: Regex = Regex::new("(?P<attr>[a-z_0-9]+)=(?P<val>\\d+)").unwrap();
    static ref PARAMETER_SET_SUCCESS: Regex = Regex::new("Parameter \\w+ is now '\\d+'").unwrap();
}

pub struct TargetCLI {
    #[allow(dead_code)]
    /// Shell
    pub cmd: ControlModule,
    /// This is the stream opened into the targetcli command on the remove server
    pub targetcli: ControlStream,
}

impl TargetCLI {
    pub async fn send_cmd(&mut self, cmd: &str) -> Result<String> {
        self.targetcli.sendline(cmd).await?;
        let (output, _) = self.wait_for_prompt().await?;
        Ok(output)
    }

    pub async fn wait_for_prompt(
        &mut self,
    ) -> Result<(std::string::String, std::option::Option<u32>)> {
        self.targetcli.wait_for(&TARGETCLI_PROMPT).await
    }

    pub async fn list_iscsi_devices(&mut self) -> Result<Vec<String>> {
        let mut result = vec![];
        let output = self.send_cmd("ls /iscsi 1").await?;
        for cap in IQN_LINE.captures_iter(output.as_str()) {
            result.push(cap["iqn"].to_string());
        }
        Ok(result)
    }

    pub async fn create_backstore(&mut self, volume_id: &str) -> Result<String> {
        let normalized_id = volume_id.replace("/", "-");
        let backstore_name = format!("k8s-{}", normalized_id);
        let cmd = format!(
            "/backstores/block create {} /dev/zvol/{}",
            backstore_name, volume_id
        );
        self.send_cmd(&cmd).await?;
        Ok(backstore_name)
    }

    pub async fn create_target(&mut self, base_iqn: &str, volume_id: &str) -> Result<String> {
        let normalized_id = volume_id.replace("/", "-");
        let device_iqn = format!("{}:{}", base_iqn, normalized_id);
        let cmd = format!("/iscsi create {}", device_iqn);
        self.send_cmd(&cmd).await?;
        Ok(device_iqn)
    }

    pub async fn set_target_backstore(&mut self, iqn: &str, backstore: &str) -> Result<()> {
        let cmd = format!(
            "/iscsi/{}/tpg1/luns create /backstores/block/{}",
            iqn, backstore
        );
        self.send_cmd(&cmd).await?;
        Ok(())
    }

    pub async fn get_target_attributes(
        &mut self,
        iqn: &str,
        tpg: &str,
    ) -> Result<HashMap<String, i64>> {
        let mut result = HashMap::new();
        let cmd = format!("/iscsi/{0}/{1} get attribute", iqn, tpg);
        let output = self.send_cmd(&cmd).await?;
        for cap in TPG_ATTRIBUTE.captures_iter(output.as_str()) {
            result.insert(cap["attr"].to_string(), cap["val"].parse()?);
        }
        debug!("{} has attributes: {:?}", iqn, result);
        Ok(result)
    }

    pub async fn set_attribute(&mut self, iqn: &str, attr: &str, val: &str) -> Result<()> {
        let cmd = format!("/iscsi/{0}/tpg1 set attribute {1}={2}", iqn, attr, val);
        let output = self.send_cmd(cmd.as_str()).await?;
        if !PARAMETER_SET_SUCCESS.is_match(&output) {
            Err(AppError::Generic("Failed to set parameter!".into()))
        } else {
            Ok(())
        }
    }

    pub async fn close(mut self) -> Result<()> {
        self.targetcli.sendline("exit").await?;
        self.targetcli.wait_for_completion().await?;
        Ok(())
    }
}
