extern crate clap; 
extern crate kube; 
extern crate k8s_openapi; 

use clap::Parser;
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client, api::{ListParams, ObjectMeta}};

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

async fn find_pod(client: Client, config: &Config, query: ParsedQuery) -> anyhow::Result<Option<String>> {
    let namespace = config.namespace.as_deref().unwrap_or("default");
    let lp = ListParams::default();

    // Only look at specific workload type if specified
    if query.workload_type != WorkloadType::Any {
        match query.workload_type {
            WorkloadType::Deployment => {
                let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
                if let Ok(list) = deployments.list(&lp).await {
                    for d in list.items {
                        if let Some(name) = d.metadata.name {
                            if name.starts_with(&query.name) {
                                // Get pods for this deployment
                                let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
                                if let Ok(pod_list) = pods.list(&lp).await {
                                    for pod in pod_list.items {
                                        if let Some(pod_name) = pod.metadata.name {
                                            if pod_name.starts_with(&name) {
                                                return Ok(Some(pod_name));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            WorkloadType::StatefulSet => {
                let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
                if let Ok(list) = statefulsets.list(&lp).await {
                    for ss in list.items {
                        if let Some(name) = ss.metadata.name {
                            if name.starts_with(&query.name) {
                                // Get pods for this statefulset
                                let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
                                if let Ok(pod_list) = pods.list(&lp).await {
                                    for pod in pod_list.items {
                                        if let Some(pod_name) = pod.metadata.name {
                                            if pod_name.starts_with(&name) {
                                                return Ok(Some(pod_name));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            WorkloadType::DaemonSet => {
                let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
                if let Ok(list) = daemonsets.list(&lp).await {
                    for ds in list.items {
                        if let Some(name) = ds.metadata.name {
                            if name.starts_with(&query.name) {
                                // Get pods for this daemonset
                                let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
                                if let Ok(pod_list) = pods.list(&lp).await {
                                    for pod in pod_list.items {
                                        if let Some(pod_name) = pod.metadata.name {
                                            if pod_name.starts_with(&name) {
                                                return Ok(Some(pod_name));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            WorkloadType::Any => unreachable!(),
        }
        return Ok(None);
    }

    // Look through workload types in preferred order
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    if let Ok(list) = deployments.list(&lp).await {
        for d in list.items {
            if let Some(name) = d.metadata.name {
                if name.starts_with(&query.name) {
                    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
                    if let Ok(pod_list) = pods.list(&lp).await {
                        for pod in pod_list.items {
                            if let Some(pod_name) = pod.metadata.name {
                                if pod_name.starts_with(&name) {
                                    return Ok(Some(pod_name));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    if let Ok(list) = statefulsets.list(&lp).await {
        for ss in list.items {
            if let Some(name) = ss.metadata.name {
                if name.starts_with(&query.name) {
                    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
                    if let Ok(pod_list) = pods.list(&lp).await {
                        for pod in pod_list.items {
                            if let Some(pod_name) = pod.metadata.name {
                                if pod_name.starts_with(&name) {
                                    return Ok(Some(pod_name));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
    if let Ok(list) = daemonsets.list(&lp).await {
        for ds in list.items {
            if let Some(name) = ds.metadata.name {
                if name.starts_with(&query.name) {
                    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
                    if let Ok(pod_list) = pods.list(&lp).await {
                        for pod in pod_list.items {
                            if let Some(pod_name) = pod.metadata.name {
                                if pod_name.starts_with(&name) {
                                    return Ok(Some(pod_name));
                                }
                            }
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
    
    if let Some(query) = config.query.as_deref() {
        let parsed = parse_query(query);
        match find_pod(client, &config, parsed).await? {
            Some(pod_name) => println!("{}", pod_name),
            None => println!("No matching pods found"),
        }
    } else {
        println!("No query provided");
    }
    Ok(())
}
