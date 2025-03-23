use crate::helpers::start_test_app;
use crate::seed::create_and_save_contract;
use std::collections::HashSet;
use xrf1::core::{create_contract, find_contract_by_asset_id, Contract, Currency, DomainError};

#[tokio::test]
async fn test_create_contract_success() {
    // 1. Set up
    let app = start_test_app().await;

    // 2. create seed data
    let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
        .await
        .expect("Failed to create and save seed asset");

    // 3. Create test data
    let contract = create_test_contract(asset.id).expect("failed to create contract");

    // 4. Act:
    // Insert the contract into the database
    let result = create_contract(&app.db_pool, contract).await;

    // 5. Assert:
    // Check that the insertion was successful
    assert!(result.is_ok());

    // 6. Clean up: drop database that was connected to
    app.drop_db().await
}

#[tokio::test]
async fn test_find_contract_by_asset_id() {
    // 1. Set up
    let app = start_test_app().await;

    // 2. create seed data
    let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
        .await
        .expect("Failed to create and save seed asset");

    // 3. Create contract
    let asset_id = asset.id;
    let contract = create_test_contract(asset_id.clone()).expect("failed to create contract");
    create_contract(&app.db_pool, contract).await.expect("Failed to create contract");

    // 4. Find contract by asset id
    let created_contract = find_contract_by_asset_id(&asset_id, &app.db_pool).await;

    assert!(created_contract.is_ok());
    assert_eq!(created_contract.unwrap().asset_id, asset_id);

    // 6. Clean up: drop database that was connected to
    app.drop_db().await
}

fn create_test_contract(asset_id: String) -> Result<Contract, DomainError> {
    // Create a sample CurrencyList with various currencies
    let currencies = vec![Currency::USD, Currency::EUR, Currency::BTC];
    let currency_list = HashSet::from_iter(currencies);

    // Create a sample DbContract
    Contract::new(
        asset_id,
        "11".to_string(),
        "summary".to_string(),
        "user_fp".to_string(),
        20.0,
        false,
        3.0,
        "user_fp".to_string(),
        currency_list,
    )
}
