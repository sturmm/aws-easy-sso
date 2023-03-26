use std::env;
// use std::fmt::format;
use std::fs;
use std::path::Path;

const TARGET_SCRIPT_NAME: &str = "_aws-easy-sso";

fn main() -> Result<(), std::io::Error> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).parent().unwrap().parent().unwrap().parent().unwrap();
    let dest_bin_path = dest_path.join("bin");
    
    if !dest_bin_path.exists() {
        fs::create_dir(&dest_bin_path)?;
    }

    fs::write(
        &dest_bin_path.join(TARGET_SCRIPT_NAME),
        format!("#!/bin/false\n\n{}/aws-easy-sso\n. ~/.awseasysso/export_profile", &dest_path.to_str().unwrap())
    )?;

    Ok(())
}