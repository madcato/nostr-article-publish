use crate::cli::Args;
use crate::config::Relays;
use anyhow::{Context, Result};
use nostr_sdk::prelude::*;
use std::{env, fs};
use toml;

pub async fn init_nostr_client(args: Args) -> Result<(Keys, Client)> {
    // Load nostr sec key to sign the message
    let bech32_sec_key = env::var("NOSTR_SEC_KEY").with_context(|| format!("To launch this command, define the enviroment variable NOSTR_SEC_KEY with the signing key"))?;
    let keys = Keys::parse(&bech32_sec_key)?;

    let config_file = args.config.unwrap_or(String::from("relays.toml"));

    // Load relays from relays.toml
    let relays_str = fs::read_to_string(config_file).with_context(|| format!("Configuration file 'relays.toml' could not be read."))?;
    let relays: Relays = toml::from_str(&relays_str).with_context(|| format!("Error deserializing 'relays.toml'."))?;

    // Show bech32 public key
    let bech32_pubkey: String = keys.public_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Create new client with custom options
    let client = Client::builder().signer(keys.clone()).build();
    
    // Add relays
    for relay in relays.relays {
        client.add_relay(relay).await?;
    }
    
    // Connect to relays
    client.connect().await;

    Ok((keys, client))
}