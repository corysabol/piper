use serde_json::Value;

pub async fn notify(uri: String, json: Value) -> Result<(), anyhow::Error> {
    // Create a new hyper client
    // Convert the JSON blob to a string
    let json_string = match serde_json::to_string(&json) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            anyhow::bail!("Error serializing JSON: {}", e);
        }
    };
    let client = reqwest::Client::new();
    let _resp = client.post(uri).body(json_string).send().await?;
    Ok(())
}
