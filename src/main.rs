//! `pleme-io/kubectl-wait` — typed kubectl wait wrapper.
//!
//! Universal primitive: pod ready, deployment available, CRD established,
//! job complete, custom-resource condition met. Replaces ~10 lines of
//! inline `kubectl wait --for=…` boilerplate per workflow.

use std::process::{Command, Stdio};

use pleme_actions_shared::{ActionError, Input, Output, StepSummary};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Inputs {
    /// Kubernetes resource type, e.g. `pod`, `deployment`, `crd`,
    /// `autoscalingrunnerset.actions.github.com`.
    resource: String,
    /// Either a single name (`my-pod`), label selector
    /// (`-l app=foo`), or `--all`. The action accepts whichever
    /// makes sense for the resource.
    selector: String,
    /// Wait condition, e.g. `condition=Ready`, `condition=Available`,
    /// `delete`, `jsonpath=…`.
    condition: String,
    #[serde(default)]
    namespace: Option<String>,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
    #[serde(default)]
    kubectl_context: Option<String>,
}

fn default_timeout() -> u64 { 300 }

fn main() {
    pleme_actions_shared::log::init();
    if let Err(e) = run() {
        e.emit_to_stdout();
        if e.is_fatal() {
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), ActionError> {
    let inputs = Input::<Inputs>::from_env()?;

    let args = build_kubectl_args(&inputs);
    let stdout = run_kubectl(&args)?;

    let output = Output::from_runner_env()?;
    output.set("output", stdout.trim())?;

    let mut summary = StepSummary::from_runner_env()?;
    summary
        .heading(2, &format!("kubectl wait — {}", inputs.resource))
        .table(
            &["Field", "Value"],
            vec![
                vec!["resource".into(), inputs.resource.clone()],
                vec!["selector".into(), inputs.selector.clone()],
                vec!["condition".into(), inputs.condition.clone()],
                vec![
                    "namespace".into(),
                    inputs.namespace.clone().unwrap_or_else(|| "(cluster-scoped or default)".into()),
                ],
                vec!["timeout".into(), format!("{}s", inputs.timeout_seconds)],
                vec!["status".into(), pleme_actions_shared::summary::status::PASSED.into()],
            ],
        );
    summary.commit()?;

    Ok(())
}

fn build_kubectl_args(inputs: &Inputs) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(ctx) = &inputs.kubectl_context {
        args.push("--context".into());
        args.push(ctx.clone());
    }
    if let Some(ns) = &inputs.namespace {
        args.push("-n".into());
        args.push(ns.clone());
    }
    args.push("wait".into());
    args.push(inputs.resource.clone());

    // Selector: --all / `-l label=value` / single name. Pass through verbatim;
    // the consumer constructs it correctly on the typed input side.
    if inputs.selector == "--all" {
        args.push("--all".into());
    } else if inputs.selector.starts_with("-l ") {
        args.push("-l".into());
        args.push(inputs.selector["-l ".len()..].to_string());
    } else {
        args.push(inputs.selector.clone());
    }

    args.push(format!("--for={}", inputs.condition));
    args.push(format!("--timeout={}s", inputs.timeout_seconds));
    args
}

fn run_kubectl(args: &[String]) -> Result<String, ActionError> {
    let output = Command::new("kubectl")
        .args(args.iter().map(String::as_str))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| ActionError::error(format!("failed to spawn kubectl: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(ActionError::error(format!(
            "kubectl wait exited with status {} (stderr: {})",
            output.status,
            stderr.trim()
        )));
    }
    Ok(stdout.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(resource: &str, selector: &str, condition: &str) -> Inputs {
        Inputs {
            resource: resource.into(),
            selector: selector.into(),
            condition: condition.into(),
            namespace: Some("default".into()),
            timeout_seconds: 60,
            kubectl_context: None,
        }
    }

    #[test]
    fn wait_for_named_pod_ready() {
        let args = build_kubectl_args(&inputs("pod", "my-pod", "condition=Ready"));
        assert_eq!(
            args,
            vec!["-n", "default", "wait", "pod", "my-pod", "--for=condition=Ready", "--timeout=60s"]
        );
    }

    #[test]
    fn wait_with_label_selector() {
        let args = build_kubectl_args(&inputs("pod", "-l app=mybot", "condition=Ready"));
        assert_eq!(
            args,
            vec!["-n", "default", "wait", "pod", "-l", "app=mybot", "--for=condition=Ready", "--timeout=60s"]
        );
    }

    #[test]
    fn wait_with_all_flag() {
        let args = build_kubectl_args(&inputs("pod", "--all", "delete"));
        assert_eq!(
            args,
            vec!["-n", "default", "wait", "pod", "--all", "--for=delete", "--timeout=60s"]
        );
    }

    #[test]
    fn cluster_scoped_omits_namespace() {
        let mut i = inputs("crd", "autoscalingrunnersets.actions.github.com", "condition=Established");
        i.namespace = None;
        let args = build_kubectl_args(&i);
        assert!(!args.contains(&"-n".to_string()));
        assert_eq!(args[0], "wait");
    }

    #[test]
    fn context_arg_prepended() {
        let mut i = inputs("pod", "my-pod", "condition=Ready");
        i.kubectl_context = Some("my-cluster".into());
        let args = build_kubectl_args(&i);
        assert_eq!(args[0], "--context");
        assert_eq!(args[1], "my-cluster");
    }
}
