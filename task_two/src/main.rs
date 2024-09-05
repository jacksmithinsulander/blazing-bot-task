use alloy::{
    eips::BlockId, network::TransactionBuilder, node_bindings::{Anvil, AnvilInstance}, primitives::{
        address, keccak256, Address, FixedBytes, Uint, U256
    }, providers::{
        ext::{AnvilApi, DebugApi, TraceApi}, Provider, ProviderBuilder, RootProvider
    }, rpc::types::{
        trace::geth::{
            CallFrame, GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions, GethDefaultTracingOptions
        }, Transaction, TransactionReceipt, TransactionRequest
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

    let receipt = provider.get_transaction_receipt(tx_hash).await?.unwrap();

    let anvil_partial = Anvil::new().fork("https://eth-pokt.nodies.app").fork_block_number(tx.block_number.unwrap()-1).try_spawn()?;
    let rpc_url_partial = anvil_partial.endpoint().parse()?;
    let provider_partial = ProviderBuilder::new().on_http(rpc_url_partial);

    trace_block(tx_hash, tx.block_number.unwrap(), provider, provider_partial /*, anvil, receipt */).await?;

    //match Action::from_str(&action_input) {
        //Ok(Action::Buy) => buy_info(tx_hash, provider, provider_partial).await?,
        //Ok(Action::Sell) => sell_info(tx_hash, provider, provider_partial).await?,
        //Err(_) => {
            //println!("Invalid action! Please enter either 'buy' or 'sell'.");
            //return Ok(());
        //}
    //}

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

async fn fetch_transaction_details(
    tx_hash: FixedBytes<32>,
) -> Result<(RootProvider<Http<Client>>, Transaction, TransactionReceipt)> {
    let rpc_url = "https://eth-pokt.nodies.app".parse()?;

    let provider = ProviderBuilder::new().on_http(rpc_url);

    let tx = provider.get_transaction_by_hash(tx_hash).await?.unwrap();
    let receipt = provider.get_transaction_receipt(tx_hash).await?.unwrap();

    Ok((provider, tx, receipt))
}

async fn sell_info(tx_hash: FixedBytes<32>, provider: RootProvider<Http<Client>>, provider_partial: RootProvider<Http<Client>>) -> Result<()>  {
    let (provider_, tx, receipt) = fetch_transaction_details(tx_hash).await?;

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

                let call_options = GethDebugTracingOptions {
                    config: GethDefaultTracingOptions {
                        disable_storage: Some(true),
                        enable_memory: Some(false),
                        ..Default::default()
                    },
                    tracer: Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer)),
                    ..Default::default()
                };

                let result = provider.debug_trace_transaction(tx_hash, call_options).await?.try_into_call_frame().unwrap();

                let sum_eth = sum_calls(&result, receipt.from);

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

async fn buy_info(tx_hash: FixedBytes<32>, provider: RootProvider<Http<Client>>, provider_partial: RootProvider<Http<Client>>) -> Result<()> {
    let (provider_, tx, receipt) = fetch_transaction_details(tx_hash).await?;

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

async fn trace_block(
    tx_hash: FixedBytes<32>, 
    included_block: u64, 
    provider: RootProvider<Http<Client>>,
    provider_partial: RootProvider<Http<Client>> 
) -> Result<(Uint<256, 4>, Uint<256, 4>)> {
    let block_trace = provider.trace_block(BlockId::number(included_block)).await?;


    let call_options = GethDebugTracingOptions {
            config: GethDefaultTracingOptions {
                disable_storage: Some(true),
                enable_memory: Some(false),
                ..Default::default()
            },
            tracer: Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer)),
            ..Default::default()
        };

    let mut bal_pre= U256::from(0);
    let mut bal_post = U256::from(0);
    for transaction in block_trace.iter() {
        let s = transaction.transaction_hash.unwrap();
        let q = transaction.transaction_position.unwrap();

        let result = provider.debug_trace_transaction(s, call_options.clone()).await?.try_into_call_frame().unwrap();
        let w = result.input;

        provider_partial.anvil_auto_impersonate_account(true).await?;
        provider_partial.anvil_impersonate_account(result.from).await?;

        if s == tx_hash {
            let o = transaction;
            println!("FOUND IT");

            let ctrct = ERC20::new(address!("ef00a1910642520c2F8e23d3C0A910933ca7f358"), provider_partial.clone());

            bal_pre= ctrct.balanceOf(result.from).call().await?.balance;
            print!("{o:?}");
            let new_transaction = TransactionRequest::default()
                .with_from(result.from)
                .with_to(result.to.unwrap())
                .with_value(result.value.unwrap())
                .with_input(w);

            let new_pending = provider_partial.send_transaction(new_transaction).await?;

            println!("Pending transaction... {}", new_pending.tx_hash());

            let new_receipt = new_pending.get_receipt().await?;

            println!(
                "Transaction included in block {}",
                new_receipt.block_number.expect("Failed to get block number")
            );
            bal_post = ctrct.balanceOf(result.from).call().await?.balance;

            println!("Before: {} After: {}", bal_pre, bal_post);

            let difference = bal_post - bal_pre;

            let decimals = ctrct.decimals().call().await?._0;
            let formatted_bought = format_with_padding(difference, decimals as usize);
            println!("Actual bought amount: {}", formatted_bought);

            break;
        }

    }


    Ok((bal_pre, bal_post))
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
