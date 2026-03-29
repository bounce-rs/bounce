use std::time::Duration;

use fantoccini::{ClientBuilder, Locator};

const APP_URL: &str = "http://localhost:8000";
const WEBDRIVER_URL: &str = "http://localhost:4444";

async fn create_client() -> fantoccini::Client {
    let mut caps = serde_json::Map::new();
    caps.insert(
        "moz:firefoxOptions".to_string(),
        serde_json::json!({"args": ["-headless"]}),
    );
    ClientBuilder::native()
        .capabilities(caps)
        .connect(WEBDRIVER_URL)
        .await
        .expect("failed to connect to WebDriver")
}

async fn get_fetch_count(client: &fantoccini::Client, url_fragment: &str) -> i64 {
    let script = format!(
        r#"return performance.getEntriesByType("resource")
            .filter(e => e.initiatorType === "fetch" && e.name.includes("{}"))
            .length;"#,
        url_fragment
    );
    client
        .execute(&script, vec![])
        .await
        .unwrap()
        .as_i64()
        .unwrap()
}

async fn clear_performance(client: &fantoccini::Client) {
    client
        .execute("performance.clearResourceTimings();", vec![])
        .await
        .unwrap();
}

async fn wait_for_result(client: &fantoccini::Client, selector: &str, data_input: u32) {
    let css = format!("{}[data-input='{}']", selector, data_input);
    client
        .wait()
        .at_most(Duration::from_secs(15))
        .for_element(Locator::Css(&css))
        .await
        .unwrap_or_else(|_| panic!("Timed out waiting for {css} -- query likely stalled"));
}

// --- use_query tests ---

#[tokio::test]
async fn use_query_no_stall_on_input_change() {
    let client = create_client().await;
    client.goto(APP_URL).await.unwrap();

    wait_for_result(&client, "#query-result", 0).await;
    clear_performance(&client).await;

    client
        .find(Locator::Css("#query-next"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#query-result", 1).await;

    let count = get_fetch_count(&client, "/get?n=").await;
    assert_eq!(count, 1, "Expected 1 fetch for new input, got {count}");

    client.close().await.unwrap();
}

#[tokio::test]
async fn use_query_caches_seen_input() {
    let client = create_client().await;
    client.goto(APP_URL).await.unwrap();

    wait_for_result(&client, "#query-result", 0).await;

    client
        .find(Locator::Css("#query-next"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#query-result", 1).await;

    clear_performance(&client).await;

    client
        .find(Locator::Css("#query-prev"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#query-result", 0).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let count = get_fetch_count(&client, "/get?n=").await;
    assert_eq!(count, 0, "Expected 0 fetches for cached input, got {count}");

    client.close().await.unwrap();
}

#[tokio::test]
async fn use_query_no_duplicate_requests() {
    let client = create_client().await;
    client.goto(APP_URL).await.unwrap();

    wait_for_result(&client, "#query-result", 0).await;
    clear_performance(&client).await;

    client
        .find(Locator::Css("#query-next"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#query-result", 1).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let count = get_fetch_count(&client, "/get?n=").await;
    assert_eq!(
        count, 1,
        "Expected exactly 1 fetch (no duplicates), got {count}"
    );

    client.close().await.unwrap();
}

// --- use_prepared_query tests ---

#[tokio::test]
async fn use_prepared_query_no_stall_on_input_change() {
    let client = create_client().await;
    client.goto(&format!("{APP_URL}/#prepared")).await.unwrap();

    wait_for_result(&client, "#prepared-result", 0).await;
    clear_performance(&client).await;

    client
        .find(Locator::Css("#prepared-next"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#prepared-result", 1).await;

    let count = get_fetch_count(&client, "/get?n=").await;
    assert_eq!(count, 1, "Expected 1 fetch for new input, got {count}");

    client.close().await.unwrap();
}

#[tokio::test]
async fn use_prepared_query_caches_seen_input() {
    let client = create_client().await;
    client.goto(&format!("{APP_URL}/#prepared")).await.unwrap();

    wait_for_result(&client, "#prepared-result", 0).await;

    client
        .find(Locator::Css("#prepared-next"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#prepared-result", 1).await;

    clear_performance(&client).await;

    client
        .find(Locator::Css("#prepared-prev"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#prepared-result", 0).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let count = get_fetch_count(&client, "/get?n=").await;
    assert_eq!(count, 0, "Expected 0 fetches for cached input, got {count}");

    client.close().await.unwrap();
}

#[tokio::test]
async fn use_prepared_query_no_duplicate_requests() {
    let client = create_client().await;
    client.goto(&format!("{APP_URL}/#prepared")).await.unwrap();

    wait_for_result(&client, "#prepared-result", 0).await;
    clear_performance(&client).await;

    client
        .find(Locator::Css("#prepared-next"))
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
    wait_for_result(&client, "#prepared-result", 1).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let count = get_fetch_count(&client, "/get?n=").await;
    assert_eq!(
        count, 1,
        "Expected exactly 1 fetch (no duplicates), got {count}"
    );

    client.close().await.unwrap();
}
