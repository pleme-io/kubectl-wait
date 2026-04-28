{
  description = "pleme-io/kubectl-wait — typed kubectl wait wrapper";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    crate2nix = { url = "github:nix-community/crate2nix"; inputs.nixpkgs.follows = "nixpkgs"; };
    flake-utils.url = "github:numtide/flake-utils";
    substrate = { url = "github:pleme-io/substrate"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = inputs @ { self, nixpkgs, crate2nix, flake-utils, substrate, ... }:
    (import "${substrate}/lib/rust-action-release-flake.nix" {
      inherit nixpkgs crate2nix flake-utils;
    }) {
      toolName = "kubectl-wait";
      src = self;
      repo = "pleme-io/kubectl-wait";
      action = {
        description = "Typed kubectl wait wrapper. Resource + selector (name / -l label= / --all) + condition + namespace + timeout. Universal primitive for waiting on pod ready, deployment available, CRD established, job complete, etc.";
        inputs = [
          { name = "resource"; description = "Resource type (pod / deployment / crd / autoscalingrunnerset.actions.github.com)"; required = true; }
          { name = "selector"; description = "Single name, '-l label=value' selector, or '--all'"; required = true; }
          { name = "condition"; description = "Wait condition (condition=Ready, condition=Available, delete, jsonpath=...)"; required = true; }
          { name = "namespace"; description = "Namespace; omit for cluster-scoped resources"; }
          { name = "timeout-seconds"; description = "Timeout in seconds"; default = "300"; }
          { name = "kubectl-context"; description = "kubectl context"; }
        ];
        outputs = [
          { name = "output"; description = "kubectl wait stdout (e.g. 'condition met')"; }
        ];
      };
    };
}
