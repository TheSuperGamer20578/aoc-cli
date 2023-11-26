use anyhow::Result;
use dialoguer::Password;
use tracing::info;
use crate::Config;

pub fn token(config: &mut Config) -> Result<()> {
    config.token = Some(Password::new()
        .with_prompt("Enter your session token")
        .interact()?);
    info!("Token saved!");
    Ok(())
}
