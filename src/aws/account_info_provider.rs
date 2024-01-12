use aws_sdk_sso::Client;
use anyhow::Result;
use std::fmt::Display;
use serde::{ Deserialize, Serialize };
use super::AccessToken;

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountInfo {
    pub account_name: String,
    pub account_id: String,
}

impl Display for AccountInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.account_name, self.account_id)
    }
}

pub struct AccountInfoProvider {
    client: Client
}

impl AccountInfoProvider {
    pub fn new(sdk_config: &aws_config::SdkConfig) -> Self {
        AccountInfoProvider { 
            client: Client::new(sdk_config)
         }
    }

    pub async fn get_account_list(&self, access_token: &AccessToken) -> Result<Vec<AccountInfo>> {
        let list_accounts = self.client.list_accounts()
            .access_token(access_token.access_token.as_str())
            .max_results(300)
            .send().await?;
    
        let account_infos = list_accounts.account_list().iter()
            .map(|account| {
                AccountInfo {
                    account_id: String::from(account.account_id().unwrap()),
                    account_name: String::from(account.account_name().unwrap_or("unknown")),
                }
            })
            .collect::<Vec<_>>();
    
        Ok(account_infos)
    }

    pub async fn get_roles_for_account(&self, access_token: &AccessToken, account_info: &AccountInfo) -> Result<Vec<String>>{
        let account_roles = self.client.list_account_roles()
            .access_token(access_token.access_token.as_str())
            .account_id(account_info.account_id.as_str())
            .max_results(10)
            .send().await?;
    
        Ok(
            account_roles.role_list().iter()
                .map(|r| r.role_name().unwrap() )
                .map(String::from)
                .collect::<Vec<_>>()
        )
    }
    
}
