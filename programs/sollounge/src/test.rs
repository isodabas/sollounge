use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;

mod constants;
use crate::constants::*;

fn main() {
    use super::*;
    // let rpc_url = String::from("https://api.devnet.solana.com");
    let rpc_url = String::from("https://rpc.ankr.com/solana_devnet");
    let connection = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let program_id = Pubkey::from_str("7KdfDLvqrAfoZGZUjqsi17duspUb7fQR67FuTZZYEY61").unwrap();
    let accounts = connection.get_program_accounts(&program_id).unwrap();

    println!("accounts for {}, {:?}", program_id, accounts);
}