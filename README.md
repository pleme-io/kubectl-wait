# pleme-io/kubectl-wait

Typed `kubectl wait` wrapper. Universal primitive for any K8s wait condition.

```yaml
- uses: pleme-io/kubectl-wait@v1
  with:
    resource: deployment
    selector: my-app
    condition: condition=Available
    namespace: production
    timeout-seconds: 300
```

```yaml
- uses: pleme-io/kubectl-wait@v1
  with:
    resource: pod
    selector: -l app.kubernetes.io/component=runner-scale-set-listener
    condition: condition=Ready
    namespace: arc-rio-default
```

```yaml
- uses: pleme-io/kubectl-wait@v1
  with:
    resource: crd
    selector: autoscalingrunnersets.actions.github.com
    condition: condition=Established
```

## Inputs

| Name | Required | Default | Description |
|---|---|---|---|
| `resource` | yes | — | `pod` / `deployment` / `crd` / `autoscalingrunnerset.actions.github.com` |
| `selector` | yes | — | Single name, `-l label=value`, or `--all` |
| `condition` | yes | — | `condition=Ready`, `condition=Available`, `delete`, `jsonpath=...` |
| `namespace` | no | — | Omit for cluster-scoped |
| `timeout-seconds` | no | `300` | |
| `kubectl-context` | no | — | |

## Outputs

| Name | Description |
|---|---|
| `output` | kubectl wait stdout (typically `condition met`) |
