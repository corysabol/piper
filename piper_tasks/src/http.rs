use std::{env, str::FromStr};
use std::collections::HashMap;
use hyper::{body::HttpBody as _, Client, Uri};
use tokio::io::{self, AsyncWriteExt as _};

// AS simple type alias so as to DRY
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(args: &HashMap<String, String>) {

}

pub async fn http_req() -> Result<()> {
    let client = Client::new();
    let res = client.get(Uri::from_static("http://httpbin.org/ip")).await?;

    println!("Status: {}", res.status());
    let buf = hyper::body::to_bytes(res).await?;
    println!("Body: {:?}", buf);

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
pub async fn raw_http_req(args: &HashMap<String, String>) -> Result<()> {
    Ok(())
}

/// fetch_url
pub async fn fetch_url(args: &HashMap<String, String>) -> Result<()> {
    let url: hyper::Uri = hyper::Uri::from_str(
        args.get("url").unwrap()
    ).unwrap();

    let client = Client::new();
    let mut res = client.get(url).await.unwrap();

    // Stream body, writing each chunk to stdout as we get it
    // instead of buffering and printing at the end.
    while let Some(next) = res.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    Ok(())
}
