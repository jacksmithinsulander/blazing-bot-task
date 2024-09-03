use dotenv::dotenv;
use std::{env, /*ops::Add */};
use alloy::{
    network::{EthereumWallet, /* TransactionBuilder */ },
    primitives::{address, FixedBytes, U256, Address},
    providers::{
        ProviderBuilder,
    },
    //rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};
use eyre::{ErrReport, Result};
use url::Url;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Blaze,
    "abis/Blaze.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "abis/ERC20.json"
);

pub enum HoldingOptions {
    Eth,
    Token
}

pub async fn disperse_token(addresses: &[&str], amount: U256, token: &str) -> Result<FixedBytes<32>, ErrReport> {
    let signer = create_signer();

    let wallet = EthereumWallet::from(signer);
    let rpc_url: Url = "https://1rpc.io/sepolia".parse()?;

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);

    let contract = Blaze::new(address!("e5B4784F93671E6cF04026183E1DAe1A1E82F55A"), provider);

    let mut addresses_formatted = Vec::new();

    for address in addresses.iter() {
        addresses_formatted.push(Address::parse_checksummed(address, None).unwrap());
    }

    let token_formatted = Address::parse_checksummed(token, None).unwrap(); 

    let tx_hash = contract.disperseToken(addresses_formatted, amount, token_formatted).send().await?.watch().await?;

    Ok(tx_hash)
}

pub async fn disperse_eth(addresses: &[&str], amount: U256) -> Result<FixedBytes<32>, ErrReport> {
    let signer = create_signer();

    let wallet = EthereumWallet::from(signer);
    let rpc_url: Url = "https://1rpc.io/sepolia".parse()?;

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);

    let contract = Blaze::new(address!("e5B4784F93671E6cF04026183E1DAe1A1E82F55A"), provider);

    let mut addresses_formatted = Vec::new();

    for address in addresses.iter() {
        addresses_formatted.push(Address::parse_checksummed(address, None).unwrap());
    }

    let tx_hash = contract.disperseETH(addresses_formatted).value(amount).send().await?.watch().await?;

    Ok(tx_hash)
}

pub async fn collect_token(addresses: &[&str], amounts: &[U256], token: &str, to: &str) -> Result<FixedBytes<32>, ErrReport> {
    assert!(addresses.len() == amounts.len(), "The lengths of addresses and amounts do not match.");

    let signer = create_signer();

    let wallet = EthereumWallet::from(signer);
    let rpc_url: Url = "https://1rpc.io/sepolia".parse()?;

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);

    let contract = Blaze::new(address!("e5B4784F93671E6cF04026183E1DAe1A1E82F55A"), provider);

    let mut collect_datas = Vec::new();

    for (address, amount) in addresses.iter().zip(amounts.iter()) {
        let collect_data = Blaze::CollectData { payee: Address::parse_checksummed(address, None).unwrap(), amount: *amount };
        collect_datas.push(collect_data);
    }

    let to_formatted = Address::parse_checksummed(to, None).unwrap();

    let token_formatted = Address::parse_checksummed(token, None).unwrap();

    let tx_hash = contract.collectToken(collect_datas, token_formatted, to_formatted).send().await?.watch().await?;

    Ok(tx_hash)
}

pub async fn approve_contract_spending(address: &str) -> Result<Vec<FixedBytes<32>>, ErrReport> {
    let signers = create_signers();

    let rpc_url: Url = "https://1rpc.io/sepolia".parse()?;

    let mut tx_hashes = Vec::new();

    for signer in signers.iter() {
        let wallet = EthereumWallet::from(signer.clone());
        
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url.clone());

        let contract_address = Address::parse_checksummed(address, None).unwrap();

        let contract = ERC20::new(contract_address, provider);

        let approve_amount: i128 = 10_000_000_000_000_000_000_000;

        let num = U256::from(approve_amount);
        let addr = address!("e5B4784F93671E6cF04026183E1DAe1A1E82F55A");

        let tx_hash = contract.approve(addr, num).send().await?.watch().await?;
        
        tx_hashes.push(tx_hash);
    }

    Ok(tx_hashes)
}

pub async fn get_user_holdings(option: HoldingOptions, user: &str, token: Option<&str>) -> Result<U256, Box<dyn std::error::Error>> {
    let rpc_url: Url = "https://1rpc.io/sepolia".parse().unwrap();

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let contract = Blaze::new(address!("e5B4784F93671E6cF04026183E1DAe1A1E82F55A"), provider);

    let user_address = Address::parse_checksummed(user, None).unwrap();

    let balance = match option {
        HoldingOptions::Eth => contract.getEthHoldings(user_address).call().await?._userHoldings,
        HoldingOptions::Token => {
            let token_address = token.expect("Token address must be provided for Token holding option.");
            let parsed_token_address = Address::parse_checksummed(token_address, None).unwrap();

            contract.getTokenHolding(parsed_token_address, user_address).call().await?._userHoldings
        }
    };

    Ok(balance)
}

pub async fn get_user_holdings_percentage(option: HoldingOptions, user: &str, token: Option<&str>, percentage: U256) -> Result<U256, Box<dyn std::error::Error>> {
    let holdings = get_user_holdings(option, user, token).await?;

    let denominator: U256 = U256::from(10_000);

    let percentage_holdings = (holdings * percentage) / denominator;

    Ok(percentage_holdings)
}

fn create_signer() -> PrivateKeySigner {
    dotenv().ok();

    let pkey = env::var("PKEY_ONE").expect("No pkey in dotenv file");
    let signer: PrivateKeySigner = pkey.parse().expect("Failed to parse");
    signer
}

fn create_signers() -> [PrivateKeySigner; 4] {
    dotenv().ok();

    let pkeys = [
        env::var("PKEY_ONE").expect("No PKEY_ONE in dotenv file"),
        env::var("PKEY_TWO").expect("No PKEY_TWO in dotenv file"),
        env::var("PKEY_THREE").expect("No PKEY_THREE in dotenv file"),
        env::var("PKEY_FOUR").expect("No PKEY_FOUR in dotenv file"),
    ];

    let signers: [PrivateKeySigner; 4] = [
        pkeys[0].parse().expect("Failed to parse PKEY_ONE"),
        pkeys[1].parse().expect("Failed to parse PKEY_TWO"),
        pkeys[2].parse().expect("Failed to parse PKEY_THREE"),
        pkeys[3].parse().expect("Failed to parse PKEY_FOUR"),
    ];

    signers
}
