use std::path::PathBuf;
use tracing::info;
use anyhow::Result;
use crate::Config;
use crate::display::confirm;

pub fn trust(config: &mut Config, dir: &PathBuf) -> Result<()> {
    let dir = dir.canonicalize()?;
    if !confirm(format!("Are you sure you want to trust {}?", dir.display()))? {
        return Ok(());
    }
    config.trusted_dirs.push(dir.clone());
    info!("Successfully trusted {}!", dir.display());
    Ok(())
}
