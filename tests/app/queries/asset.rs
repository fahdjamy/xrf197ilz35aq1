use crate::helpers::start_test_app;
use crate::seed::create_asset;
use xrf1::core::queries;

#[tokio::test]
async fn test_create_asset() {
    // 1. Start app
    let app = start_test_app().await;
    // 2. Set up test data
    let asset = create_asset().expect("Failed to create asset object");

    // 3. Create asset
    let result = queries::create_new_asset(&asset, app.user_fp.clone(), &app.db_pool).await;

    // 4. Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    app.drop_db().await;
}

#[tokio::test]
async fn test_find_asset_by_id_success() {
    // 1. Start app
    let app = start_test_app().await;

    // 2. Set up test data
    let asset = create_asset().expect("Failed to create asset object");

    // 3. Create asset in db
    queries::create_new_asset(&asset, app.user_fp.clone(), &app.db_pool).await
        .expect("Failed to create asset object");

    // 4. fetch created asset
    let result = queries::find_asset_by_id(&asset.id, &app.db_pool).await;

    // 5. Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, asset.id);

    // 6. Cleanup
    app.drop_db().await;
}

#[tokio::test]
async fn test_find_asset_by_owner_fp_success() {
    // 1. Start app
    let app = start_test_app().await;

    // let results =
    // 6. Cleanup
    app.drop_db().await;
}
