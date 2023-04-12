use crate::aws::{session_name, AccountInfoProvider, AwsCliConfigService, SsoAccessTokenProvider};
use crate::config::SsoConfig;
use anyhow::Result;
use aws_types::region::Region;
use clap::Parser;
use directories::UserDirs;
use inquire::{Select, Text};
use script_writer::write_script;
use std::path::{PathBuf, Path};
use std::{fs, env};

pub mod aws;
pub mod config;
pub mod script_writer;
pub mod utils;

/// Improves handling of aws sso on the console by providing a UX 
/// comparable to the web UI.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Show selection of available sso_sessions. The selected session becomes the default for next logins.
    #[arg(short, long, default_value_t = false)]
    select_sso_session: bool,
    
    /// Show sso session configuration prompt
    #[arg(short, long, default_value_t = false)]
    configure: bool,
    
    /// Configure new session from the given start_url.
    #[arg(long, requires="sso_region", default_value = None)]
    start_url: Option<String>,
    
    /// Configure new session from the given sso_region.
    #[arg(long, requires="start_url", default_value = None)]
    sso_region: Option<String>,
}

impl Args {
    fn has_sso_config(&self) -> bool {
        self.start_url.is_some() && self.sso_region.is_some()
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let user_dirs = UserDirs::new().expect("Could not resolve user HOME.");
    let home_dir = user_dirs.home_dir();

    let config_dir = get_config_dir(home_dir)?;
    let aws_config_dir = home_dir.join(".aws");
    let aws_config_file = aws_config_dir.join("config");

    let sso_config = get_sso_config(&config_dir, &args)?;

    let config = aws_config::SdkConfig::builder()
        .region(Region::new(sso_config.region.clone()))
        .build();
    let aws_config_service = AwsCliConfigService::new(&aws_config_file);
    let account_info_provider = AccountInfoProvider::new(&config);

    let session_name = session_name(&sso_config.start_url.as_str());
    let token_provider =
        SsoAccessTokenProvider::new(&config, session_name.as_str(), &aws_config_dir);

    let access_token = token_provider
        .get_access_token(&sso_config.start_url)
        .await?;

    let sso_accounts = account_info_provider
        .get_account_list(&access_token)
        .await?;
    let selected_account = Select::new("Select account:", sso_accounts).prompt()?;

    let roles = account_info_provider
        .get_roles_for_account(&access_token, &selected_account)
        .await?;
    let selected_role = Select::new("Select role:", roles).prompt()?;

    let profile_name = aws_config_service.create_or_update_profile(
        &selected_account.account_id,
        &selected_account.account_name,
        &selected_role,
        &sso_config.start_url,
        &session_name,
        &sso_config.region,
    )?;

    if let Ok(_) = env::var("AWS_EASY_SSO_SOURCING_MODE") {
        write_script(&config_dir, &profile_name)?;
    } else {
        println!("export AWS_PROFILE={}", &profile_name);
    }

    Ok(())
}

fn get_config_dir(home_dir: &Path) -> Result<PathBuf> {
    let config_dir = home_dir.join(".awseasysso");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

fn get_sso_config(config_dir: &PathBuf, args: &Args) -> Result<SsoConfig> {
    let config_service = crate::config::ConfigProvider::new(config_dir);
    let mut user_config = config_service.get_user_config()?;

    let sso_config = if args.has_sso_config() {
        SsoConfig::new(
            args.start_url.as_deref().unwrap(),
            args.sso_region.as_deref().unwrap(),

        )
    } else if args.configure || !user_config.has_config() {
        configure_sso()?
    } else{
        if !args.select_sso_session {
            return Ok(user_config.get_default().unwrap().clone());
        } else {
            Select::new("SSO Config:", user_config.sso_config.clone()).prompt()?
        }
    };

    let mut filtered_sso_config_list: Vec<SsoConfig> = user_config
        .sso_config
        .into_iter()
        .filter(|it| !(it.start_url == sso_config.start_url && it.region == sso_config.region))
        .collect();

    let mut updated_sso_config = vec![sso_config.clone()];
    updated_sso_config.append(&mut filtered_sso_config_list);
    user_config.sso_config = updated_sso_config;
    config_service.update_user_config(&user_config)?;
    Ok(sso_config)
}

fn configure_sso() -> Result<SsoConfig> {
    let start_url = Text::new("SSO start-url:").prompt()?;
    let regions: Vec<String> = aws::REGIONS.iter().map(|it| String::from(*it)).collect();
    let sso_region = Select::new("SSO region:", regions).prompt()?;

    Ok(SsoConfig {
        start_url,
        region: String::from(sso_region),
    })
}
