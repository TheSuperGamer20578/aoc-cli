use std::io;
use std::io::Write;
use anyhow::Result;
use rpassword::read_password;
use crate::Config;

pub fn token(config: &mut Config) -> Result<()> {
    print!("Enter your session token: ");
    io::stdout().flush()?;
    config.token = Some(read_password()?);
    println!("Token saved!");
    Ok(())
}
