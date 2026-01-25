use alloy::primitives::Address;
use alloy::signers::Signer;
use alloy::signers::local::{LocalSigner, PrivateKeySigner};
use anyhow::{Context, Result};
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::clob::types::SignatureType;
use polymarket_client_sdk::clob::{Client, Config};
use std::str::FromStr as _;
use std::sync::Arc;

pub struct SigningUtils {
    pub client: Arc<Client<Authenticated<Normal>>>,
    pub signer: PrivateKeySigner,
    pub funder_address: Address,
}

impl SigningUtils {
    pub async fn new_client(private_key: &String, funder_address: &str) -> Result<Self> {
        let signer = LocalSigner::from_str(private_key)
            .context("Invalid private key")?
            .with_chain_id(Some(POLYGON));
        let funder: Address = funder_address.parse().context("Invalid funder address")?;
        let client = Client::new("https://clob.polymarket.com", Config::default())?
            .authentication_builder(&signer)
            .funder(funder)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await
            .context("Failed to create client")?;
        // println!("Client created: {:?}", client);
        Ok(Self {
            client: Arc::new(client),
            signer,
            funder_address: funder,
        })
    }
}
