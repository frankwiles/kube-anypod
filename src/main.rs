extern crate clap;
extern crate k8s_openapi;
extern crate kube;

use anypod::{parse_query, ParsedQuery, WorkloadType};
use clap::Parser;
use colorful::Colorful;
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, StatefulSet};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client, api::ListParams};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Config {
    #[clap(short = 'n', long)]
    namespace: Option<String>,

    #[clap(short = 'e', long, default_value_t = false)]
    exec: bool,

    #[clap(value_name = "QUERY")]
    query: String,
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
                        let is_ready = status.conditions.iter().flatten().any(|condition| {
                            condition.type_ == "Ready" && condition.status == "True"
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

async fn show_available_workloads(client: &Client, namespace: &str) -> anyhow::Result<()> {
    let lp = ListParams::default();

    // Show Deployments
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    if let Ok(list) = deployments.list(&lp).await {
        if !list.items.is_empty() {
            println!("{}", "Deployments:".blue().bold());
            for d in list.items {
                if let Some(name) = d.metadata.name {
                    println!("  {}", name.light_green());
                }
            }
            println!();
        }
    }

    // Show StatefulSets
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    if let Ok(list) = statefulsets.list(&lp).await {
        if !list.items.is_empty() {
            println!("{}", "StatefulSets:".blue().bold());
            for ss in list.items {
                if let Some(name) = ss.metadata.name {
                    println!("  {}", name.light_green());
                }
            }
            println!();
        }
    }

    // Show DaemonSets
    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
    if let Ok(list) = daemonsets.list(&lp).await {
        if !list.items.is_empty() {
            println!("{}", "DaemonSets:".blue().bold());
            for ds in list.items {
                if let Some(name) = ds.metadata.name {
                    println!("  {}", name.light_green());
                }
            }
            println!();
        }
    }

    Ok(())
}

async fn find_pod(
    client: Client,
    config: &Config,
    query: ParsedQuery,
) -> anyhow::Result<Option<String>> {
    let namespace = if let Some(ns) = &config.namespace {
        ns.clone()
    } else {
        let kubeconfig = kube::Config::infer().await?;
        kubeconfig.default_namespace
    };
    let namespace = namespace.as_str();
    let lp = ListParams::default();

    // Handle specific workload type requests first
    match query.workload_type {
        WorkloadType::Deployment => {
            let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = deployments.list(&lp).await {
                if let Some(deployment) = list.items.iter().find(|d| {
                    d.metadata
                        .name
                        .as_ref()
                        .map_or(false, |name| name.starts_with(&query.name))
                }) {
                    if let Some(name) = &deployment.metadata.name {
                        return find_matching_pod(&client, namespace, name).await;
                    }
                }
            }
            return Ok(None);
        }
        WorkloadType::StatefulSet => {
            let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = statefulsets.list(&lp).await {
                if let Some(statefulset) = list.items.iter().find(|ss| {
                    ss.metadata
                        .name
                        .as_ref()
                        .map_or(false, |name| name.starts_with(&query.name))
                }) {
                    if let Some(name) = &statefulset.metadata.name {
                        return find_matching_pod(&client, namespace, name).await;
                    }
                }
            }
            return Ok(None);
        }
        WorkloadType::DaemonSet => {
            let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = daemonsets.list(&lp).await {
                if let Some(daemonset) = list.items.iter().find(|ds| {
                    ds.metadata
                        .name
                        .as_ref()
                        .map_or(false, |name| name.starts_with(&query.name))
                }) {
                    if let Some(name) = &daemonset.metadata.name {
                        return find_matching_pod(&client, namespace, name).await;
                    }
                }
            }
            return Ok(None);
        }
        WorkloadType::Any => {
            // Try each type in order
            let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = deployments.list(&lp).await {
                if let Some(deployment) = list.items.iter().find(|d| {
                    d.metadata
                        .name
                        .as_ref()
                        .map_or(false, |name| name.starts_with(&query.name))
                }) {
                    if let Some(name) = &deployment.metadata.name {
                        if let Some(pod_name) = find_matching_pod(&client, namespace, name).await? {
                            return Ok(Some(pod_name));
                        }
                    }
                }
            }

            let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = statefulsets.list(&lp).await {
                if let Some(statefulset) = list.items.iter().find(|ss| {
                    ss.metadata
                        .name
                        .as_ref()
                        .map_or(false, |name| name.starts_with(&query.name))
                }) {
                    if let Some(name) = &statefulset.metadata.name {
                        if let Some(pod_name) = find_matching_pod(&client, namespace, name).await? {
                            return Ok(Some(pod_name));
                        }
                    }
                }
            }

            let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
            if let Ok(list) = daemonsets.list(&lp).await {
                if let Some(daemonset) = list.items.iter().find(|ds| {
                    ds.metadata
                        .name
                        .as_ref()
                        .map_or(false, |name| name.starts_with(&query.name))
                }) {
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
    let namespace = if let Some(ns) = &config.namespace {
        ns.clone()
    } else {
        let kubeconfig = kube::Config::infer().await?;
        kubeconfig.default_namespace
    };
    let namespace = namespace.as_str();

    match find_pod(client.clone(), &config, parsed).await? {
        Some(pod_name) => {
            if config.exec {
                // Build kubectl command
                let mut cmd = Command::new("kubectl");

                if let Some(ns) = &config.namespace {
                    cmd.args(["-n", ns]);
                }

                cmd.args(["exec", "-it", &pod_name, "--", "/bin/bash"])
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());

                // Execute kubectl
                match cmd.status() {
                    Ok(_) => (),
                    Err(e) => println!("Failed to execute kubectl: {}", e),
                }
            } else {
                println!("{}", pod_name);
            }
        }
        None => {
            println!(
                "No matching pods found in namespace '{}'!",
                namespace.blue()
            );
            println!();
            println!("Here are the workloads that exist in this namespace.");
            println!();
            show_available_workloads(&client, namespace).await?;
        }
    }
    Ok(())
}
