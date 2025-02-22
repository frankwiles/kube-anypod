extern crate clap; 
extern crate kube; 
extern crate k8s_openapi; 

use clap::Parser;
use kube::{Api, Client};

#[derive(Debug, PartialEq)]
enum WorkloadType {
    Any,
    Deployment,
    StatefulSet,
    DaemonSet,
}

#[derive(Debug)]
struct ParsedQuery {
    workload_type: WorkloadType,
    name: String,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Config {
    #[clap(short='n', long)]
    namespace: Option<String>,

    #[clap(short='e', long, default_value_t = false)]
    exec: bool,

    query: Option<String>,
}

fn parse_query(query: &str) -> ParsedQuery {
    let parts: Vec<&str> = query.split('/').collect();
    
    match parts.as_slice() {
        [prefix, name] => {
            let workload_type = match *prefix {
                "deployment" => WorkloadType::Deployment,
                "statefulset" => WorkloadType::StatefulSet,
                "daemonset" => WorkloadType::DaemonSet,
                _ => WorkloadType::Any,
            };
            ParsedQuery {
                workload_type,
                name: name.to_string(),
            }
        }
        [name] => ParsedQuery {
            workload_type: WorkloadType::Any,
            name: name.to_string(),
        },
        _ => ParsedQuery {
            workload_type: WorkloadType::Any,
            name: query.to_string(),
        },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    
    if let Some(query) = config.query.as_deref() {
        let parsed = parse_query(query);
        println!("Parsed query: {:#?}", parsed);
    } else {
        println!("No query provided");
    }
    Ok(())
}
