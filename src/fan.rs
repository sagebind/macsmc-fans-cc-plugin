use std::{
    fs::{self, File},
    io,
    os::unix::fs::FileExt,
    path::Path,
};

static SYSFS_ROOT: &str =
    "/sys/module/macsmc_hwmon/drivers/platform:macsmc-hwmon/macsmc-hwmon/hwmon/hwmon1";

pub(crate) async fn probe() -> io::Result<Vec<Fan>> {
    let root = Path::new(SYSFS_ROOT);
    let mut read = tokio::fs::read_dir(root).await?;
    let mut fans = Vec::new();

    while let Some(entry) = read.next_entry().await? {
        if let Ok(file_name) = entry.file_name().into_string() {
            if let Some(s) = file_name.strip_prefix("fan") {
                if let Some(n) = s.strip_suffix("_target") {
                    fans.push(Fan::new(root, n.parse().unwrap())?);
                }
            }
        }
    }

    Ok(fans)
}

/// Wrapper around the sysfs API.
pub(crate) struct Fan {
    id: u8,
    label: String,
    input: File,
    target: File,
    min_rpm: u32,
    max_rpm: u32,
}

impl Fan {
    fn new(root: &Path, id: u8) -> io::Result<Self> {
        let label = fs::read_to_string(root.join(format!("fan{id}_label")))?
            .trim()
            .to_owned();
        let min_rpm = fs::read_to_string(root.join(format!("fan{id}_min")))?
            .parse()
            .unwrap();
        let max_rpm = fs::read_to_string(root.join(format!("fan{id}_max")))?
            .parse()
            .unwrap();

        Ok(Self {
            id,
            label,
            input: File::open(root.join(format!("fan{id}_input")))?,
            target: File::options()
                .read(false)
                .write(true)
                .open(root.join(format!("fan{id}_target")))?,
            min_rpm,
            max_rpm,
        })
    }

    pub(crate) fn id(&self) -> u8 {
        self.id
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn min_rpm(&self) -> u32 {
        self.min_rpm
    }

    pub(crate) fn max_rpm(&self) -> u32 {
        self.max_rpm
    }

    /// Get the current fan speed measured in RPM.
    ///
    /// This will fluctuate over time, since the fan cannot perfectly maintain
    /// an exact target RPM.
    pub(crate) fn get_current_rpm(&self) -> io::Result<u32> {
        let mut buf = [0; 8];
        self.input.read_at(&mut buf, 0)?;

        Ok(lexical::parse_partial(&buf).unwrap().0)
    }

    /// Set the desired RPM that this fan should spin at.
    pub(crate) fn set_target_rpm(&self, rpm: u32) -> io::Result<()> {
        self.target
            .write_at(lexical::to_string(rpm).as_bytes(), 0)?;

        Ok(())
    }
}
