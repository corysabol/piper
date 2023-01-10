use std::net::*;
use std::collections::HashMap;
use std::path::*;
use tokio::runtime::Runtime;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::*;

/*
A somewhat high-speed subdomain bruteforcer.

Currently it is very basic in functionality.

Techniques:
    1. blast through top1000 subdomains, etc from sources like SecLists doing DNS lookups

        Basic algorithm:
        results = []
        for name do
            for sub do
                address = lookup(sub+name)
                if address
                    results.push(sub+name)
        return results

    2. fetch HTTP content of top level hosts and scan for subdomains
        2a. Recursive mode will try to HTTP req of all newly discovered subs and process the content for new subs
    3. Crawl internet sources (crtsh, alienvault, etc)
*/

pub struct ScanOpts {
    names: Vec::<String>,
    resolver_ip: std::net::IpAddr,
    dns_brute: bool,
    wordlists: Vec::<PathBuf>,
    content_scan: bool,
    source_scan: bool,
    sources: Vec::<String> // valid options: crtsh, alienvault, TODO determine sources
}

pub struct ScanResults {
    discovered_subs: Vec::<std::net::IpAddr>,
}

pub fn run(_args: &HashMap<String, String>) {
    println!("Not implemented");
}

pub fn scan(options: ScanOpts) {
    // Start with the simplest case of using a wordlist
    /*match options {
        ScanOpts { dns_brute: true, .. } => dns_lookup(options.names).unwrap(),
        _: => Ok(())
    }*/

    let mut results = ScanResults {
        discovered_subs: Vec::<std::net::IpAddr>::new(),
    };

    if options.dns_brute == true {
        results.discovered_subs.append(&mut dns_lookup(options.names).unwrap());
    }
}

// TODO: updated this to be async using tokio and to process a list of names
pub fn dns_lookup(names: Vec<String>) -> Result<Vec::<std::net::IpAddr>, Box<dyn std::error::Error>> {
    let mut io_loop = Runtime::new().unwrap();

    let resolver = io_loop.block_on(async {
        TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default()
        )
    }).expect("failed to connect resolver");

    // TODO use tokio to async generate lookup tasks
    //let name = "www.secureideas.com";
    let name = &names[0]; 
    let lookup_future = resolver.lookup_ip(name.to_string());
    let mut response = io_loop.block_on(lookup_future).unwrap();

    // There can be many IP addresses associated with a name.
    // This can return IPv4 and/or IPv6 addresses
    //let address: std::net::IpAddr = response.iter().next().expect("no addresses returned!");
    let addresses: Vec::<std::net::IpAddr> = response.iter().collect();
    Ok(addresses)
} 

// Fetch and search HTML content for more subdomains
fn search_html(name: String) {

}

// Search sources like crtsh for subdomains
fn search_sources(name: String) {

}

pub fn test() {
    println!("This is a module / workspace test!");
}
