use super::*;
use crate::{control::ControlModule, Result};
use std::collections::HashMap;

#[derive(Debug, Deref, DerefMut, From)]
pub struct ZFS(ControlModule);

impl ZFS {
    pub async fn list_datasets(&self) -> Result<Vec<ZFSDatasetEntry>> {
        let mut result = vec![];
        let (output, code) = self.exec("zfs list -H").await?;
        if code != 0 {
            return Err(AppError::Generic(format!(
                "ZFS list failed with code {}!\n{}",
                code, output
            )));
        }
        for line in output.split("\n") {
            let props: Vec<&str> = line.split("\t").collect();
            if props.len() != 5 {
                continue;
            }
            result.push(ZFSDatasetEntry {
                name: props[0].to_string(),
                used: props[1].to_string(),
                avail: props[2].to_string(),
                refer: props[3].to_string(),
                mountpoint: props[4].to_string(),
            });
        }
        Ok(result)
    }

    pub async fn get_dataset<T: Into<String>>(&self, name: T) -> Result<Option<ZFSDataset>> {
        let name = name.into();
        let cmd = format!("zfs get -H all '{}'", name);
        let (output, code) = self.exec(&cmd).await?;
        if code == 1 {
            return Ok(None);
        }
        let mut properties = HashMap::new();
        for line in output.split("\n") {
            let props: Vec<&str> = line.split("\t").collect();
            if props.len() != 4 {
                continue;
            }
            properties.insert(
                props[1].into(),
                ZFSProperty {
                    value: props[2].into(),
                    source: props[3].into(),
                },
            );
        }
        Ok(Some(ZFSDataset { properties, name }))
    }

    pub async fn create_dataset<T: Into<String>>(&self, name: T, size: Option<i64>) -> Result<()> {
        let name = name.into();
        let vopt = if let Some(s) = size {
            format!("-V {}", s)
        } else {
            Default::default()
        };
        let cmd = format!("zfs create {} '{}'", vopt, name);
        let (output, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(AppError::Generic(format!(
                "Failed to create ZFS dataset, exit code {}\n{}",
                code, output
            )));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ZFSDatasetEntry {
    pub name: String,
    pub used: String,
    pub avail: String,
    pub refer: String,
    pub mountpoint: String,
}

#[derive(Debug)]
pub struct ZFSDataset {
    name: String,
    properties: HashMap<String, ZFSProperty>,
}

#[derive(Debug)]
pub struct ZFSProperty {
    value: String,
    source: String,
}
