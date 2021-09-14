use super::*;
use crate::{control::ControlModule, Result};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZFSOptions {
    pub parent_dataset: String,
    pub attributes: HashMap<String, String>,
}

impl ZFSOptions {
    const ATTR_PREFIX: &'static str = "zfs.attr.";

    pub fn new(params: &HashMap<String, String>) -> Result<Self> {
        let mut parent_dataset = params
            .get("zfs.parentDataset")
            .ok_or_else(|| AppError::Generic(format!("ZFS Parent Dataset is required!")))?
            .to_string();
        if !parent_dataset.ends_with("/") {
            parent_dataset.push_str("/");
        }
        let mut attributes: HashMap<String, String> = Default::default();
        for (k, v) in params.iter() {
            if k.starts_with(Self::ATTR_PREFIX) {
                attributes.insert(k.to_string().split_off(9), v.to_string());
            }
        }
        Ok(ZFSOptions {
            parent_dataset,
            attributes,
        })
    }
}

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

    pub async fn set_attributes(
        &self,
        dataset: &str,
        attrs: &HashMap<String, String>,
    ) -> Result<()> {
        if attrs.len() > 0 {
            let mut cmd = "zfs set ".to_string();
            for (key, val) in attrs.iter() {
                cmd.push_str(&format!("'{}={}' ", key, val));
            }
            cmd.push_str(dataset);
            self.exec_checked(&cmd).await?;
        }
        Ok(())
    }

    pub async fn create_dataset<T: Into<String>>(&self, name: T, size: Option<i64>) -> Result<()> {
        let name = name.into();
        debug!("Creating ZFS dataset with name '{}'", name);
        let vopt = if let Some(s) = size {
            format!("-V {}", s)
        } else {
            Default::default()
        };
        if name.contains("/") {
            let mut parts: Vec<&str> = name.split("/").collect();
            let mut curpath = parts.remove(0).to_string();
            while parts.len() > 1 {
                let part = parts.remove(0);
                curpath = format!("{}/{}", curpath, part);
                debug!("Creating parent path '{}'", curpath);
                self.exec(&format!("zfs create '{}'", curpath)).await?;
            }
        }
        let cmd = format!("zfs create {} '{}'", vopt, name);
        let (output, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(AppError::Generic(
                format!(
                    "Failed to create ZFS dataset, exit code {}\n{}",
                    code, output
                )
                .trim()
                .to_string(),
            ));
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
