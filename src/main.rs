use std::time::Instant;
mod utils;

use drillx::{
    difficulty,
    gpu::{drill_hash, gpu_init, set_noise},
    noise::NOISE,
};
use ore_api::state::Proof;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{clock::Clock, signature::Keypair, signer::Signer, sysvar};
use utils::{get_config, get_updated_proof_with_authority};

async fn get_cutoff(rpc_client: &RpcClient, proof: Proof, buffer_time: u64) -> u64 {
    let clock = get_clock(rpc_client).await;
    proof
        .last_hash_at
        .saturating_add(60)
        .saturating_sub(buffer_time as i64)
        .saturating_sub(clock.unix_timestamp)
        .max(0) as u64
}

pub async fn get_clock(client: &RpcClient) -> Clock {
    let data = client
        .get_account_data(&sysvar::clock::ID)
        .await
        .expect("Failed to get clock account");
    bincode::deserialize::<Clock>(&data).expect("Failed to deserialize clock")
}

#[tokio::main]
async fn main() {
    // Initialize gpu
    unsafe {
        gpu_init();
        set_noise(NOISE.as_usize_slice().as_ptr());
    }
    let signer = Keypair::from_bytes(&[
        137, 73, 115, 40, 126, 248, 95, 34, 20, 34, 178, 161, 24, 216, 157, 98, 113, 44, 122, 56,
        3, 43, 247, 61, 41, 135, 117, 116, 158, 63, 61, 63, 13, 251, 185, 1, 229, 43, 48, 45, 22,
        223, 120, 167, 55, 106, 79, 178, 148, 2, 195, 12, 74, 194, 174, 66, 3, 190, 233, 223, 209,
        181, 38, 214,
    ])
    .unwrap();

    let rpc_client = RpcClient::new(
        "https://mainnet.helius-rpc.com/?api-key=43fb5389-26ee-4e30-af67-fc01f04ad86d".to_string(),
    );
    let mut last_hash_at = 0;
    // Fetch proof
    // let config = get_config(&rpc_client).await;
    let proof = get_updated_proof_with_authority(&rpc_client, signer.pubkey(), last_hash_at).await;

    //last_hash_at = proof.last_hash_at;

    //let cutoff_time = get_cutoff(&rpc_client, proof, 1).await;
    // Current challenge (255s for demo)
    let timer = Instant::now();
    let secs = 5;
    let challenge = proof.challenge;
    let mut nonce = [0; 8];
    unsafe {
        drill_hash(challenge.as_ptr(), nonce.as_mut_ptr(), secs);
    }
    println!("{nonce:?}");

    // Calculate hash
    let hx = drillx::hash(&challenge, &nonce);
    println!(
        "gpu found hash with difficulty {} in {} seconds: {}",
        difficulty(hx),
        timer.elapsed().as_secs(),
        bs58::encode(hx).into_string(),
    );
}
