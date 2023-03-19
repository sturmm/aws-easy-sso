use anyhow::Result;
use ini::Ini;
use std::path::{Path, PathBuf};

pub struct AwsCliConfigService {
    config_dir: PathBuf
}

impl AwsCliConfigService {

    pub fn new(config_dir: &Path) -> Self {
        Self { config_dir: config_dir.to_path_buf() }
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
        let session_section_name = format!("easy-sso-session {}", &session_name);
        let profile_name = format!("{session_name}:{role_name}@{account_name}");
        let profile_section_name = format!("profile {}", &profile_name);
        let config_path = self.config_dir.join("config");
        let mut config = Ini::load_from_file(&config_path)?;

        config.set_to(Some(&session_section_name), String::from("sso_region") , String::from(sso_region));
        config.set_to(Some(&session_section_name), String::from("sso_start_url") , String::from(start_url));
        config.set_to(Some(&session_section_name), String::from("sso_registration_scopes") , String::from("sso:account:access"));

        config.set_to(Some(&profile_section_name), String::from("sso_session"), String::from(session_name));
        config.set_to(Some(&profile_section_name), String::from("sso_account_id"), String::from(account_id));
        config.set_to(Some(&profile_section_name), String::from("sso_role_name"), String::from(role_name));
        config.set_to(Some(&profile_section_name), String::from("region"), String::from(sso_region));
        config.set_to(Some(&profile_section_name), String::from("output"), String::from("json"));
        
        config.write_to_file(config_path)?;

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