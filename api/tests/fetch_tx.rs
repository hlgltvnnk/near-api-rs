use near_api::{advanced::ExecuteSignedTransaction, *};
use near_api_types::{AccountId, NearToken};
use near_openapi_client::types::RpcTransactionStatusRequest;
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};
use testresult::TestResult;

#[tokio::test]
async fn fetch_tx_status() -> TestResult {
    let receiver: AccountId = "tmp_account".parse()?;
    let account: AccountId = DEFAULT_GENESIS_ACCOUNT.into();

    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    sandbox.create_account(receiver.clone()).send().await?;

    let network = NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let signer = Signer::from_secret_key(DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?)?;
    signer.set_sequential(true);

    let start_nonce = Account(account.clone())
        .access_key(signer.get_public_key().await?)
        .fetch_from(&network)
        .await?
        .data
        .nonce;

    let tx = Tokens::account(account.clone())
        .send_to(receiver.clone())
        .near(NearToken::from_millinear(1))
        .with_signer(signer.clone())
        .presign_with(&network)
        .await?;

    let signed = tx.transaction.signed_ref().clone().unwrap().clone();

    println!("TX: {:?}", signed.transaction);

    let tx = tx
        .wait_until(near_api_types::TxExecutionStatus::Included)
        .send_to(&network)
        .await?
        .assert_success();

    let hash1 = tx.transaction().get_hash();

    println!("TX2: {:?}", tx.transaction());

    println!("HASHES: {:?} {:?}", hash1, signed.get_hash());

    let end_nonce = Account(account.clone())
        .access_key(signer.get_public_key().await?)
        .fetch_from(&network)
        .await?
        .data
        .nonce;

    assert_eq!(end_nonce.0, start_nonce.0 + 1);

    // let tx = Tokens::account(account.clone())
    //     .send_to(receiver.clone())
    //     .near(NearToken::from_millinear(1))
    //     .with_signer(signer.clone())
    //     .wait_until(near_api_types::TxExecutionStatus::Included)
    //     .send_to(&network)
    //     .await?
    //     .assert_success();

    let res = ExecuteSignedTransaction::fetch_tx(
        network,
        // this is not working
        RpcTransactionStatusRequest::Variant1 {
            sender_account_id: account.clone(),
            tx_hash: tx.transaction().get_hash().into(),
            wait_until: near_api_types::TxExecutionStatus::IncludedFinal,
        },
        // this will work
        // RpcTransactionStatusRequest::Variant0 {
        //     signed_tx_base64: signed.into(),
        //     wait_until: near_api_types::TxExecutionStatus::IncludedFinal,
        // },
    )
    .await?
    .assert_success();

    assert!(res.outcome().is_success());

    Ok(())
}
