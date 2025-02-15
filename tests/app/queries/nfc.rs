use crate::helpers::start_test_app;
use crate::seed::create_and_save_contract;
use xrf1::core::{create_nfc, get_nfc_by_id, get_nfc_trails_by_nfc_id, PgTransaction, NFC};

#[tokio::test]
async fn test_create_and_get_nfc_successfully() {
    // 1. Set up
    let app = start_test_app().await;

    // 2. create seed data
    let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
        .await
        .expect("Failed to create and save seed asset");

    // 3. Create nfc
    let nfc = NFC::new(asset.id).expect("Failed to load asset");
    let nfc_id = nfc.id.clone();

    let tx: PgTransaction = app
        .db_pool
        .begin()
        .await
        .expect("Failed to start transaction");

    let result = create_nfc(tx, nfc, "user_fp".to_string()).await;

    // 5. Assert:
    // Check that the insertion was successful
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(true));

    // 6. Get created nfc
    let created_nfc = get_nfc_by_id(&nfc_id, &app.db_pool).await;
    assert!(created_nfc.is_ok());
    assert_eq!(created_nfc.unwrap().id, nfc_id);

    // 7. Clean up: drop database that was connected to
    app.drop_db().await
}

#[tokio::test]
async fn test_create_nfc_creates_first_nfc_trail_successfully() {
    let app = start_test_app().await;

    let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
        .await
        .expect("Failed to create and save seed asset");

    let nfc = NFC::new(asset.id).expect("Failed to load asset");

    let nfc_id = nfc.id.clone();
    let tx: PgTransaction = app
        .db_pool
        .begin()
        .await
        .expect("Failed to start transaction");

    let result = create_nfc(tx, nfc, "user_fp".to_string()).await;
    assert!(result.is_ok());
    assert_eq!(result.ok(), Some(true));

    // Get nfc trail
    let trails = get_nfc_trails_by_nfc_id(&nfc_id, &app.db_pool).await;
    assert!(trails.is_ok());
    let trails_list = trails.unwrap();
    assert_eq!(trails_list.len(), 1);
    assert_eq!(trails_list.get(0).unwrap().nfc_id, nfc_id);

    // Clean up: drop database that was connected to
    app.drop_db().await
}
