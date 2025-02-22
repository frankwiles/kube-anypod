extern crate clap; 
extern crate kube; 
extern crate k8s_openapi; 

use clap::Parser;
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client, api::ListParams};

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

    #[clap(value_name = "QUERY")]
    query: String,
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

async fn find_matching_pod(
    client: &Client,
    namespace: &str,
    workload_name: &str,
) -> anyhow::Result<Option<String>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default();

    if let Ok(pod_list) = pods.list(&lp).await {
        // Find first running and ready pod that matches the workload name
        for pod in pod_list.items {
            if let Some(pod_name) = pod.metadata.name {
                if pod_name.starts_with(workload_name) {
                    // Check if pod is running and ready
                    if let Some(status) = pod.status {
                        let is_running = status.phase.as_deref() == Some("Running");
                        let is_ready = status.conditions.iter()
                            .flatten()
                            .any(|condition| {
                                condition.type_ == "Ready" && 
                                condition.status == "True"
                            });
                        
                        if is_running && is_ready {
                            return Ok(Some(pod_name));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

async fn find_pod(client: Client, config: &Config, query: ParsedQuery) -> anyhow::Result<Option<String>> {
    let namespace = if let Some(ns) = &config.namespace {
        ns.clone()
    } else {
        let kubeconfig = kube::Config::infer().await?;
        kubeconfig.default_namespace
    };
    let namespace = namespace.as_str();
    let lp = ListParams::default();

    // Helper closure to check workload names
    let matches_query = |name: &str| name.starts_with(&query.name);

    match query.workload_type {
        WorkloadType::Deployment | WorkloadType::Any => {
            let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = deployments.list(&lp).await {
                if let Some(deployment) = list.items
                    .iter()
                    .find(|d| d.metadata.name.as_ref().map_or(false, matches_query)) {
                    if let Some(name) = &deployment.metadata.name {
                        if let Some(pod_name) = find_matching_pod(&client, namespace, name).await? {
                            return Ok(Some(pod_name));
                        }
                    }
                }
            }
            if query.workload_type != WorkloadType::Any {
                return Ok(None);
            }
        }

        WorkloadType::StatefulSet | WorkloadType::Any => {
            let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = statefulsets.list(&lp).await {
                if let Some(statefulset) = list.items
                    .iter()
                    .find(|ss| ss.metadata.name.as_ref().map_or(false, matches_query)) {
                    if let Some(name) = &statefulset.metadata.name {
                        if let Some(pod_name) = find_matching_pod(&client, namespace, name).await? {
                            return Ok(Some(pod_name));
                        }
                    }
                }
            }
            if query.workload_type != WorkloadType::Any {
                return Ok(None);
            }
        }

        WorkloadType::DaemonSet | WorkloadType::Any => {
            let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = daemonsets.list(&lp).await {
                if let Some(daemonset) = list.items
                    .iter()
                    .find(|ds| ds.metadata.name.as_ref().map_or(false, matches_query)) {
                    if let Some(name) = &daemonset.metadata.name {
                        if let Some(pod_name) = find_matching_pod(&client, namespace, name).await? {
                            return Ok(Some(pod_name));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let client = Client::try_default().await?;
    
    let parsed = parse_query(&config.query);
    match find_pod(client, &config, parsed).await? {
        Some(pod_name) => println!("{}", pod_name),
        None => println!("No matching pods found"),
    }
    Ok(())
}
