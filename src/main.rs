extern crate clap; 
extern crate kube; 
extern crate k8s_openapi; 

use clap::Parser;
use kube::{Api, Client};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Config {
    #[clap(short='n', long)]
    namespace: Option<String>,

    #[clap(short='e', long, default_value_t = false)]
    exec: bool,

    query: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse(); 
    println!("Hello, world!");

    println!("{:#?}", config); 
    Ok(())
}
