use iroha::data_model::prelude::*;
use iroha_primitives::numeric::numeric;
use iroha_test_network::*;
use iroha_test_samples::gen_account_in;

#[test]
fn send_tx_with_different_chain_id() {
    let (_rt, _peer, test_client) = <PeerBuilder>::new().with_port(11_250).start_with_runtime();
    wait_for_genesis_committed(&[test_client.clone()], 0);
    // Given
    let (sender_id, sender_keypair) = gen_account_in("wonderland");
    let (receiver_id, _receiver_keypair) = gen_account_in("wonderland");
    let asset_definition_id = "test_asset#wonderland"
        .parse::<AssetDefinitionId>()
        .unwrap();
    let to_transfer = numeric!(1);

    let create_sender_account = Register::account(Account::new(sender_id.clone()));
    let create_receiver_account = Register::account(Account::new(receiver_id.clone()));
    let register_asset_definition =
        Register::asset_definition(AssetDefinition::numeric(asset_definition_id.clone()));
    let register_asset = Register::asset(Asset::new(
        AssetId::new(asset_definition_id.clone(), sender_id.clone()),
        numeric!(10),
    ));
    test_client
        .submit_all_blocking::<InstructionBox>([
            create_sender_account.into(),
            create_receiver_account.into(),
            register_asset_definition.into(),
            register_asset.into(),
        ])
        .unwrap();
    let chain_id_0 = ChainId::from("00000000-0000-0000-0000-000000000000"); // Value configured by default
    let chain_id_1 = ChainId::from("1");

    let transfer_instruction = Transfer::asset_numeric(
        AssetId::new("test_asset#wonderland".parse().unwrap(), sender_id.clone()),
        to_transfer,
        receiver_id.clone(),
    );
    let asset_transfer_tx_0 = TransactionBuilder::new(chain_id_0, sender_id.clone())
        .with_instructions([transfer_instruction.clone()])
        .sign(sender_keypair.private_key());
    let asset_transfer_tx_1 = TransactionBuilder::new(chain_id_1, sender_id.clone())
        .with_instructions([transfer_instruction])
        .sign(sender_keypair.private_key());
    test_client
        .submit_transaction_blocking(&asset_transfer_tx_0)
        .unwrap();
    let _err = test_client
        .submit_transaction_blocking(&asset_transfer_tx_1)
        .unwrap_err();
}