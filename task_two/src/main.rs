use alloy::{
    network::TransactionBuilder, node_bindings::Anvil, primitives::{
        keccak256, Address, FixedBytes, Uint, U256
    }, providers::{
        ext::{AnvilApi, DebugApi }, Provider, ProviderBuilder, RootProvider
    }, rpc::types::{
        trace::geth::{
            CallFrame, GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions, GethDefaultTracingOptions
        }, Transaction, TransactionRequest
    }, sol, transports::http::{Client, Http}
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

    let anvil = Anvil::new().fork("https://eth-pokt.nodies.app").try_spawn()?;

    let rpc_url = anvil.endpoint().parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);

    let tx = provider.get_transaction_by_hash(tx_hash).await?.unwrap();

    let anvil_partial = Anvil::new().fork("https://eth-pokt.nodies.app").fork_block_number(tx.block_number.unwrap()-1).try_spawn()?;
    let rpc_url_partial = anvil_partial.endpoint().parse()?;
    let provider_partial = ProviderBuilder::new().on_http(rpc_url_partial);

    match Action::from_str(&action_input) {
        Ok(Action::Buy) => buy_info(provider, provider_partial, tx).await?,
        Ok(Action::Sell) => sell_info(provider, provider_partial, tx).await?,
        Err(_) => {
            println!("Invalid action! Please enter either 'buy' or 'sell'.");
            return Ok(());
        }
    }

    Ok(())
}

fn sum_calls(call_frame: &CallFrame, target_address: Address) -> U256 {
    let mut total_value = U256::from(0);

    if call_frame.typ == "CALL" {
        if let Some(to_address) = call_frame.to {
            if to_address == target_address {
                if let Some(value) = call_frame.value {
                    total_value += value;
                }
            }
        }
    }

    for sub_call in &call_frame.calls {
        total_value += sum_calls(sub_call, target_address);
    }

    total_value
}

fn prompt_user(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

async fn sell_info(provider: RootProvider<Http<Client>>, provider_partial: RootProvider<Http<Client>>, tx: Transaction) -> Result<()>  {
    let receipt = provider.get_transaction_receipt(tx.hash).await?.unwrap();

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

                let call_options = GethDebugTracingOptions {
                    config: GethDefaultTracingOptions {
                        disable_storage: Some(true),
                        enable_memory: Some(false),
                        ..Default::default()
                    },
                    tracer: Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer)),
                    ..Default::default()
                };

                let result = provider.debug_trace_transaction(tx.hash, call_options).await?.try_into_call_frame().unwrap();

                let sum_eth = sum_calls(&result, receipt.from);

                let (pre, post, decimals, ticker) = get_token_info(tx, provider_partial, sender_address, token_address).await?;

                let amount_token = format_with_padding(pre - post, decimals as usize);
                
                let amount_eth = format_with_padding(U256::from(sum_eth), 18);

                println!("Sent {} {}", &amount_token, ticker.to_lowercase());
                println!("Received {} ether", &amount_eth);
                break;
            }
        }
    }

    Ok(())
}

async fn buy_info(provider: RootProvider<Http<Client>>, provider_partial: RootProvider<Http<Client>>, tx: Transaction) -> Result<()> {
    let receipt = provider.get_transaction_receipt(tx.hash).await?.unwrap();

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

                let (pre, post, decimals, ticker) = get_token_info(tx, provider_partial, receiver_address, token_address).await?;

                let amount_sent = format_with_padding(post - pre, decimals as usize);

                println!("Sent {} ether", value);
                println!("Received {} {}", &amount_sent, ticker.to_lowercase());
                break;
            }
        }
    }

    Ok(())
}

async fn get_token_info(
    tx: Transaction,
    provider_partial: RootProvider<Http<Client>>,
    user_address: Address,
    token_address: Address 
) -> Result<(Uint<256, 4>, Uint<256, 4>, u8, String)> {

    provider_partial.anvil_auto_impersonate_account(true).await?;
    provider_partial.anvil_impersonate_account(user_address).await?;

    let contract = ERC20::new(token_address, provider_partial.clone());
    let decimals = contract.decimals().call().await?._0;
    let ticker = contract.symbol().call().await?._0;

    let bal_pre= contract.balanceOf(user_address).call().await?.balance;

    let new_tx = TransactionRequest::default()
        .with_from(tx.from)
        .with_to(tx.to.unwrap())
        .with_value(tx.value)
        .with_input(tx.input);

    provider_partial.send_transaction(new_tx).await?.get_receipt().await?;
    let bal_post= contract.balanceOf(user_address).call().await?.balance;

    Ok((bal_pre, bal_post, decimals, ticker))
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
