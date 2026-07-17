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

## Observability: the report carries real values (resolved in `f1644fe`)

Early runs wrote `{"tier1":{"fixed":0,"gate_result":false}}` on every run: the
`run.campaign.sweep-report` script had assumed a `PRAXEC_CTX_*` / `PRAXEC_RUN_ID`
env contract that does not exist (`praxec` injects only `PRAXEC_SCRIPT_HASH`
and `PRAXEC_SCRIPT_SUBJECT`). Commit `f1644fe` wired the run context through
the `reporting` state's executor via templated `args:`, which the report
script reads positionally — so the report now carries the real run values.

Two praxec facts confirmed while fixing it:
- `workingDirectory:` sets the process **cwd** (it does not consume an argv
  slot); `args:` entries become argv[1], argv[2], … in order.
- Executor templating uses **jsonpath `$.context.x` syntax, not Handlebars
  `{{ }}`** — a literal `{{$.context.x}}` passes through unresolved (the praxec
  spec's brace example is misleading). No run-id is templatable, so the report
  stamp uses a UTC wall-clock value.

A verified run now produces:
`{"tier":1,"findings_in":{"SOS027":1},"tier1":{"fixed":1,"gate_result":true},"unknown_codes":[]}`.

## Cleanup

```bash
rm -rf "$WORK"
```

Confirm no residue leaked into the pack repo itself:

```bash
git branch --list "campaign/*"   # must be empty
ls .praxec 2>&1                  # must not exist
```
