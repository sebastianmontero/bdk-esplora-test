use bdk::bitcoin::Txid;

use bdk::wallet::Update;
use bdk_esplora::{esplora_client, EsploraAsyncExt};
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let descriptor = "tr(814ae3fe7ab70538b4cef1c3153afe374aa26c6641941fbb7a1fea22884826b4,multi_a(2,[49652e19/86'/1'/784923']tpubDDkkbqetgxmZJeisHpUR3YonP4VnDoTfAuko67tQ4nsR4ZZioUzawPLWxiiWy7aRKBQyPR8Ns7YPUkivU87VhenAfPfP6xwHLpLdjQcjskJ/0/*,[cbde5d20/86'/1'/784923']tpubDDmqeznxFtXV5n4KW5tih3T1cM6Ca1CcM8HDhyJqHtzcaPzzJosEDYgixQAwWHg5CiMa4E1GZGW76VWrnmJ5jwqZBcwCrFLdNkPYTpdK9kT/0/*,[602431f3/86'/1'/784923']tpubDCXPWtns3nQ9VfFCJVfNc2Dyc3CDzZpvbEgxL2hEiH2DrhV5xgx944LTe8S8cAgq3yJreaHQaPtRvo6DzFCFTG4j9iYuHmSbZ7wtZp3axVg/0/*))#zrwgl0h6";

    let endpoint = "https://mempool.space/testnet/api/";
    let request_stop_gap = 40;
    let parallel_requests = 5;
    let mut wallet =
        bdk::Wallet::new_no_persist(descriptor, None, bdk::bitcoin::Network::Testnet).unwrap();
    let blockchain = esplora_client::Builder::new(endpoint)
        .build_async()
        .unwrap();

    let prev_tip = wallet.latest_checkpoint();
    let keychain_spks = wallet.spks_of_all_keychains();
    let (update_graph, last_active_indices) = blockchain
        .scan_txs_with_keychains(
            keychain_spks,
            None,
            None,
            request_stop_gap,
            parallel_requests,
        )
        .await
        .unwrap();
    let update = Update {
        graph: update_graph,
        ..Default::default()
    };
    wallet.apply_update(update).unwrap();

    let missing_heights = wallet.tx_graph().missing_heights(wallet.local_chain());
    let chain_update = blockchain
        .update_local_chain(prev_tip, missing_heights)
        .await
        .unwrap();
    let update = Update {
        last_active_indices,
        chain: Some(chain_update),
        ..Default::default()
    };
    wallet.apply_update(update).unwrap();
    wallet.commit().unwrap();

    let txid =
        Txid::from_str("c986542760cce19005b436fc45675a43819084bf37f683dae06e4816e77e8e9f").unwrap();
    let canonical_tx = wallet.get_tx(txid).unwrap();
    dbg!(canonical_tx.clone());
    dbg!(canonical_tx.chain_position);
    dbg!(canonical_tx.chain_position.is_confirmed());
    let confirmation_time: bdk::chain::ConfirmationTime =
        canonical_tx.chain_position.cloned().into();
    match confirmation_time {
        bdk::chain::ConfirmationTime::Confirmed { height, time } => {
            dbg!(height);
            dbg!(time);
        }
        bdk::chain::ConfirmationTime::Unconfirmed { last_seen } => {
            dbg!(last_seen);
        }
    }
}
