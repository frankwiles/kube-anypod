#[derive(Debug, PartialEq)]
pub enum WorkloadType {
    Any,
    Deployment,
    StatefulSet,
    DaemonSet,
}

#[derive(Debug)]
pub struct ParsedQuery {
    pub workload_type: WorkloadType,
    pub name: String,
}

pub fn parse_query(query: &str) -> ParsedQuery {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_simple_name() {
        let result = parse_query("nginx");
        assert_eq!(result.workload_type, WorkloadType::Any);
        assert_eq!(result.name, "nginx");
    }

    #[test]
    fn test_parse_query_deployment() {
        let result = parse_query("deployment/nginx");
        assert_eq!(result.workload_type, WorkloadType::Deployment);
        assert_eq!(result.name, "nginx");
    }

    #[test]
    fn test_parse_query_statefulset() {
        let result = parse_query("statefulset/postgresql");
        assert_eq!(result.workload_type, WorkloadType::StatefulSet);
        assert_eq!(result.name, "postgresql");
    }

    #[test]
    fn test_parse_query_daemonset() {
        let result = parse_query("daemonset/fluentd");
        assert_eq!(result.workload_type, WorkloadType::DaemonSet);
        assert_eq!(result.name, "fluentd");
    }

    #[test]
    fn test_parse_query_unknown_prefix() {
        let result = parse_query("unknown/nginx");
        assert_eq!(result.workload_type, WorkloadType::Any);
        assert_eq!(result.name, "nginx");
    }

    #[test]
    fn test_parse_query_empty_string() {
        let result = parse_query("");
        assert_eq!(result.workload_type, WorkloadType::Any);
        assert_eq!(result.name, "");
    }

    #[test]
    fn test_parse_query_multiple_slashes() {
        let result = parse_query("deployment/nginx/extra");
        assert_eq!(result.workload_type, WorkloadType::Any);
        assert_eq!(result.name, "deployment/nginx/extra");
    }

    #[test]
    fn test_parse_query_with_spaces() {
        let result = parse_query("deployment/nginx app");
        assert_eq!(result.workload_type, WorkloadType::Deployment);
        assert_eq!(result.name, "nginx app");
    }

    #[test]
    fn test_parse_query_case_sensitive() {
        let result = parse_query("DEPLOYMENT/nginx");
        assert_eq!(result.workload_type, WorkloadType::Any);
        assert_eq!(result.name, "nginx");
    }
}
