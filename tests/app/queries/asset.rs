use crate::helpers::start_test_app;
use crate::seed::create_asset;
use xrf1::core::queries;
use xrf1::core::queries::OrderType;

#[tokio::test]
async fn test_create_asset() {
    // 1. Start app
    let app = start_test_app().await;
    // 2. Set up test data
    let asset = create_asset(app.user_fp.clone()).expect("Failed to create asset object");

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
    let asset = create_asset(app.user_fp.clone()).expect("Failed to create asset object");

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

    // 2. Check that user has no assets
    let user_fp = app.user_fp.clone();
    let assets = queries::find_assets_by_owner(&user_fp, 0,
                                               0,
                                               true, OrderType::Asc, &app.db_pool).await;
    assert!(assets.is_ok());
    assert_eq!(assets.unwrap().len(), 0);

    // 3. Set up test data
    let asset = create_asset(user_fp.clone()).expect("Failed to create asset object");

    // 4. Create asset in db
    queries::create_new_asset(&asset, user_fp.clone(), &app.db_pool).await
        .expect("Failed to create asset object");

    let assets = queries::find_assets_by_owner(&user_fp, 2,
                                               0,
                                               true, OrderType::Asc, &app.db_pool).await;
    // 5. Assert
    assert!(assets.is_ok());
    let stored_assets = assets.unwrap();
    assert_eq!(stored_assets.len(), 1);
    assert_eq!(stored_assets.get(0).unwrap().id, asset.id);

    // let results =
    // 6. Cleanup
    app.drop_db().await;
}

#[tokio::test]
async fn test_find_assets_name_like_success() {
    // 1. Start app
    let app = start_test_app().await;

    // 2. Check that user has no assets
    let user_fp = app.user_fp.clone();

    // 3. Set up test data
    let asset = create_asset(user_fp.clone()).expect("Failed to create asset object");

    // 4. Create asset in db
    queries::create_new_asset(&asset, user_fp.clone(), &app.db_pool).await
        .expect("Failed to create asset object");

    let asset_name = asset.name.clone();
    let offset = 0;
    let limit = 8;
    let assets = queries::find_assets_name_like(&asset_name[..5],
                                                offset,
                                                limit,
                                                OrderType::Asc, &app.db_pool)
        .await;

    // 5. Assert
    assert!(assets.is_ok());
    assert_eq!(assets.unwrap().len(), 1);

    // 6. Cleanup
    app.drop_db().await;
}

#[tokio::test]
async fn test_find_assets_symbol_like_success() {
    // 1. Start app
    let app = start_test_app().await;

    // 2. Check that user has no assets
    let user_fp = app.user_fp.clone();

    // 3. Set up test data
    let asset = create_asset(user_fp.clone()).expect("Failed to create asset object");
    // 4. Create asset in db
    queries::create_new_asset(&asset, user_fp.clone(), &app.db_pool).await
        .expect("Failed to create asset object");

    let symbol = asset.symbol.clone();
    let offset = 0;
    let limit = 8;

    let assets = queries::find_assets_symbol_like(&symbol[..3],
                                                  limit, offset, OrderType::Desc, &app.db_pool).await;

    // 5. Assert
    assert!(assets.is_ok());
    assert_eq!(assets.unwrap().len(), 1);

    // 6. Cleanup
    app.drop_db().await;
}
