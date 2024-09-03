use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use dotenv::dotenv;
use std::{env, /*ops::Add */};
use serde::Deserialize;
use alloy::primitives::U256;

mod eth_operations;
use eth_operations::{disperse_token, disperse_eth, collect_token, approve_contract_spending, get_user_holdings, get_user_holdings_percentage, HoldingOptions};


#[derive(Deserialize)]
struct AddressInfo {
    address: String,
}

#[derive(Deserialize)]
struct DisperseTokenInfo {
    token: String,
    wallets: Vec<String>,
    amount: U256,
    percentage: bool
}

#[derive(Deserialize)]
struct DisperseEthInfo {
    wallets: Vec<String>,
    amount: U256,
    percentage: bool
}

#[derive(Deserialize)]
struct AddressWithAmount {
    address: String,
    amount: U256
}

#[derive(Deserialize)]
struct CollectTokenInfo {
    address_with_amount: Vec<AddressWithAmount>,
    token: String,
    to: String,
    percentage: bool
}

#[post("/disperse_token")]
async fn disperse_token_handler(body: web::Json<DisperseTokenInfo>) -> impl Responder {
    dotenv().ok();
    let pubkey = env::var("PUBKEY_ONE").expect("No pkey in dotenv file");
    let b = body.into_inner();
    
    let amount = if b.percentage {
        match get_user_holdings_percentage(HoldingOptions::Token, &pubkey, Some(&b.token), b.amount).await {
            Ok(result) => result,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Error calculating percentage: {}", e)),
        }
    } else {
        b.amount
    };

    let wallet_refs: Vec<&str> = b.wallets.iter().map(|s| s.as_str()).collect();

    for wallet in wallet_refs.iter() {
        print!("{}", wallet);
    }

    match disperse_token(&wallet_refs, amount, &b.token).await {
        Ok(tx_hash) => HttpResponse::Ok().body(format!("Transaction Hash: {:?}", tx_hash)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[post("/disperse_eth")]
async fn disperse_eth_handler(body: web::Json<DisperseEthInfo>) -> impl Responder {
    dotenv().ok();
    let pubkey = env::var("PUBKEY_ONE").expect("No pkey in dotenv file");
    let b = body.into_inner();
    
    let amount = if b.percentage {
        match get_user_holdings_percentage(HoldingOptions::Eth, &pubkey, None, b.amount).await {
            Ok(result) => result,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Error calculating percentage: {}", e)),
        }
    } else {
        b.amount
    };

    let wallet_refs: Vec<&str> = b.wallets.iter().map(|s| s.as_str()).collect();

    for wallet in wallet_refs.iter() {
        print!("{}", wallet);
    }

    match disperse_eth(&wallet_refs, amount).await {
        Ok(tx_hash) => HttpResponse::Ok().body(format!("Transaction Hash: {:?}", tx_hash)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[post("/collect_token")]
async fn collect_token_handler(body: web::Json<CollectTokenInfo>) -> impl Responder {
    dotenv().ok();

    let b = body.into_inner();

    let mut addresses: Vec<&str> = Vec::new();
    let mut amounts: Vec<U256> = Vec::new();

    for entry in &b.address_with_amount {
        let address = entry.address.as_str();
        addresses.push(address);

        if b.percentage {
            match get_user_holdings_percentage(HoldingOptions::Token, address, Some(&b.token), entry.amount).await {
                Ok(percentage_amount) => amounts.push(percentage_amount),
                Err(e) => return HttpResponse::InternalServerError().body(format!("Error calculating percentage for address {}: {}", address, e)),
            }
        } else {
            amounts.push(entry.amount);
        }
    }

    match collect_token(&addresses, &amounts, &b.token, &b.to).await {
        Ok(tx_hash) => HttpResponse::Ok().body(format!("Transaction Hash: {:?}", tx_hash)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[post("/approve_spending")]
async fn approve_spending_handler(body: web::Json<AddressInfo>) -> impl Responder {
    let address = body.into_inner().address;
    match approve_contract_spending(&address).await {
        Ok(tx_hash) => HttpResponse::Ok().body(format!("Transaction Hash: {:?}", tx_hash)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(approve_spending_handler)
            .service(disperse_token_handler)
            .service(disperse_eth_handler)
            .service(collect_token_handler)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

