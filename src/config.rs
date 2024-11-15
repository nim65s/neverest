//! # Configuration
//!
//! Module dedicated to the main configuration of Neverest CLI.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::account::config::TomlAccountConfig;

/// The main configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TomlConfig {
    /// The configuration of all the accounts.
    pub accounts: HashMap<String, TomlAccountConfig>,
}

#[async_trait]
impl pimalaya_tui::terminal::config::TomlConfig for TomlConfig {
    type AccountConfig = TomlAccountConfig;

    fn project_name() -> &'static str {
        "neverest"
    }

    fn get_default_account_config(&self) -> Option<(String, Self::AccountConfig)> {
        self.accounts.iter().find_map(|(name, account)| {
            account
                .default
                .filter(|default| *default)
                .map(|_| (name.to_owned(), account.clone()))
        })
    }

    fn get_account_config(&self, name: &str) -> Option<(String, Self::AccountConfig)> {
        self.accounts
            .get(name)
            .map(|account| (name.to_owned(), account.clone()))
    }

    #[cfg(feature = "wizard")]
    async fn from_wizard(path: &std::path::Path) -> color_eyre::Result<Self> {
        use std::{fs, process::exit};

        use pimalaya_tui::terminal::{print, prompt};

        use crate::account;

        print::warn(format!("Cannot find configuration at {}.", path.display()));

        if !prompt::bool("Would you like to create one with the wizard?", true)? {
            exit(0);
        }

        print::section("Configuring your default account");

        let mut config = TomlConfig::default();

        let (account_name, account_config) = account::wizard::configure().await?;
        config.accounts.insert(account_name, account_config);

        let path = prompt::path("Where to save the configuration?", Some(path))?;
        println!("Writing the configuration to {}…", path.display());

        let toml = config.pretty_serialize()?;
        fs::create_dir_all(path.parent().unwrap_or(&path))?;
        fs::write(path, toml)?;

        println!("Done! Exiting the wizard…");

        Ok(config)
    }

    fn to_toml_account_config(
        &self,
        account_name: Option<&str>,
    ) -> pimalaya_tui::Result<(String, Self::AccountConfig)> {
        #[allow(unused_mut)]
        let (name, mut config) = match account_name {
            Some("default") | Some("") | None => self
                .get_default_account_config()
                .ok_or(pimalaya_tui::Error::GetDefaultAccountConfigError),
            Some(name) => self
                .get_account_config(name)
                .ok_or_else(|| pimalaya_tui::Error::GetAccountConfigError(name.to_owned())),
        }?;

        #[cfg(all(feature = "imap", feature = "keyring"))]
        if let Some(Backend::Imap(imap_config)) = config.backend.as_mut() {
            imap_config.auth.replace_empty_secrets(&name)?;
        }

        #[cfg(all(feature = "smtp", feature = "keyring"))]
        if let Some(SendingBackend::Smtp(smtp_config)) = config.message_send_backend_mut() {
            smtp_config.auth.replace_empty_secrets(&name)?;
        }

        Ok((name, config))
    }
}
