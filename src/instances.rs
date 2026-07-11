use std::{
    io,
    path::{Path, PathBuf},
};

use tokio::fs;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instance {
    name: String,
    path: PathBuf,
}

impl Instance {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone, Debug)]
pub struct InstanceManager {
    root: PathBuf,
}

impl InstanceManager {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub async fn create(&self, name: impl AsRef<str>) -> io::Result<Instance> {
        let name = validate_name(name.as_ref())?;
        let path = self.root.join(&name);

        fs::create_dir_all(&self.root).await?;
        match fs::metadata(&path).await {
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Instance '{name}' already exists"),
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&path).await?;
                Ok(Instance { name, path })
            }
            Err(error) => Err(error),
        }
    }

    pub async fn get(&self, name: impl AsRef<str>) -> io::Result<Instance> {
        let name = validate_name(name.as_ref())?;
        let path = self.root.join(&name);
        let metadata = fs::metadata(&path).await?;

        if !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("Instance '{name}' is not a directory"),
            ));
        }

        Ok(Instance { name, path })
    }

    pub async fn list(&self) -> io::Result<Vec<Instance>> {
        let mut instances = Vec::new();
        let mut entries = match fs::read_dir(&self.root).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(instances),
            Err(error) => return Err(error),
        };

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                instances.push(Instance {
                    name,
                    path: entry.path(),
                });
            }
        }

        instances.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(instances)
    }
}

fn validate_name(name: &str) -> io::Result<String> {
    if name.is_empty()
        || Path::new(name).components().count() != 1
        || !matches!(
            Path::new(name).components().next(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Instance name must be a single directory name",
        ));
    }

    Ok(name.to_owned())
}
