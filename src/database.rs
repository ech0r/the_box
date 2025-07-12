use sled::{Db, Tree};
use serde::{Serialize, Deserialize};
use crate::auth::GitHubUser;
use crate::config::ConfigResponse;

#[derive(Clone)]
pub struct Database {
    users: Tree,
    configs: Tree,
    config_hashes: Tree,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        let users = db.open_tree("users")?;
        let configs = db.open_tree("configs")?;
        let config_hashes = db.open_tree("config_hashes")?;
        
        Ok(Database {
            users,
            configs,
            config_hashes,
        })
    }
    
    pub fn store_user(&self, user: &GitHubUser) -> Result<(), Box<dyn std::error::Error>> {
        let key = user.id.to_string();
        let value = serde_json::to_vec(user)?;
        self.users.insert(key, value)?;
        Ok(())
    }
    
    pub fn get_user(&self, user_id: u64) -> Result<Option<GitHubUser>, Box<dyn std::error::Error>> {
        let key = user_id.to_string();
        if let Some(value) = self.users.get(key)? {
            let user: GitHubUser = serde_json::from_slice(&value)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    pub fn store_config(
        &self,
        user_id: u64,
        hash: &str,
        config: &ConfigResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let user_key = user_id.to_string();
        let config_value = serde_json::to_vec(config)?;
        
        // Store config for user
        self.configs.insert(user_key, config_value.clone())?;
        
        // Store hash mapping
        self.config_hashes.insert(hash, config_value)?;
        
        Ok(())
    }
    
    pub fn get_config(&self, user_id: u64) -> Result<Option<ConfigResponse>, Box<dyn std::error::Error>> {
        let key = user_id.to_string();
        if let Some(value) = self.configs.get(key)? {
            let config: ConfigResponse = serde_json::from_slice(&value)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
    
    pub fn get_config_by_hash(&self, hash: &str) -> Result<Option<ConfigResponse>, Box<dyn std::error::Error>> {
        if let Some(value) = self.config_hashes.get(hash)? {
            let config: ConfigResponse = serde_json::from_slice(&value)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
}

