# Tier-2 structural E2E (go/no-go)

Requires `praxec` + `structureos-mcp` on `PATH`, sibling `cognitive-architectures`
+ `cognitive-architectures-max` checkouts, and (to complete a real decomposition)
a coding agent wired — NOT needed for this go/no-go, which proves selection +
dispatch + budget + the zero-god-file edge only. The full decomposition is the
referenced `flow.refactor.god-file`'s job (already dogfooded elsewhere) — not
re-proven here.

- `praxec` (this procedure was verified against `praxec 0.0.24`).
- `jq` on `PATH` (used by the campaign scripts and by this procedure's assertions).
- `../cognitive-architectures` and `../cognitive-architectures-max` beside this
  repo (supply `verify.cargo.cwd` and `flow.refactor.god-file` respectively).
- Every mutating/driving run happens in an ISOLATED throwaway copy (`mktemp -d`
  + fresh `git init`) so it never touches the pack repo.

## Procedure

1. Copy a fixture to a throwaway git repo and declare it as a writable praxec
   repo (a `praxec.repo.yaml` manifest is required at the repo root — same
   requirement documented in `tests/campaign-tier1-e2e.md`):

   ```bash
   WORK=$(mktemp -d)
   cp -r tests/fixtures/rust-godfile/. "$WORK/"   # or rust-findings for the zero-count case
   rm -rf "$WORK/target" "$WORK/.frontrails/structure/cache"
   git -C "$WORK" init -q
   cat > "$WORK/praxec.repo.yaml" <<'EOF'
   schema: praxec.repo/v1
   name: rust-godfile-fixture
   namespace: e2e
   version: 0.0.0
   description: Throwaway E2E fixture repo; ships no capabilities/flows of its own.
   EOF
   git -C "$WORK" add -A
   git -C "$WORK" -c user.email=e2e@local -c user.name=e2e commit -qm init
   ```

2. Structure check the real shipped config first:

   ```bash
   CFG="$PWD/examples/campaign-check.yaml"
   praxec check --config "$CFG"   # exit 0
   ```

3. **Build a driver config.** `examples/campaign-check.yaml` has no `store:`
   (in-memory — fine for `check`, not durable across separate CLI invocations).
   Layer a throwaway config *outside* the pack repo that `include`s the same
   files by absolute path, adds a `store: { kind: sqlite, path: ... }`, and
   declares the throwaway `repos:` entry from step 1:

   ```yaml
   # e2e-gateway.yaml (throwaway; NOT part of this repo)
   include:
     - /abs/path/to/frontrails-praxec-pack/frontrails.yaml
     - /abs/path/to/frontrails-praxec-pack/frontrails-campaign.yaml
     - /abs/path/to/frontrails-praxec-pack/extensions/rust.yaml
     - /abs/path/to/cognitive-architectures/scripts-library/verify.cargo.cwd.yaml
     - /abs/path/to/cognitive-architectures-max/orchestrators/flow.refactor.god-file.yaml
   store:
     kind: sqlite
     path: /abs/path/to/throwaway/e2e-store.db
   repos:
     - path: "$WORK"
       writable: true
   ```

   Adding a `repos:` entry here is ONLY for `$WORK` (the throwaway fixture) —
   it does not change how `cognitive-architectures-max` resolves (it is still
   raw-`include`d, so `flow.refactor.god-file` stays unprefixed; see
   `frontrails-campaign.yaml`'s `refactoring` state comment for why the
   prefixed `cognitive-max/flow.refactor.god-file` form is NOT the one
   actually loaded here).

   ```bash
   praxec check --config /abs/path/to/throwaway/e2e-gateway.yaml   # exit 0
   ```

4. Drive with `$WORK` as `cwd` (so `$.run.repo_root` resolves there):

   ```bash
   cd "$WORK"
   praxec command --config "$DRIVER_CFG" \
     '{"definitionId":"flow.findings.structural","input":{"budget":1}}'
   ```

   Every transition is `actor: deterministic` except the referenced flow's
   human gate, so the parent chains all the way to `refactoring`'s `decompose`
   transition, which dispatches the child and then legitimately PARKS
   (`result.status: "waiting"`, a `pending_human` block naming the child
   workflow id) once the child reaches ITS first human gate (`reviewing` /
   `approve`). That parked state — not a completed decomposition — is the
   dispatch proof this go/no-go needs; do not attempt to resolve the child's
   human gate.

5. Repeat with `input: { budget: 0 }` against a **fresh** throwaway copy of
   `rust-godfile` — expect a full auto-chain straight to a terminal `paused`
   state (no human gate involved, since `picking` routes to
   `reporting_paused` before ever reaching `refactoring`).

6. Repeat with `tests/fixtures/rust-findings` (no god-file) and
   `input: { budget: 1 }` against a **fresh** throwaway copy — expect a full
   auto-chain to terminal `done`.

## Acceptance (selection + dispatch + budget + zero-count)

```bash
# Run 1 (budget:1, rust-godfile) — read from the `praxec command` response:
[ -n "$WORST_PATH" ] && echo OK_SELECT || echo FAIL_SELECT
# Child's context.target_path (via `praxec query '{"workflowId":"<child id>"}'`)
[ "$CHILD_TARGET" = "$WORST_PATH" ] && echo OK_DISPATCH || echo FAIL_DISPATCH

# Run 2 (budget:0, fresh rust-godfile) — chain must show
# picking -> budget_spent -> reporting_paused -> paused (terminal), and:
cat "$WORK2/.praxec/reports/structural-"*.json \
  | jq -e '.tier2.residual >= 1 and .tier2.targets_attempted == 0' \
  >/dev/null && echo OK_BUDGET || echo FAIL_BUDGET

# Run 3 (budget:1, fresh rust-findings, NO god-file) — chain must show
# picking -> none -> reporting -> done (terminal), NOT stuck:
[ "$RUN3_STATE" = "done" ] && echo OK_ZEROCOUNT || echo FAIL_ZEROCOUNT
```

Expected: `OK_SELECT`, `OK_DISPATCH`, `OK_BUDGET`, `OK_ZEROCOUNT`. Ensure no
`campaign/*` branch or `.praxec/` directory is left in the pack repo itself
(all `.praxec/reports/` output lands inside each throwaway `$WORK`, never here).

## Cleanup

```bash
rm -rf "$WORK" "$WORK2" "$WORK3"   # every throwaway fixture copy
git branch --list "campaign/*"    # must be empty
ls .praxec 2>&1                   # must not exist
```

## Zero-god-file edge — real defect found + fixed

The brief anticipated one specific risk here: that `_summary.by_id.SOS001`
might be ABSENT (not `0`) when a scan finds no god file, which would leave
`picking`'s `== 0` / `> 0` guards both unmatched and the flow permanently
stuck. Driving the actual runtime surfaced that risk was real — **and also
surfaced a second, more fundamental defect that would have broken the
selection/dispatch/budget cases too, not just the zero-count edge.**

### Defect 1 (the anticipated one): `_summary.by_id.SOS001` is absent, not 0

Confirmed directly against `structureos-mcp scan_repo` on
`tests/fixtures/rust-findings` (no god file):

```json
{"_summary": {"by_id": {"SOS-RUSTC-OFF": 1}, "worst_files": []}, "focus": null}
```

No `SOS001` key at all in `by_id`, and `focus` is `null` (not an object with
`items`). A naive `.by_id.SOS001` read is therefore `null`/absent, not `0`.

### Defect 2 (discovered live, not anticipated by the brief): `kind: mcp` executor output is a text envelope, not parsed JSON

The very first live run (budget:1 against `rust-godfile`, which DOES have a
god file) reproduced the exact "stuck" symptom described for the zero-count
case — `chain.failed`, `"no viable deterministic transition in state
'picking': all 3 candidates had failing guards"` — even though a god file
was present. Reading the `workflow.transition` audit event showed why:
`scanning`'s `output:` bindings (`god_file_count:
"$.output._summary.by_id.SOS001"`, `worst_path:
"$.output.focus.items[0].node_id"`) both resolved to `null` regardless of
what the scan found.

Root cause, confirmed by reading `praxec-executors/src/mcp.rs` (lines
~428-434) and driving `structureos-mcp` directly over stdio JSON-RPC:
`structureos-mcp` declares no `outputSchema` for its one `structureos` tool
and returns a plain MCP **text** content block, never `structuredContent`.
praxec's `kind: mcp` executor's fallback for that case is
`{"content":[{"type":"text","text":"<the real JSON, as a string>"}]}` — so
`$.output._summary...` / `$.output.focus...` were reaching for fields that
simply are not there; the real data is one level deeper, JSON-encoded as a
*string*, and praxec's plain JSON-pointer path reader (`mapping.rs`) has no
mechanism to parse a string value further. This is NOT specific to the
zero-god-file case — it broke every scan, present god-file or not, and would
have failed the selection (Run 1) and budget (Run 2) assertions too, not
just the zero-count one (Run 3).

(`kind: script` already has an analogous convenience — its stdout is
auto-JSON-parsed into `$.output.json.*` — `kind: mcp` has no equivalent.)

### Fix applied (`frontrails-campaign.yaml`)

- `scanning`'s `kind: mcp` output now binds only the raw text out of the
  envelope: `scan_text: "$.output.content.0.text"` (a plain pointer path —
  numeric segments index arrays fine, no `[*]` needed), and its `scan`
  transition now targets a **new** state, `extracting`, instead of `picking`
  directly.
- **New state `extracting`** runs a **new script**,
  `run.campaign.extract-godfile-summary` (blessed `run.` root, SPEC §22.4),
  which takes `scan_text` as `argv1`, uses `jq` to pull
  `._summary.by_id.SOS001 // 0` and `._summary.worst_files[0].path // empty`
  (both defaulting safely when absent/null — this is where Defect 1 is
  actually closed), and prints `{"god_file_count": N, "worst_path": "..."}`
  to stdout — auto-parsed by the script executor into `$.output.json.*`,
  which `extracting`'s `output:` then binds to `god_file_count` / `worst_path`.
  `extracting`'s `extract` transition targets `picking` (unchanged from there
  on).

### Defect 3 (whole-branch review, source-verified): worst_path must come from `worst_files`, not `focus`

An earlier draft of `extracting` bound `worst_path` from
`.focus.items[0].node_id`. That is wrong on a real repo. `focus` is the
FIRST category in StructureOS's `PRIORITY_ORDER` with a non-zero count, and
that order is `ParseErrors(SOS400) -> GodFiles(SOS001) -> ...`
(`structureos-contract/src/action_digest.rs`). So `focus.category ==
god_files` ONLY when the repo has NO parse errors. On a repo with parse
errors alongside god-files, `god_file_count` (from `_summary.by_id.SOS001`)
is still > 0 so `picking` correctly routes to `refactor` — but
`focus.items[0].node_id` would resolve to a PARSE-ERROR node, dispatching
the god-file flow against the wrong file. The fixtures here hid this (each
has only god-files or no findings, so `focus` is always `god_files`/`null`).
**Fix:** bind `worst_path` from `._summary.worst_files[0].path` instead —
that array is category-independent and sorted by `god_file_score`
descending (`router_summary.rs` `build_worst_files`, filtered to
score > 0.5), so `worst_files[0].path` is the worst god-file whenever
SOS001 > 0, and empty when none exist. Both bindings agree when only
god-files are present (why all three E2E runs still pass), but only the
`worst_files` binding is correct once parse errors coexist.
(An extra `worst_path == ""` guard on `picking` was considered and
deliberately NOT added — it risks a stuck state, and the `worst_files`
binding makes `worst_path` reliable whenever SOS001 > 0.)

**Live proof on a real repo (`frontrails-product`, 1397 files, 29 god-files).**
A dedicated parse-error probe turned out to be infeasible: tree-sitter
*recovers* from broken syntax (it embeds ERROR nodes and returns
`Some(tree)`, so `parse_ok` stays `true` — `scan/mod.rs` DOC-002 comment +
`parse_by_language`), so `parse_ok = false` — the ONLY trigger for SOS400 —
fires only when the parser returns `None` (no parser / timeout), never on
ordinary bad Rust. Confirmed empirically twice: a throwaway god-file crate
with a deliberately-broken `.rs` file still reported
`_summary.by_id` WITHOUT `SOS400` and `focus.category == "god_files"`; and a
full scan of `frontrails-product` reported `parse_error_count: 0` /
`parse_success_ratio: 1.0` across all 1397 files. So the parse-error branch
of this defect rests on the source-verified `PRIORITY_ORDER` reasoning
above, not a live repro.

BUT the same real scan proved the fix matters even with ZERO parse errors:
with `focus.category == "god_files"`, the two bindings still resolve to
DIFFERENT files —
`_summary.worst_files[0].path == "crates/structureos-runtime/src/refactor/pipeline/tests/mod.rs"`
(god_file_score **2.76**, the true worst by score) versus
`focus.items[0].node_id == "crates/structureos-runtime/src/scan/mod.rs"`
(god_file_score **1.83**). `focus.items` is ranked/filtered by the server's
own presentation logic (it foregrounds the non-test decomposition target),
NOT strictly by `god_file_score`, so the old binding would have dispatched
the god-file flow against a lower-scored file than the plan-blessed
"worst". `_summary.worst_files[0].path` is the category-independent,
score-sorted field, so it is correct in BOTH the parse-error case (verified
by source) and the divergent-ranking case (verified live).
- A **fourth, independent** defect surfaced while proving Run 2 (budget:0):
  `initialContext` used to hardcode a literal `budget: 3`. praxec's
  input→context seeding (`runtime.rs`, `ctx.entry(k).or_insert(...)`) only
  fills a context slot from the caller's `input` when `initialContext` does
  **not** already declare that key — so the literal `budget: 3` silently
  discarded every caller-supplied `input: {budget: N}` (confirmed live: a
  `budget:1` start still showed `context.budget == 3` before the fix). Fixed
  by removing the literal from `initialContext` and moving the default onto
  the input schema instead: `inputs.budget: { type: integer, required:
  false, default: 3 }` — `apply_schema_defaults` now fills the input only
  when the caller omits it, and the (now-empty-of-`budget`) `initialContext`
  lets that value actually reach `$.context.budget`.

All three fixes are additive and scoped to `frontrails-campaign.yaml` only;
`flow.refactor.god-file` (the dispatched flow) and `structureos-mcp` itself
are untouched, per the brief's file-list constraint.

### Live verification after the fix (rust-findings, no god-file, budget:1)

```
chain:
  scanning   -> extracting ( scan )
  extracting -> picking    ( extract )
  picking    -> reporting  ( none )
  reporting  -> done       ( report )
workflow.state: done
result: {"status": "succeeded"}
context.god_file_count: 0
context.worst_path: ""
```

Reaches terminal `done` via `picking -> none -> reporting`, exactly as the
brief specifies — no stuck state.
