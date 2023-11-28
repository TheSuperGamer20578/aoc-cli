use std::path::Path;
use anyhow::{bail, Result};
use tera::{Context, Tera};
use tokio::fs;
use tracing::info;

fn or_ellipsis(opt: Option<impl ToString>) -> String {
    opt.map_or("...".to_string(), |val| val.to_string())
}

pub async fn new(base_dir: &Path, template: String, new_path: &Path, year: Option<u16>, day: Option<u8>, part: Option<u8>) -> Result<()> {
    if new_path.exists() {
        bail!("{} already exists!", new_path.display());
    }
    let template_path = base_dir.join("templates")
        .join(&template)
        .with_extension("tera");
    let file = fs::read_to_string(&template_path).await?;
    let mut context = Context::new();
    context.insert("year", &or_ellipsis(year));
    context.insert("day", &or_ellipsis(day));
    context.insert("part", &or_ellipsis(part));
    context.insert("url", &match (year, day, part) {
        (Some(year), Some(day), Some(2)) => format!("https://adventofcode.com/{year}/day/{day}#part2"),
        (Some(year), Some(day), _) => format!("https://adventofcode.com/{year}/day/{day}"),
        (Some(year), _, _) => format!("https://adventofcode.com/{year}"),
        (_, _, _) => "https://adventofcode.com".to_string(),
    });
    let rendered = Tera::one_off(&file, &context, false)?;
    fs::write(&new_path, rendered).await?;
    info!("Successfully created {}!", new_path.display());
    Ok(())
}
