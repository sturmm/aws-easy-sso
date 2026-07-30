use crate::utils::serde::json_date_format;
use anyhow::{anyhow, Result};
use aws_config::SdkConfig;
use aws_sdk_ssooidc::Client;
use base64::Engine;
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccessToken {
    pub start_url: String,
    pub region: String,
    pub access_token: String,
    #[serde(with = "json_date_format")]
    pub expires_at: DateTime<Utc>,
    #[serde(flatten)]
    pub device_client: DeviceClient,
    pub refresh_token: String,
}

impl AccessToken {
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceClient {
    pub client_id: String,
    pub client_secret: String,
    #[serde(with = "json_date_format")]
    pub registration_expires_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,
}

impl DeviceClient {
    pub fn is_expired(&self) -> bool {
        self.registration_expires_at < Utc::now()
    }
}

pub struct SsoAccessTokenProvider {
    sso_session_name: String,
    client: Client,
    cache: super::AccessTokenCache,
}

#[derive(Debug, PartialEq, Eq)]
enum CachedTokenStrategy {
    Reuse,
    Refresh,
    Reauthenticate,
}

fn cached_token_strategy(token: &AccessToken) -> CachedTokenStrategy {
    if !token.is_expired() {
        CachedTokenStrategy::Reuse
    } else if !token.device_client.is_expired() {
        CachedTokenStrategy::Refresh
    } else {
        CachedTokenStrategy::Reauthenticate
    }
}

fn authorization_endpoint_for_region(region: &str) -> String {
    format!("https://oidc.{region}.amazonaws.com/authorize")
}

impl SsoAccessTokenProvider {
    const CLIENT_NAME: &'static str = "aws-easy-sso";
    const REFRESH_GRANT_TYPE: &'static str = "refresh_token";

    pub fn new(
        config: &SdkConfig,
        sso_session_name: &str,
        config_dir: &Path,
    ) -> anyhow::Result<Self> {
        let sso_cache_dir = config_dir.join("sso").join("cache");
        if !sso_cache_dir.exists() {
            fs::create_dir_all(&sso_cache_dir)?;
        }
        Ok(Self {
            sso_session_name: String::from(sso_session_name),
            client: Client::new(config),
            cache: super::AccessTokenCache::new(sso_session_name, sso_cache_dir.as_path()),
        })
    }

    pub async fn get_access_token(&self, start_url: &str) -> Result<AccessToken> {
        match self.cache.get_cached_token() {
            Ok(cached_token) => match cached_token_strategy(&cached_token) {
                // Do not contact OIDC while the cached access token is usable. This is
                // the same cache-first behaviour used by the AWS CLI and SDKs.
                CachedTokenStrategy::Reuse => Ok(cached_token),
                CachedTokenStrategy::Refresh => match self.refresh_token(&cached_token).await {
                    Ok(access_token) => Ok(access_token),
                    Err(error) => {
                        eprintln!(
                            "The cached IAM Identity Center session could not be refreshed ({error}); signing in again."
                        );
                        if cached_token.device_client.authorization_endpoint.is_some() {
                            self.authorize(start_url, cached_token.device_client).await
                        } else {
                            self.get_new_token(start_url).await
                        }
                    }
                },
                CachedTokenStrategy::Reauthenticate => self.get_new_token(start_url).await,
            },
            Err(_) => self.get_new_token(start_url).await,
        }
    }

    async fn get_new_token(&self, start_url: &str) -> Result<AccessToken> {
        let redirect_uri = crate::aws::pkce_flow::REDIRECT_URI;
        let mut issuer_url = url::Url::parse(start_url)?;
        issuer_url.set_fragment(None);

        let response = self
            .client
            .register_client()
            .client_name(format!("{}-{}", Self::CLIENT_NAME, self.sso_session_name))
            .client_type("public")
            .scopes("sso:account:access")
            .grant_types("authorization_code")
            .grant_types("refresh_token")
            .redirect_uris(redirect_uri)
            .issuer_url(issuer_url.as_str())
            .send()
            .await?;
        let client_id = response
            .client_id()
            .ok_or_else(|| anyhow!("OIDC registration did not return a client ID"))?;
        let client_secret = response
            .client_secret()
            .ok_or_else(|| anyhow!("OIDC registration did not return a client secret"))?;
        let authorization_endpoint = response
            .authorization_endpoint()
            .map(String::from)
            .unwrap_or_else(|| self.default_authorization_endpoint());
        let registration_expires_at = Utc
            .timestamp_opt(response.client_secret_expires_at(), 0)
            .single()
            .ok_or_else(|| anyhow!("OIDC registration returned an invalid expiration time"))?;
        let device_client = DeviceClient {
            client_id: String::from(client_id),
            client_secret: String::from(client_secret),
            registration_expires_at,
            authorization_endpoint: Some(authorization_endpoint),
        };

        self.authorize(start_url, device_client).await
    }

    async fn authorize(&self, start_url: &str, device_client: DeviceClient) -> Result<AccessToken> {
        let server = crate::aws::pkce_flow::LocalServer::bind().await?;
        let redirect_uri = server.redirect_uri.clone();
        let authorization_endpoint = device_client
            .authorization_endpoint
            .as_deref()
            .ok_or_else(|| anyhow!("The cached OIDC client has no authorization endpoint"))?;

        let pkce = crate::aws::pkce::Pkce::new();
        let mut state_bytes = [0u8; 16];
        getrandom::fill(&mut state_bytes)?;
        let state = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_bytes);

        let mut auth_url = url::Url::parse(authorization_endpoint)?;
        auth_url
            .query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &device_client.client_id)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("scopes", "sso:account:access")
            .append_pair("state", &state)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256");

        println!("Opening browser for authentication...");
        open::that(auth_url.as_str())?;

        let code = server.wait_for_code(&state).await?;

        let token_response = self
            .client
            .create_token()
            .client_id(&device_client.client_id)
            .client_secret(&device_client.client_secret)
            .grant_type("authorization_code")
            .code(code)
            .redirect_uri(redirect_uri)
            .code_verifier(&pkce.verifier)
            .send()
            .await?;

        let access_token = token_response
            .access_token()
            .ok_or_else(|| anyhow!("OIDC authorization did not return an access token"))?;
        let refresh_token = token_response
            .refresh_token()
            .ok_or_else(|| anyhow!("OIDC authorization did not return a refresh token"))?;
        let expires_at = Utc::now() + Duration::seconds(token_response.expires_in() as i64);

        let access_token = AccessToken {
            region: self.client.config().region().unwrap().to_string(),
            start_url: String::from(start_url),
            access_token: String::from(access_token),
            expires_at,
            device_client,
            refresh_token: String::from(refresh_token),
        };

        std::io::stdout().flush()?;

        self.cache.cache_token(access_token)
    }

    fn default_authorization_endpoint(&self) -> String {
        let region = self
            .client
            .config()
            .region()
            .expect("SSO OIDC client must have a region");
        authorization_endpoint_for_region(region.as_ref())
    }

    async fn refresh_token(&self, cached_token: &AccessToken) -> Result<AccessToken> {
        let device_client = &cached_token.device_client;
        let response = self
            .client
            .create_token()
            .client_id(device_client.client_id.as_str())
            .client_secret(device_client.client_secret.as_str())
            .grant_type(Self::REFRESH_GRANT_TYPE)
            .refresh_token(cached_token.refresh_token.as_str())
            .send()
            .await?;

        let access_token = response.access_token().unwrap();
        // AWS normally rotates the refresh token. Retain the existing one if a
        // compliant response omits a replacement rather than panicking and
        // discarding the usable cache entry.
        let refresh_token = response
            .refresh_token()
            .unwrap_or(cached_token.refresh_token.as_str());
        let expires_at = Utc::now() + Duration::seconds(response.expires_in() as i64);

        let new_access_token = AccessToken {
            region: self.client.config().region().unwrap().to_string(),
            start_url: cached_token.start_url.clone(),
            access_token: String::from(access_token),
            expires_at,
            device_client: cached_token.device_client.clone(),
            refresh_token: String::from(refresh_token),
        };

        self.cache.cache_token(new_access_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(expires_at: DateTime<Utc>, registration_expires_at: DateTime<Utc>) -> AccessToken {
        AccessToken {
            start_url: "https://example.awsapps.com/start".into(),
            region: "eu-west-1".into(),
            access_token: "access-token".into(),
            expires_at,
            device_client: DeviceClient {
                client_id: "client-id".into(),
                client_secret: "client-secret".into(),
                registration_expires_at,
                authorization_endpoint: None,
            },
            refresh_token: "refresh-token".into(),
        }
    }

    #[test]
    fn uses_a_valid_cached_access_token_without_refreshing() {
        let now = Utc::now();
        let token = token(now + Duration::minutes(30), now + Duration::days(1));

        assert_eq!(cached_token_strategy(&token), CachedTokenStrategy::Reuse);
    }

    #[test]
    fn refreshes_only_after_the_access_token_expires() {
        let now = Utc::now();
        let token = token(now - Duration::minutes(1), now + Duration::days(1));

        assert_eq!(cached_token_strategy(&token), CachedTokenStrategy::Refresh);
    }

    #[test]
    fn reauthenticates_when_the_client_registration_has_expired() {
        let now = Utc::now();
        let token = token(now - Duration::minutes(1), now - Duration::minutes(1));

        assert_eq!(
            cached_token_strategy(&token),
            CachedTokenStrategy::Reauthenticate
        );
    }

    #[test]
    fn derives_the_regional_authorization_endpoint() {
        assert_eq!(
            authorization_endpoint_for_region("eu-west-1"),
            "https://oidc.eu-west-1.amazonaws.com/authorize"
        );
    }
}
