use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::{ PathBuf, Path };

pub fn write_script(config_dir: &Path, sso_profile_name: &str) -> Result<PathBuf> {
    let file_path = config_dir.join("export_profile");
    let mut file = File::create(&file_path)?;

    let script = format!("#!/bin/false\nexport AWS_PROFILE=\"{}\"\n", sso_profile_name);

    file.write_all(script.as_bytes())?;
    Ok(file_path)
}
