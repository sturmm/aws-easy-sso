use std::env;
use std::fs;
use std::path::Path;

const TARGET_SCRIPT_NAME: &str = "_aws-easy-sso";

fn main() -> Result<(), std::io::Error> {
    let out_dir = env!("CARGO_HOME");
    let dest_bin_path = Path::new(out_dir).join("bin");
    
    let project_dir = env::current_dir().unwrap();
    let source_file = project_dir.join("run.sh");
    
    if !dest_bin_path.exists() {
        fs::create_dir_all(&dest_bin_path)?;
    }

    let script_file = dest_bin_path.join(TARGET_SCRIPT_NAME);
    fs::copy(&source_file, &script_file)?;

    Ok(())
}