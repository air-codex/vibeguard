#!/usr/bin/env bash

vg_resolve_runtime() {
  local repo_dir="${1:?vg_resolve_runtime requires repo dir}"

  if [[ -n "${VIBEGUARD_RUNTIME:-}" ]]; then
    printf '%s\n' "${VIBEGUARD_RUNTIME}"
    return 0
  fi

  local candidates=(
    "${repo_dir}/vibeguard-runtime/target/release/vibeguard-runtime"
    "${repo_dir}/vibeguard-runtime/target/debug/vibeguard-runtime"
  )
  if [[ -n "${HOME:-}" ]]; then
    candidates+=("${HOME}/.vibeguard/installed/bin/vibeguard-runtime")
  fi

  local candidate
  for candidate in "${candidates[@]}"; do
    if [[ -x "${candidate}" ]] && vg_runtime_supports_observe "${candidate}"; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  if command -v vibeguard-runtime >/dev/null 2>&1; then
    candidate="$(command -v vibeguard-runtime)"
    if vg_runtime_supports_observe "${candidate}"; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  fi

  printf '%s\n' "vibeguard-runtime not found. Run cargo build --manifest-path vibeguard-runtime/Cargo.toml or setup.sh." >&2
  return 2
}

vg_runtime_supports_observe() {
  local candidate="$1"
  local output
  output="$("${candidate}" 2>&1 || true)"
  grep -qE '(^|[[:space:]])observe([[:space:]]|$)' <<<"${output}"
}
