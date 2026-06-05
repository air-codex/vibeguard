# SPEC: ECC-Inspired Guard Intake

- Date: 2026-06-05
- Status: accepted for design intake; hook behavior remains future work
- Owner: VibeGuard guard/setup maintainers
- Routing: `plan_first` for new guard behavior, `execute_direct` for the setup help regression
- Related files: `hooks/pre-edit-guard.sh`, `hooks/pre-write-guard.sh`, `hooks/pre-bash-guard.sh`, `hooks/manifest.json`, `schemas/hooks-manifest.schema.json`, `scripts/setup/install.sh`, `setup.sh`, `tests/test_setup.sh`

## Goal

Selectively adopt the high-signal defensive ideas from the ECC comparison without changing the VibeGuard Core boundary. VibeGuard should reimplement useful guard behavior through its own hooks, manifests, schemas, and tests instead of installing or stacking external ECC hooks.

## Adopted Subset

### Optional Fact Forcing

GateGuard-style fact forcing is eligible only as a VibeGuard-native guard. The first implementation should be profile-gated behind `strict`, then promoted only after latency and false-positive evidence supports it.

Candidate trigger points:

- first edit to a source file in a session
- first new source file write
- destructive bash commands already routed through `hooks/pre-bash-guard.sh`

The guard must use existing hook entrypoints and `hooks/manifest.json`. It must not add a daemon, telemetry loop, or operator control plane.

### Structured Recovery Contract

Guard and check outputs should converge on a structured recovery shape when they block or warn:

- `status`
- `summary`
- `next_actions`
- `artifacts`
- `root_cause_hint`
- `safe_retry`
- `stop_condition`

The initial contract can be documented and fixture-tested before it is enforced. User-visible missing data or wrong output must fail loudly instead of downgrading to a warning plus fallback.

### Profile Language

Keep the existing `minimal`, `core`, `full`, and `strict` profiles. ECC-style selective-install wording can inform documentation, but VibeGuard must not introduce ECC profile names or imply that ECC is installed. `strict` is the only acceptable first profile for fact-forcing trials.

### Catalog Drift

No catalog/count validation change is required unless this work expands skills, agents, commands, hooks, or schemas. If any of those surfaces are added, extend the relevant manifest and validation tests in the same change.

### External Audit Tools

AgentShield-style scans are allowed only as optional read-only evidence. First trials must run against fixtures or a temporary HOME and report through a doctor/check surface. They must not become default blocking hooks until fixture-backed behavior and review criteria are established.

## Rejected Surfaces

- Do not install ECC globally as part of VibeGuard.
- Do not copy raw ECC hooks JSON into VibeGuard.
- Do not enable ECC continuous-learning or session-observation hooks by default.
- Do not add Hermes/ECC2-style operator control plane work here.
- Do not mutate high-context user config without explicit dry-run, audit, and rollback/receipt behavior.
- Do not add MCP health checks as runtime hooks; keep them as setup/doctor diagnostics first.

## Implementation Plan

1. Fix the current setup help regression so `bash setup.sh --help` prints usage and exits 0.
2. Keep this spec as the committed intake artifact for issue 380.
3. Add a future strict-profile fact-forcing experiment only after its prompt, timeout, suppression, and false-positive policy are specified in fixtures.
4. Add structured recovery contract tests before changing hook output shape.
5. Run latency/perf checks before enabling any new hook gate outside fixtures.

## Verification

Current slice:

- `bash tests/test_setup.sh`
- `bash scripts/ci/validate-doc-paths.sh`
- `bash scripts/ci/validate-doc-command-paths.sh`

Future hook slices:

- focused hook tests for the changed entrypoint
- `bash tests/test_hook_perf_contract.sh`
- `bash scripts/ci/validate-hooks-manifest.sh`
- `bash tests/test_manifest_contract.sh`
