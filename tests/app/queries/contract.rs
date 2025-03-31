use crate::queries::suit::{run_test_async, TestError};
use crate::seed::create_and_save_contract;
use std::collections::HashSet;
use xrf1::core::{queries, Contract, Currency, DomainError};

#[tokio::test]
async fn test_create_contract_success() {
    run_test_async(|app| async move {
        let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
            .await
            .expect("Failed to create and save seed asset");

        let contract = create_test_contract(asset.id).expect("failed to create contract");

        // Act:
        // Insert the contract into the database
        let result = queries::create_contract(&app.db_pool, contract).await;

        // Assert:
        // Check that the insertion was successful
        assert!(result.is_ok());

        Ok::<_, TestError>(())
    }).await
}

#[tokio::test]
async fn test_find_contract_by_asset_id() {
    run_test_async(|app| async move {
        let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
            .await
            .expect("Failed to create and save seed asset");

        // 3. Create contract
        let asset_id = asset.id;
        let contract = create_test_contract(asset_id.clone()).expect("failed to create contract");
        queries::create_contract(&app.db_pool, contract).await.expect("Failed to create contract");

        // 4. Find contract by asset id
        let created_contract = queries::find_contract_by_asset_id(&asset_id, &app.db_pool).await;

        assert!(created_contract.is_ok());
        assert_eq!(created_contract.unwrap().asset_id, asset_id);

        Ok::<_, TestError>(())
    }).await
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
