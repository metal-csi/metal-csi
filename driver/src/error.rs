pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Command failed with code {}{}", code, output)]
    CommandFailed {
        code: u32,
        output: String,
    },
    Generic(String),
}

impl<T: Into<anyhow::Error>> From<T> for AppError {
    fn from(e: T) -> Self {
        let e: anyhow::Error = e.into();
        Self::Generic(e.to_string())
    }
}

impl From<AppError> for tonic::Status {
    fn from(e: AppError) -> Self {
        warn!("{}", e);
        tonic::Status::aborted(e.to_string())
    }
}
