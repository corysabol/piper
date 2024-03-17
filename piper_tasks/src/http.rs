use std::collections::HashMap;

use anyhow::Ok;

// Issues a simple HTTP GET request.
pub async fn http_get(args: &HashMap<String, String>) -> Result<(), anyhow::Error> {
    let res = reqwest::get("http://httpbin.org/ip").await?;

    println!("Status: {}", res.status());
    println!("body = {res:?}");
    Ok(())
}

// Issues a simple HTTP POST request.
pub async fn http_post(args: &HashMap<String, String>) -> Result<(), anyhow::Error> {
    Ok(())
}

/// Issues a raw HTTP request
///
/// # Arguments
///
/// * `args` - A ref to a HashMap containing String args for this task function.
///
/// # Examples
///
/// Example task args in yaml
///
/// - name: POST to /foo/bar
///   task: raw_http_req
///   args:
///     https: false
///     host: foo.bar
///     req: |
///         POST /foo/bar HTTP/2
///         Host: foo.bar
///         Content-Type: application/json
///
///         {"foo":"bar"}
pub async fn raw_http_req(args: &HashMap<String, String>) -> Result<(), anyhow::Error> {
    Ok(())
}
