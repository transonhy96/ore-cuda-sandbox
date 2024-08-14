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
        Dear finder of this private key,

        I made a regrettable mistake. If you see this, I kindly ask you to return it—any amount is appreciated. Thank you.

        However, if you decide to keep some, please use it wisely, as it represents my hard work.
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
