# Tier-1 sweep-safe E2E (go/no-go)

Requires `praxec` + `structureos-mcp` on `PATH` and a checkout of
`cognitive-architectures` beside this repo. The mutating run happens in an
ISOLATED throwaway copy so it never touches the pack repo.

## Requirements

- `praxec` (this procedure was verified against `praxec 0.0.24`).
- `cargo`/`rustc` on `PATH` (the flow's `detect.safe` / `fix.safe` /
  `verify.build` seam scripts shell out to `cargo build` / `cargo fix` /
  `cargo fmt` / `cargo clippy` / `cargo test`).
- `../cognitive-architectures` beside this repo, supplying
  `verify.cargo.cwd` (`scripts-library/verify.cargo.cwd.yaml`).
- `jq` on `PATH` (used by the campaign scripts to JSON-encode script output).

## Procedure

1. Copy the fixture to a throwaway git repo:

   ```bash
   WORK=$(mktemp -d)
   cp -r tests/fixtures/rust-findings/. "$WORK/"
   rm -rf "$WORK/target"   # drop any stale build cache copied from the fixture
   git -C "$WORK" init -q
   ```

2. **Declare `$WORK` as a writable repo with a minimal manifest.** A `praxec`
   run requires `$.run.repo_root`, which resolves from a `repos:` entry
   marked `writable: true` — and loading a declared repo requires a
   `praxec.repo.yaml` manifest at its root (this is a real, load-bearing
   requirement discovered while executing this procedure, not an artifact of
   the fixture):

   ```bash
   cat > "$WORK/praxec.repo.yaml" <<'EOF'
   schema: praxec.repo/v1
   name: rust-findings-fixture
   namespace: e2e
   version: 0.0.0
   description: Throwaway E2E fixture repo; ships no capabilities/flows of its own.
   EOF
   git -C "$WORK" add -A
   git -C "$WORK" -c user.email=e2e@local -c user.name=e2e commit -qm init
   ```

3. **Build a driver config.** `examples/campaign-check.yaml` has no `store:`
   key, i.e. the default in-memory store — fine for `praxec check`, but a
   `praxec command` call against it cannot durably persist a workflow
   instance across separate CLI invocations (each invocation is a fresh
   process). Two options, both verified:

   - **(used here)** Layer a throwaway config *outside* the pack repo that
     `include`s the same three files by absolute path, adds a `store: {
     kind: sqlite, path: ... }`, and declares the `repos:` entry from step 2:

     ```yaml
     # e2e-gateway.yaml (throwaway; NOT part of this repo)
     include:
       - /abs/path/to/frontrails-praxec-pack/frontrails.yaml
       - /abs/path/to/frontrails-praxec-pack/frontrails-campaign.yaml
       - /abs/path/to/frontrails-praxec-pack/extensions/rust.yaml
       - /abs/path/to/cognitive-architectures/scripts-library/verify.cargo.cwd.yaml
     store:
       kind: sqlite
       path: /abs/path/to/throwaway/e2e-store.db
     repos:
       - path: "$WORK"
         writable: true
     ```

   - **(alternative, not exercised here)** Keep `examples/campaign-check.yaml`
     verbatim and drive it via `praxec serve --config "$CFG"` plus the
     `praxec.command` MCP tool inside one long-lived session — the in-memory
     store is fine as long as the whole drive happens inside one process.
     Use this if you cannot add a `repos:`/`store:` overlay.

   Either way: `CFG="$PWD/examples/campaign-check.yaml"` must still pass
   structurally on its own — validate the *actual* shipped config first:

   ```bash
   CFG="$PWD/examples/campaign-check.yaml"
   praxec check --config "$CFG"   # exit 0 — validates the real pack config
   ```

   then validate the throwaway driver overlay:

   ```bash
   DRIVER_CFG=/abs/path/to/throwaway/e2e-gateway.yaml
   praxec check --config "$DRIVER_CFG"   # exit 0
   ```

4. Start the flow with `$WORK` as `cwd` (so `repo_root` resolves there):

   ```bash
   cd "$WORK"
   praxec command --config "$DRIVER_CFG" \
     '{"definitionId":"flow.findings.sweep-safe","input":{}}'
   ```

   Every transition in `flow.findings.sweep-safe` is `actor: deterministic`
   (the only `actor: human` state, `needs_human`, is reached only on a red
   build). Deterministic transitions auto-chain, so **one `command` call
   drives the whole flow to a terminal state** — no follow-up `submit` calls
   were needed in this run. Read the returned `chain` / `workflow.state` to
   confirm; if a run ever stops short of `done`/`needs_human`, follow the
   response's `links` with `expectedVersion` to continue.

## Acceptance (all assertions against $WORK)

```bash
grep -q "use std::collections::HashMap;" "$WORK/src/lib.rs" && echo FAIL_IMPORT || echo OK_IMPORT
ls "$WORK"/.praxec/reports/sweep-safe-*.json >/dev/null 2>&1 && echo OK_REPORT || echo FAIL_REPORT
git -C "$WORK" rev-parse --verify campaign/sweep-safe >/dev/null 2>&1 && echo OK_BRANCH || echo FAIL_BRANCH
```

Expected: `OK_IMPORT`, `OK_REPORT`, `OK_BRANCH`. Also confirm the workflow's
final `state` is `done` (not `needs_human`) and `result.status` is
`succeeded` in the `praxec command` response.

## Known gap: the report's `tier1.fixed` / findings counts read as 0

The flow itself is sound end-to-end (import removed, build gate green,
`done`, branch committed). But `run.campaign.sweep-report`
(`frontrails-campaign.yaml`) reads its inputs from `PRAXEC_CTX_finding_count`
/ `PRAXEC_CTX_fixed_count` / `PRAXEC_CTX_build_passed` / `PRAXEC_RUN_ID`
environment variables with a `:-0`/`:-false` fallback — **that env-var
contract does not exist**. Verified by inspecting the script executor's
actual environment (`env | sort` from inside a script body): `praxec` injects
exactly two script-scoped variables, `PRAXEC_SCRIPT_HASH` and
`PRAXEC_SCRIPT_SUBJECT` — no `PRAXEC_CTX_*`, no `PRAXEC_RUN_ID`. Everything
else in the child environment is just the parent process's inherited
environment.

Per `docs/reference/spec.md` in `praxec-kernel`, the real `kind: script`
contract for passing workflow context into a script is explicit, not
ambient:

```yaml
executor:
  kind: script
  subject: run.campaign.sweep-report
  workingDirectory: "$.run.repo_root"
  args: ["{{$.context.finding_count}}"]      # templated argv
  env: { PRAXEC_CTX_fixed_count: "{{$.context.fixed_count}}" }  # templated env
```

`flow.findings.sweep-safe`'s `reporting` state has neither an `args:` nor an
`env:` block on the `report` transition's executor today, so the script has
no way to see `$.context.finding_count` / `fixed_count` / `build_passed` —
it silently falls back to `0`/`0`/`false` every time, even on a real fix.
The resulting report JSON is therefore always
`{"tier1":{"fixed":0,"gate_result":"false"}}` regardless of the actual run —
a silent, always-wrong observability report. **This is a real defect to fix
in `frontrails-campaign.yaml`'s `reporting` state (wire `env:`/`args:` on the
`report` transition), not a gap in this test procedure**; recorded here as
the go/no-go signal for the report-contract itself.

## Cleanup

```bash
rm -rf "$WORK"
```

Confirm no residue leaked into the pack repo itself:

```bash
git branch --list "campaign/*"   # must be empty
ls .praxec 2>&1                  # must not exist
```
