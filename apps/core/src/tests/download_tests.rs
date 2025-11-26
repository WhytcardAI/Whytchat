<![CDATA[
#![cfg(test)]
use crate::download_file;
use tempfile::tempdir;
use mockito::mock;

#[tokio::test]
async fn download_fails_on_zero_content_length() {
    let _m = mock("GET", "/empty-file")
        .with_status(200)
        .with_header("Content-Length", "0")
        .with_body("")
        .create();

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("empty-file");
    
    // Create a mock window object. This is tricky without a running Tauri app.
    // We'll create a minimal mock that can handle emits. For this test, we don't need
    // a fully functional window, just something that satisfies the type signature.
    // In a real app, you'd likely have a more robust mocking setup.
    let app = tauri::test::mock_builder().build();
    let window = app.get_window("main").unwrap();


    let result = download_file(&mockito::server_url(), file_path.to_str().unwrap(), &window, 0, 100).await;

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("size was 0"));
    }
}
]]>