use std::env;
use std::fs;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

const TARGET_SCRIPT_NAME: &str = "_aws-easy-sso";

fn main() -> Result<(), std::io::Error> {
    let out_dir = env!("CARGO_HOME");
    let dest_bin_path = Path::new(out_dir).join("bin");
    
    if !dest_bin_path.exists() {
        fs::create_dir(&dest_bin_path)?;
    }

    let script_file = dest_bin_path.join(TARGET_SCRIPT_NAME);
    fs::write(
        &script_file,
        format!("#!/bin/false\n\nAWS_EASY_SSO_SOURCING_MODE=true aws-easy-sso-cli $@\n. ~/.awseasysso/export_profile")
    )?;

    let mut perms = fs::metadata(&script_file)?.permissions();
    perms.set_mode(0o776);
    fs::set_permissions(&script_file, perms)?;

    Ok(())
}