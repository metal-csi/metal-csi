use super::*;
use futures::future::{ready, Ready};
use std::io::Write;
use std::sync::Arc;
use thrussh::client::*;
use thrussh::*;
use thrussh_keys::*;
use tokio::sync::RwLock;

impl client::Handler for SSHClient {
    type Error = anyhow::Error;
    type FutureUnit = Ready<anyhow::Result<(Self, client::Session), anyhow::Error>>;
    type FutureBool = Ready<anyhow::Result<(Self, bool), anyhow::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        ready(Ok((self, b)))
    }

    fn finished(self, session: client::Session) -> Self::FutureUnit {
        ready(Ok((self, session)))
    }

    fn check_server_key(self, _: &key::PublicKey) -> Self::FutureBool {
        self.finished_bool(true)
    }
}

#[derive(Deref, Debug, Clone)]
pub struct SSHClient(Arc<InnerSSHClient>);

pub struct InnerSSHClient {
    private_key: String,
    host: String,
    user: String,
    handle: RwLock<Option<Handle<SSHClient>>>,
    sudo: bool,
}

impl std::fmt::Debug for InnerSSHClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerCSIClient")
            .field("user", &self.user)
            .field("private_key", &"[...]")
            .field("host", &self.host);
        Ok(())
    }
}

impl SSHClient {
    pub fn new<T: Into<String>, U: Into<String>, V: Into<String>>(
        user: V,
        host: T,
        private_key: U,
        sudo: bool,
    ) -> Result<Self> {
        Ok(SSHClient(Arc::new(InnerSSHClient {
            private_key: private_key.into(),
            host: host.into(),
            user: user.into(),
            handle: RwLock::new(None),
            sudo,
        })))
    }
}

#[async_trait]
impl ControlModuleTrait for SSHClient {
    async fn connect(&self) -> Result<()> {
        let config = thrussh::client::Config::default();
        let config = Arc::new(config);

        let key = Arc::new(thrussh_keys::decode_secret_key(
            self.private_key.as_str(),
            None,
        )?);

        let mut session =
            thrussh::client::connect(config, self.host.as_str(), self.clone()).await?;

        session
            .authenticate_publickey(self.user.as_str(), key)
            .await?;

        *self.handle.write().await = Some(session);
        Ok(())
    }
    async fn exec_open(&self, cmd: &str) -> Result<ControlStream> {
        let mut channel = if let Some(handle) = &mut *self.handle.write().await {
            handle.channel_open_session().await?
        } else {
            return Err(AppError::Generic("Not connected!".into()));
        };
        let cmd = self.build_command(self.sudo, None, cmd);
        channel.exec(true, &cmd).await.unwrap();
        Ok(ControlStream(Box::new(SSHStream { channel })))
    }

    async fn exec(&self, cmd: &str) -> Result<(String, u32)> {
        let cmd = self.build_command(self.sudo, None, cmd);
        let mut stream = self.exec_open(&cmd).await?;
        stream.wait_for_completion().await
    }

    async fn disconnect(&self) -> Result<()> {
        if self.is_connected().await? {
            let handle = self.handle.write().await.take();
            if let Some(mut handle) = handle {
                handle
                    .disconnect(Disconnect::ByApplication, "Client closed", "")
                    .await?;
            }
        }
        Ok(())
    }

    async fn is_connected(&self) -> Result<bool> {
        Ok(self.handle.read().await.is_some())
    }
}

pub struct SSHStream {
    channel: Channel,
}

impl Debug for SSHStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SSHStream").finish()
    }
}

#[async_trait]
impl ControlStreamTrait for SSHStream {
    async fn wait_for_completion(&mut self) -> Result<(String, u32)> {
        let mut output = Vec::new();
        let mut code = None;
        while let Some(msg) = self.channel.wait().await {
            match msg {
                thrussh::ChannelMsg::Data { ref data } => {
                    output.write_all(&data).unwrap();
                    debug!("{}", std::str::from_utf8(data)?);
                }
                thrussh::ChannelMsg::ExitStatus { exit_status } => code = Some(exit_status),
                thrussh::ChannelMsg::Eof => {
                    debug!("<EOF>")
                }
                _ => {}
            }
        }
        self.channel.eof().await?;
        Ok((std::str::from_utf8(&output)?.into(), code.unwrap_or(256)))
    }

    async fn wait_for(&mut self, ptrn: &Regex) -> Result<(String, Option<u32>)> {
        let mut output = Vec::new();
        let mut code = None;
        while let Some(msg) = self.channel.wait().await {
            match msg {
                thrussh::ChannelMsg::Data { ref data } => {
                    let str_data = std::str::from_utf8(data).unwrap_or_default();
                    debug!("{}", str_data);

                    if ptrn.is_match(str_data) {
                        debug!("Found the pattern we're waiting for...");
                        break;
                    }
                    output.write_all(&data).unwrap();
                }
                thrussh::ChannelMsg::ExitStatus { exit_status } => code = Some(exit_status),
                thrussh::ChannelMsg::Eof => {
                    debug!("<EOF>")
                }
                _ => {}
            }
        }
        Ok((std::str::from_utf8(&output)?.into(), code))
    }

    async fn sendline(&mut self, data: &str) -> Result<()> {
        let data = format!("{}\n", data);
        let data: Vec<u8> = data.into();
        self.channel.data(data.as_slice()).await?;
        Ok(())
    }
}
