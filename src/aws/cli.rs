use std::fs::File;
use anyhow::Result;
use ini::Ini;
use std::path::{Path, PathBuf};

pub struct AwsCliConfigService {
    config_file: PathBuf
}

impl AwsCliConfigService {

    pub fn new(config_file: &Path) -> Self {
        Self { config_file: config_file.to_path_buf() }
    }

    pub fn create_or_update_profile(
        &self,
        account_id: &str,
        account_name: &str,
        role_name: &str,
        start_url: &str,
        session_name: &str,
        sso_region: &str,
    ) -> Result<String> { 
        let session_section_name = format!("sso-session {}", &session_name);
        let profile_name = format!("{session_name}_{role_name}_{account_name}");
        let profile_section_name = format!("profile {}", &profile_name);

        if !&self.config_file.try_exists()? {
            File::create(&self.config_file)?;
        }

        let mut config = Ini::load_from_file(&self.config_file)?;

        config.set_to(Some(&session_section_name), String::from("sso_region") , String::from(sso_region));
        config.set_to(Some(&session_section_name), String::from("sso_start_url") , String::from(start_url));
        config.set_to(Some(&session_section_name), String::from("sso_registration_scopes") , String::from("sso:account:access"));

        config.set_to(Some(&profile_section_name), String::from("sso_session"), String::from(session_name));
        config.set_to(Some(&profile_section_name), String::from("sso_account_id"), String::from(account_id));
        config.set_to(Some(&profile_section_name), String::from("sso_role_name"), String::from(role_name));
        config.set_to(Some(&profile_section_name), String::from("region"), String::from(sso_region));
        config.set_to(Some(&profile_section_name), String::from("output"), String::from("json"));
        
        config.write_to_file(&self.config_file)?;

        Ok(profile_name)
    }

}

pub fn session_name(start_url: &str) -> String {
    let start_url_without_schema = start_url.replace("https://", "");
    let (subdomain, _) = start_url_without_schema
        .split_once(".")
        .unwrap();

    format!("sso-{}", &subdomain)
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn should_generate_update_config_file() {
        // let service = AwsCliConfigService::new("./test_data");
        
        // let profile_name = service.create_or_update_profile("011111111111", "account-name", "read-only", "https://987654.awsapps.com/start", "eu-west-1");
        
        // assert_eq!(profile_name.expect("Profile name generated"), "sso-read-only@account-name");
    }
    
}