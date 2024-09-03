use alloy::{
    eips::BlockId, primitives::{keccak256, Address, FixedBytes, U256}, providers::{Provider, ProviderBuilder, RootProvider}, rpc::types::{Transaction, TransactionReceipt}, sol, transports::http::{Client, Http}
};
use eyre::Result;
use std::io;
use std::str::FromStr;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "abis/ERC20.json"
);

#[derive(Debug)]
enum Action {
    Buy,
    Sell
}

impl FromStr for Action {
    type Err = ();
    fn from_str(input: &str) -> Result<Action, Self::Err> {
        match input {
            "buy" => Ok(Action::Buy),
            "sell" => Ok(Action::Sell),
            _ => Err(()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥🔥🔥 Welcome To The BlazeDecoder 🔥🔥🔥");

    let tx_hash_string = prompt_user("Please enter the transaction hash for the transaction that you want to decode:");
    let action_input = prompt_user("Is this a buy or a sell transaction? (buy/sell)");

    let tx_hash = FixedBytes::from_str(&tx_hash_string).unwrap();

    match Action::from_str(&action_input) {
        Ok(Action::Buy) => buy_info(tx_hash).await?,
        Ok(Action::Sell) => sell_info(tx_hash).await?,
        Err(_) => {
            println!("Invalid action! Please enter either 'buy' or 'sell'.");
            return Ok(());
        }
    }

    Ok(())
}

fn prompt_user(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

// New helper function
async fn fetch_transaction_details(
    tx_hash: FixedBytes<32>,
) -> Result<(RootProvider<Http<Client>>, Transaction, TransactionReceipt)> {
    let rpc_url = "https://eth.merkle.io".parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let tx = provider.get_transaction_by_hash(tx_hash).await?.unwrap();
    let receipt = provider.get_transaction_receipt(tx_hash).await?.unwrap();

    Ok((provider, tx, receipt))
}

async fn sell_info(tx_hash: FixedBytes<32>) -> Result<()>  {
    let (provider, tx, receipt) = fetch_transaction_details(tx_hash).await?;

    let included_block = tx.block_number.unwrap();
    let from = tx.from;

    let logs = receipt.inner.logs();
    let signature_selector = keccak256("Transfer(address,address,uint256)");

    for log in logs.iter() {
        let topics = log.inner.data.topics();
        
        if !topics.is_empty() && topics[0] == signature_selector {
            let sender_address = Address::from_word(topics[1]);

            if sender_address == from {
                let token_address = log.inner.address;

                println!("Sent token address: {}", token_address);
                let contract = ERC20::new(token_address, &provider);

                let decimals = contract.decimals().call().await?._0;

                let ticker = contract.symbol().call().await?._0;

                let pre = contract.balanceOf(from).block(BlockId::number(included_block - 1)).call().await?.balance;
                let post = contract.balanceOf(from).block(BlockId::number(included_block + 1)).call().await?.balance;
                let bal_pre = provider.get_balance(from).block_id(BlockId::number(included_block - 1)).await?;
                let bal_post = provider.get_balance(from).block_id(BlockId::number(included_block + 1)).await?;
                let gas = U256::from(receipt.effective_gas_price * receipt.gas_used);
                println!("Gas: {}", gas);

                let amount_token = format_with_padding(pre - post, decimals as usize);

                let amount_eth = format_with_padding((bal_post - bal_pre) + gas, 18);

                println!("Sent {} {}", &amount_token, ticker.to_lowercase());
                println!("Received {} ether", &amount_eth);
                break;
            }
        }
    }

    Ok(())
}

async fn buy_info(tx_hash: FixedBytes<32>) -> Result<()> {
    let (provider, tx, receipt) = fetch_transaction_details(tx_hash).await?;

    let included_block = tx.block_number.unwrap();
    let from = tx.from;
    let value = format_with_padding(tx.value, 18);

    let logs = receipt.inner.logs();
    let signature_selector = keccak256("Transfer(address,address,uint256)");

    for log in logs.iter() {
        let topics = log.inner.data.topics();

        if !topics.is_empty() && topics[0] == signature_selector {
            let receiver_address = Address::from_word(topics[topics.len() - 1]);

            if receiver_address == from {
                let token_address = log.inner.address;
                println!("Received token address = {}", token_address);

                let contract = ERC20::new(token_address, &provider);

                let decimals = contract.decimals().call().await?._0;

                let ticker = contract.symbol().call().await?._0;

                let pre = contract.balanceOf(from).block(BlockId::number(included_block - 1)).call().await?.balance;
                let post = contract.balanceOf(from).block(BlockId::number(included_block + 1)).call().await?.balance;

                let amount_sent = format_with_padding(post - pre, decimals as usize);

                println!("Sent {} ether", value);
                println!("Received {} {}", &amount_sent, ticker.to_lowercase());
                break;
            }
        }
    }

    Ok(())
}

fn format_with_padding(value: U256, decimals: usize) -> String {
    let mut value_str = value.to_string();

    if value_str.len() < decimals + 1 {
        let padding = "0".repeat((decimals - value_str.len()) + 1);
        value_str = format!("{}{}", padding, value_str);
    }

    value_str.insert(value_str.len() - decimals, ',');
    value_str
}
