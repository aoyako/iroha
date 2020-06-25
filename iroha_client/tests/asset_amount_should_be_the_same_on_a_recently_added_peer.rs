#[cfg(test)]
mod tests {
    use async_std::task;
    use iroha::{isi, peer::isi as peer_isi, peer::PeerId, prelude::*};
    use iroha_client::client::{self, Client};
    use tempfile::TempDir;

    const CONFIGURATION_PATH: &str = "tests/test_config.json";
    const N_PEERS: usize = 4;
    const MAX_FAULTS: usize = 1;

    #[async_std::test]
    async fn asset_amount_should_be_the_same_on_a_recently_added_peer() {
        // Given
        let mut configuration =
            Configuration::from_path(CONFIGURATION_PATH).expect("Failed to load configuration.");
        let peers = create_and_start_iroha_peers(N_PEERS).await;
        task::sleep(std::time::Duration::from_millis(1000)).await;
        let domain_name = "domain";
        let create_domain = isi::Add {
            object: Domain::new(domain_name.to_string()),
            destination_id: PeerId::new(&configuration.torii_url, &configuration.public_key),
        };
        configuration.torii_url = peers
            .first()
            .expect("Failed to get first peer.")
            .address
            .clone();
        let account_name = "account";
        let account_id = AccountId::new(account_name, domain_name);
        let (public_key, _) = configuration.key_pair();
        let create_account = isi::Register {
            object: Account::with_signatory(account_name, domain_name, public_key),
            destination_id: String::from(domain_name),
        };
        let asset_id = AssetDefinitionId::new("xor", domain_name);
        let create_asset = isi::Register {
            object: AssetDefinition::new(asset_id.clone()),
            destination_id: domain_name.to_string(),
        };
        let mut iroha_client = Client::new(&configuration);
        iroha_client
            .submit_all(vec![
                create_domain.into(),
                create_account.into(),
                create_asset.into(),
            ])
            .await
            .expect("Failed to prepare state.");
        task::sleep(std::time::Duration::from_millis(
            configuration.pipeline_time_ms() * 2,
        ))
        .await;
        //When
        let quantity: u32 = 200;
        let mint_asset = isi::Mint {
            object: quantity,
            destination_id: AssetId {
                definition_id: asset_id.clone(),
                account_id: account_id.clone(),
            },
        };
        iroha_client
            .submit(mint_asset.into())
            .await
            .expect("Failed to create asset.");
        task::sleep(std::time::Duration::from_millis(
            configuration.pipeline_time_ms() * 2,
        ))
        .await;
        let key_pair = KeyPair::generate().expect("Failed to generate key pair.");
        let address = format!("127.0.0.1:{}", 1338 + N_PEERS);
        let new_peer = PeerId::new(&address, &key_pair.public_key);
        task::spawn(async move {
            let temp_dir = TempDir::new().expect("Failed to create TempDir.");
            let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                .expect("Failed to load configuration.");
            configuration.kura_block_store_path(temp_dir.path());
            configuration.torii_url = address;
            configuration.public_key = key_pair.public_key;
            configuration.private_key = key_pair.private_key.clone();
            configuration.trusted_peers(peers.clone());
            configuration.max_faulty_peers(MAX_FAULTS);
            let iroha = Iroha::new(configuration);
            iroha.start().await.expect("Failed to start Iroha.");
            //Prevents temp_dir from clean up until the end of the tests.
            loop {}
        });
        let add_peer = isi::Instruction::Peer(peer_isi::PeerInstruction::AddPeer(new_peer.clone()));
        iroha_client
            .submit(add_peer)
            .await
            .expect("Failed to add new peer.");
        task::sleep(std::time::Duration::from_millis(
            //TODO: get sync period from config
            20000,
        ))
        .await;
        //Then
        let mut configuration =
            Configuration::from_path(CONFIGURATION_PATH).expect("Failed to load configuration.");
        configuration.torii_url = new_peer.address.clone();
        let mut iroha_client = Client::new(&configuration);
        let request = client::assets::by_account_id(account_id);
        let query_result = iroha_client
            .request(&request)
            .await
            .expect("Failed to execute request.");
        if let QueryResult::GetAccountAssets(result) = query_result {
            assert!(!result.assets.is_empty());
            assert_eq!(
                quantity,
                result.assets.first().expect("Asset should exist.").quantity,
            );
        } else {
            panic!("Wrong Query Result Type.");
        }
    }

    async fn create_and_start_iroha_peers(n_peers: usize) -> Vec<PeerId> {
        let peer_keys: Vec<KeyPair> = (0..n_peers)
            .map(|_| KeyPair::generate().expect("Failed to generate key pair."))
            .collect();
        let peer_ids: Vec<PeerId> = peer_keys
            .iter()
            .enumerate()
            .map(|(i, key_pair)| PeerId {
                address: format!("127.0.0.1:{}", 1338 + i),
                public_key: key_pair.public_key,
            })
            .collect();
        for (peer_id, key_pair) in peer_ids.iter().zip(peer_keys) {
            let peer_ids = peer_ids.clone();
            let peer_id = peer_id.clone();
            task::spawn(async move {
                let temp_dir = TempDir::new().expect("Failed to create TempDir.");
                let mut configuration = Configuration::from_path(CONFIGURATION_PATH)
                    .expect("Failed to load configuration.");
                configuration.kura_block_store_path(temp_dir.path());
                configuration.torii_url = peer_id.address.clone();
                configuration.public_key = key_pair.public_key;
                configuration.private_key = key_pair.private_key.clone();
                configuration.trusted_peers(peer_ids.clone());
                configuration.max_faulty_peers(MAX_FAULTS);
                let iroha = Iroha::new(configuration);
                iroha.start().await.expect("Failed to start Iroha.");
                //Prevents temp_dir from clean up until the end of the tests.
                loop {}
            });
            task::sleep(std::time::Duration::from_millis(100)).await;
        }
        peer_ids.clone()
    }
}
