use crate::queries::suit::{run_test_async, TestError};
use crate::seed::create_and_save_contract;
use xrf1::core::queries;

#[tokio::test]
async fn test_nfc_is_created_when_asset_is_created_successfully() {
    run_test_async(|app| async move {
        // 2. create seed data
        let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
            .await
            .expect("Failed to create and save seed asset");

        // 3. Create nfc
        let nfc = queries::get_nfc_by_asset_id(&asset.id, &app.db_pool)
            .await
            .expect("Failed to get nfc");
        let nfc_id = nfc.id.clone();

        // Assert
        // 4. Get created nfc
        let created_nfc = queries::get_nfc_by_id(&nfc_id, &app.db_pool).await;
        assert!(created_nfc.is_ok());
        assert_eq!(created_nfc.unwrap().id, nfc_id);

        Ok::<_, TestError>(())
    }).await
}

#[tokio::test]
async fn test_create_nfc_creates_first_nfc_trail_successfully() {
    run_test_async(|app| async move {
        let asset = create_and_save_contract(app.user_fp.clone(), &app.db_pool)
            .await
            .expect("Failed to create and save seed asset");

        let nfc = queries::get_nfc_by_asset_id(&asset.id, &app.db_pool)
            .await
            .expect("Failed to get nfc");

        let nfc_id = nfc.id.clone();

        // Get nfc trail
        let trails = queries::get_nfc_trails_by_nfc_id(&nfc_id, &app.db_pool).await;
        assert!(trails.is_ok());
        let trails_list = trails.unwrap();
        assert_eq!(trails_list.len(), 1);
        assert_eq!(trails_list.get(0).unwrap().nfc_id, nfc_id);

        Ok::<_, TestError>(())
    }).await
}
