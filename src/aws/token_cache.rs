use super::AccessToken;
use anyhow::{Result};
use std::path::{Path, PathBuf};
use crate::utils::json;

pub struct AccessTokenCache {
    cache_dir: PathBuf,
    sso_session_name: String,
}

impl AccessTokenCache {
    pub fn new(sso_session_name: &str, cache_dir: &Path) -> Self {
        Self {
            cache_dir: cache_dir.to_path_buf(),
            sso_session_name: String::from(sso_session_name),
        }
    }

    pub fn get_cached_token(&self) -> Result<AccessToken> {
        let cache_file_path = self.cache_dir.join(format!("{}.json", self.hash_key()));
        json::read_from_file(cache_file_path.as_path())
    }
    
    pub fn cache_token(&self, access_token: AccessToken) -> Result<AccessToken> {
        let cache_file_path = self.cache_dir.join(format!("{}.json", self.hash_key()));
        json::write_to_file(cache_file_path.as_path(), &access_token)?;
        Ok(access_token)
    } 
    
    fn hash_key(&self) -> String {
        use sha1::{Sha1, Digest};

        let mut hasher = Sha1::new();
        hasher.update(self.sso_session_name.as_str());
    
        format!("{:02x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn should_return_some_cached_tokens() {
        // let cache = AccessTokenCache::new("./test_data");
        // let start_url = String::from("https://sso-start.awssso.aws/start");
        // let result = cache.has_access_token("eu-west-1", &start_url);

        // let expected_token = AccessToken {
        //     start_url: start_url,
        //     access_token: String::from("1234567890"),
        //     refresh_token: String::from("0987654321"),
        //     expires_in: 12345,
        // };

        // assert_eq!(result, Some(expected_token))
    }
}