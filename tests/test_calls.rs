use serde_json::json;

#[tokio::test]
async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

    let contract = sandbox.dev_deploy(&contract_wasm).await?;

    let owner_account = sandbox.dev_create_account().await?;
    let user_account = sandbox.dev_create_account().await?;
    let init_outcome = owner_account.call(contract.id(), "init").transact().await?;
    assert!(init_outcome.is_success());

    let failed_add_epoch_outcome = user_account
        .call(contract.id(), "add_epoch")
        .args_json(json!({"minimum_witness_for_claim_creation": 1, "epoch_start": 1000, "epoch_end": 2000, "witnesses": [{"address": "0x244897572368eadf65bfbc5aec98d8e5443a9072", "host": "http://localhost:3030"}]}))
        .transact()
        .await?;
    assert!(failed_add_epoch_outcome.is_failure());

    let add_epoch_outcome = owner_account
        .call(contract.id(), "add_epoch")
        .args_json(json!({"minimum_witness_for_claim_creation": 1, "epoch_start": 1000, "epoch_end": 2000, "witnesses": [{"address": "244897572368eadf65bfbc5aec98d8e5443a9072", "host": "http://localhost:3030"}]}))
        .transact()
        .await?;
    assert!(add_epoch_outcome.is_success());

    let verify_proof_outcome = user_account
        .call(contract.id(), "verify_proof")
        .args_json(json!({"proof": {
            "claimInfo": {
                "provider": "http".to_string(),
                "parameters":"{\"body\":\"\",\"geoLocation\":\"in\",\"method\":\"GET\",\"responseMatches\":[{\"type\":\"contains\",\"value\":\"_steamid\\\">Steam ID: 76561199632643233</div>\"}],\"responseRedactions\":[{\"jsonPath\":\"\",\"regex\":\"_steamid\\\">Steam ID: (.*)</div>\",\"xPath\":\"id(\\\"responsive_page_template_content\\\")/div[@class=\\\"page_header_ctn\\\"]/div[@class=\\\"page_content\\\"]/div[@class=\\\"youraccount_steamid\\\"]\"}],\"url\":\"https://store.steampowered.com/account/\"}".to_string(),
                "context": "{\"contextAddress\":\"user's address\",\"contextMessage\":\"for acmecorp.com on 1st january\"}".to_string()
            },
            "signedClaim": {
                "claim":{
                    "identifier": "531322a6c34e5a71296a5ee07af13f0c27b5b1e50616f816374aff6064daaf55".to_string(),
                    "owner": "e4c20c9f558160ec08106de300326f7e9c73fb7f".to_string(),
                    "epoch": 1,
                    "timestampS": 1710157447,
                },
                "signatures": ["52e2a591f51351c1883559f8b6c6264b9cb5984d0b7ccc805078571242166b357994460a1bf8f9903c4130f67d358d7d6e9a52df9a38c51db6a10574b946884c1b".to_string()]
            }
        }}))
        .max_gas()
        .transact()
        .await?;

    assert!(verify_proof_outcome.is_success());

    let failed_verify_proof_outcome = user_account
        .call(contract.id(), "verify_proof")
        .args_json(json!({"proof": {
            "claimInfo": {
                "provider": "http".to_string(),
                "parameters":"{\"body\":\"\",\"geoLocation\":\"in\",\"method\":\"GET\",\"responseMatches\":[{\"type\":\"contains\",\"value\":\"_steamid\\\">Steam ID: 76561199632643233</div>\"}],\"responseRedactions\":[{\"jsonPath\":\"\",\"regex\":\"_steamid\\\">Steam ID: (.*)</div>\",\"xPath\":\"id(\\\"responsive_page_template_content\\\")/div[@class=\\\"page_header_ctn\\\"]/div[@class=\\\"page_content\\\"]/div[@class=\\\"youraccount_steamid\\\"]\"}],\"url\":\"https://store.steampowered.com/account/\"}".to_string(),
                "context": "{\"contextAddress\":\"user's address\",\"contextMessage\":\"for acmecorp.com on 1st january\"}".to_string()
            },
            "signedClaim": {
                "claim":{
                    "identifier": "531322a6c34e5a71296a5ee07af13f0c27b5b1e50616f816374aff6064daaf55".to_string(),
                    "owner": "e4c20c9f558160ec08106de300326f7e9c73fb7f".to_string(),
                    "epoch": 1,
                    "timestampS": 1710157447,
                },
                "signatures": ["52e2a591f5135111883559f8b6c6264b9cb5984d0b7ccc805078571242166b357994460a1bf8f9903c4130f67d358d7d6e9a52df9a38c51db6a10574b946884c1b".to_string()]
            }
        }}))
        .max_gas()
        .transact()
        .await?;
    assert!(failed_verify_proof_outcome.is_failure());

    Ok(())
}
