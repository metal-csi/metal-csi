#[derive(Debug, Display, Serialize, Deserialize, Clone)]
pub enum FilesystemType {
    Ext2,
    Ext3,
    Ext4,
    XFS,
    NFS,
    ZFS,
    TmpFs,
    Bind,
    Unknown,
}

impl FilesystemType {
    pub fn mount_type(&self) -> Option<&'static str> {
        match self {
            FilesystemType::Ext2 => Some("ext2"),
            FilesystemType::Ext3 => Some("ext3"),
            FilesystemType::Ext4 => Some("ext4"),
            FilesystemType::XFS => Some("xfs"),
            FilesystemType::NFS => Some("nfs4"),
            FilesystemType::TmpFs => Some("tmpfs"),
            _ => None,
        }
    }

    pub fn mkfs(&self) -> Option<&'static str> {
        match self {
            FilesystemType::Ext2 => Some("mkfs.ext2"),
            FilesystemType::Ext3 => Some("mkfs.ext3"),
            FilesystemType::Ext4 => Some("mkfs.ext4"),
            FilesystemType::XFS => Some("mkfs.xfs"),
            _ => None,
        }
    }

    pub fn mount_options(&self) -> Option<&'static str> {
        match self {
            &FilesystemType::Bind => Some("bind"),
            // &FilesystemType::NFS => Some("minorversion=1"),
            _ => None,
        }
    }
}

impl From<&str> for FilesystemType {
    fn from(fsname: &str) -> Self {
        match fsname.to_lowercase().as_str() {
            "ext2" => Self::Ext2,
            "ext3" => Self::Ext3,
            "ext4" => Self::Ext4,
            "xfs" => Self::XFS,
            "nfs" => Self::NFS,
            "tmpfs" => Self::TmpFs,
            "zfs" => Self::ZFS,
            "bind" => Self::Bind,
            _ => Self::Unknown,
        }
    }
}
