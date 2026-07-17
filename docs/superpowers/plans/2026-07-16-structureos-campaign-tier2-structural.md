# StructureOS Campaign — Tier-2 Structural Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `flow.findings.structural` — a budgeted, stateless, per-target orchestrator that picks the worst god-file (StructureOS `SOS001`) and dispatches it to the proven `cognitive-max/flow.refactor.god-file`, one target per run against a `budget`.

**Architecture:** The additive `frontrails-campaign.yaml` gains one orchestrator that scans via StructureOS (god-files are *structural*, reliably surfaced — unlike Tier-1's rustc-lint `SOS027`, this tier legitimately uses StructureOS), reads the worst god-file path StructureOS hands back, and invokes the god-file decomposition flow cross-namespace via `kind: workflow` + `use:`. The decomposition itself (propose → human gate → move → verify → cargo build → agent fixup) is the *referenced* flow's job — already dogfooded. This slice proves target-selection + dispatch + budget, not the decomposition (that's proven separately).

**Tech Stack:** praxec (`praxec check`/`serve`/`command`/`fuzz`), StructureOS MCP, the referenced `cognitive-max/flow.refactor.god-file` (+ base `cognitive-architectures` for `verify.cargo.cwd`), and — because the god-file flow's `fixing` state is `actor: agent` — a coding agent must be wired in the consumer gateway.

**Scope:** ONLY the structural picker/dispatcher for **god-files**. `flow.refactor.method` / `flow.refactor.cycle` (god-methods, import-cycles) remain deferred to Phase 2 per spec §5. This slice does NOT re-prove the god-file decomposition end-to-end (the referenced flow owns that); it proves `flow.findings.structural` selects the right target, dispatches correctly, and honors `budget`.

## Global Constraints

- Home: `frontrails-praxec-pack`; additive `frontrails-campaign.yaml` only (spec §2). No `praxec.repo.yaml` migration.
- **Tier 2 is NOT agent-free** (unlike Tier 1) — the referenced god-file flow legitimately uses a human gate + agent fixup. `flow.findings.structural` itself is deterministic (scan/pick/gate/dispatch); it must not add its own `actor: agent`.
- StructureOS calls use `kind: mcp, connection: structureos, tool: structureos, map: {action, params}` (both resolve, fail-fast) — the god-file-proven pattern. God-files (`SOS001`) ARE reliably surfaced (structural analysis, not rustc cross-check).
- The god-file flow is invoked cross-namespace: `kind: workflow, definitionId: cognitive-max/flow.refactor.god-file, use: { inputs: { target_path: … } }`.
- All script subjects use blessed roots (SPEC §22.4). `subject:` must be LITERAL (a `$.context.*` subject passes `praxec check` but fails at runtime — see the Tier-1 findings).
- **Stateless + budgeted:** scan fresh each run; process at most `budget` targets (default 3); on budget-spent-with-residual land in `paused` (not `done`); never carry a target list across runs.
- Templating into scripts uses jsonpath `$.context.x` (NOT `{{ }}`) via `args:`; `workingDirectory:` sets cwd. Commit messages end with `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

---

## File Structure

- `tests/fixtures/rust-godfile/` — a cargo crate with one decomposable god-file (Task 1).
- `frontrails-campaign.yaml` — gains `flow.findings.structural` (Task 4) + `run.campaign.structural-report` script (Task 3).
- `examples/campaign-check.yaml` — add the `cognitive-architectures-max` include so `cognitive-max/flow.refactor.god-file` resolves at check time (Task 2).
- `README.md` — extend "Campaign wiring" with the Tier-2 deps (cognitive-max flow + a coding agent) (Task 2).
- `tests/campaign-tier2-e2e.md` — the manual/CI-gated E2E procedure (Task 5).

---

### Task 1: God-file fixture crate + record the SOS001 worst-path

**Files:**
- Create: `tests/fixtures/rust-godfile/Cargo.toml`
- Create: `tests/fixtures/rust-godfile/src/lib.rs`

**Interfaces:**
- Produces a cargo crate whose `src/lib.rs` trips StructureOS `SOS001` (god-file), so a scan's `focus.category == "god_files"` and `by_id.SOS001 >= 1`. Consumed by Tasks 4 (path binding) and 5 (E2E).

- [ ] **Step 1: Write the fixture Cargo.toml**

```toml
[package]
name = "rust-godfile-fixture"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
path = "src/lib.rs"
```

- [ ] **Step 2: Write a god-file `src/lib.rs`**

A god-file is heuristically loc + function-count driven (StructureOS `god_file_score`). Generate a file with many small independent functions so it is decomposable and clearly trips the heuristic. Minimum ~40 functions / ~250+ LOC. Example shape (repeat the pattern to reach the threshold):

```rust
//! Fixture god-file: many small independent functions so StructureOS SOS001 fires
//! and propose_decomposition finds clean groups. Intentionally oversized.

pub fn add(a: i64, b: i64) -> i64 { a + b }
pub fn sub(a: i64, b: i64) -> i64 { a - b }
pub fn mul(a: i64, b: i64) -> i64 { a * b }
// … continue with div, rem, min, max, clamp, gcd, lcm, abs, signum, pow,
// is_even, is_odd, factorial, fib, and enough sibling groups (string utils,
// vec utils, option utils) that the file exceeds the god-file heuristic.
```

Author enough real functions (no dead code — they can be trivially correct) to cross the heuristic. Verify in Step 4.

- [ ] **Step 3: Verify the crate builds clean**

Run: `cd tests/fixtures/rust-godfile && cargo build 2>&1 | tail -3`
Expected: builds with no errors (warnings acceptable but prefer none).

- [ ] **Step 4: Verify StructureOS flags it SOS001 and record the worst-path JSON location**

Drive `structureos-mcp` (STRUCTUREOS_WORKSPACE_ROOT=the fixture) with `scan_repo {root:"."}`, then inspect the response. Confirm `focus.category == "god_files"` and `_summary.by_id.SOS001 >= 1`. **Record the JSON path to the worst god-file** — from the live scan of a real repo it is `_action.call.params.path` (StructureOS hands back "Worst of N god files") and equivalently `focus.items[0].node_id`. Also run `get_diagnostics {id:"SOS001", limit:50}` and record whether its items carry `node_id` sorted worst-first (Task 4 chooses the most robust of these). If the fixture is too small to trip the heuristic, enlarge `src/lib.rs` until `by_id.SOS001 >= 1`.

- [ ] **Step 5: Commit**

```bash
git add tests/fixtures/rust-godfile/
git commit -m "test: fixture crate with a decomposable god-file (SOS001)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 2: Wire the cognitive-max include + README

**Files:**
- Modify: `examples/campaign-check.yaml`
- Modify: `README.md`

**Interfaces:**
- Produces a check config that additionally resolves `cognitive-max/flow.refactor.god-file`, so `praxec check` can validate the Tier-2 flow's cross-namespace reference. Consumed by Tasks 4 and 5.

- [ ] **Step 1: Add the cognitive-max include**

Append to `examples/campaign-check.yaml`'s `include:` list:
```yaml
  # Tier-2 dependency: the god-file decomposition flow (referenced, not copied)
  # and the scripts it uses live in the cognitive-architectures repos.
  - ../cognitive-architectures-max/orchestrators/flow.refactor.god-file.yaml
```
(The base `../cognitive-architectures/scripts-library/verify.cargo.cwd.yaml` is already included from the Tier-1 slice.)

- [ ] **Step 2: Confirm the god-file flow resolves**

Run: `praxec check --config examples/campaign-check.yaml`
Expected: exit 0, `validation: ok`, and `cognitive-max/flow.refactor.god-file` (or its resolved id) appears in the workflow list. If the god-file flow pulls in additional unresolved script/connection deps, add their include paths until check is clean, and record which were needed in the commit body.

- [ ] **Step 3: Extend the README wiring section**

Append to the "Campaign wiring" section in `README.md`:
```markdown
### Tier 2 (structural) additional wiring

`flow.findings.structural` references `cognitive-max/flow.refactor.god-file`, so the
consumer gateway must also load `cognitive-architectures-max` (the flow) and have a
**coding agent** wired — the god-file flow's `fixing` state is `actor: agent`.
Tier 2 is therefore not agent-free (Tier 1 is).
```

- [ ] **Step 4: Commit**

```bash
git add examples/campaign-check.yaml README.md
git commit -m "feat(campaign): wire cognitive-max god-file flow for Tier 2

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 3: `run.campaign.structural-report` script

**Files:**
- Modify: `frontrails-campaign.yaml` (add one script to `scripts:`)

**Interfaces:**
- Produces `run.campaign.structural-report` — receives run context via templated `args:` (positional: `$1`=god_file_count, `$2`=targets_done, `$3`=budget, `$4`=residual), writes `.praxec/reports/structural-<stamp>.json`+`.md`, emits `{report_path}`. exit 0. Consumed by `flow.findings.structural`'s `reporting` state (Task 4).

- [ ] **Step 1: Add the script**

Add to `scripts:` in `frontrails-campaign.yaml` (mirrors the proven Tier-1 `run.campaign.sweep-report` arg contract — context arrives as positional args, cwd is the workingDirectory):
```yaml
  run.campaign.structural-report:
    verb: run
    lifecycle: experimental
    source: frontrails-praxec-pack
    body: |
      #!/usr/bin/env bash
      # Per-run structural report (spec §9). Context arrives as templated `args:`
      # positionally ($1..$4). cwd = the executor's workingDirectory. exit 0.
      set -uo pipefail
      god_file_count="${1:-0}"; targets_done="${2:-0}"; budget="${3:-0}"; residual="${4:-0}"
      dir=".praxec/reports"; mkdir -p "$dir"
      stamp="$(date -u +%Y%m%dT%H%M%SZ 2>/dev/null || echo run)"
      json="$dir/structural-$stamp.json"
      printf '{"tier":2,"findings_in":{"SOS001":%s},"tier2":{"targets_attempted":%s,"budget":%s,"residual":%s},"unknown_codes":[]}\n' \
        "$god_file_count" "$targets_done" "$budget" "$residual" > "$json"
      {
        echo "# Structural run $stamp"
        echo "- god-files (SOS001) in: $god_file_count"
        echo "- targets attempted this run: $targets_done (budget $budget)"
        echo "- residual god-files: $residual"
      } > "$dir/structural-$stamp.md"
      printf '{"report_path": %s}\n' "$(printf '%s' "$json" | jq -Rs .)"
```

- [ ] **Step 2: Verify the JSON contract**

Run the body with args `5 1 3 4` in a scratch dir; pipe stdout to `jq -e '.report_path|type=="string"'`.
Expected: exit 0; and the written JSON has `tier2.targets_attempted == 1`, `tier2.residual == 4`.

- [ ] **Step 3: `praxec check`**

Run: `praxec check --config examples/campaign-check.yaml` → exit 0 (no `INVALID_SCRIPT_SUBJECT_ROOT`; `run` is blessed).

- [ ] **Step 4: Commit**

```bash
git add frontrails-campaign.yaml
git commit -m "feat(campaign): run.campaign.structural-report script (Tier-2 observability)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 4: `flow.findings.structural` orchestrator

**Files:**
- Modify: `frontrails-campaign.yaml` (add to `workflows:`)

**Interfaces:**
- Consumes: StructureOS (`scan_repo`, `get_diagnostics`), `cognitive-max/flow.refactor.god-file` (referenced), `run.campaign.structural-report` (Task 3).
- Produces: workflow `flow.findings.structural`, input `budget` (default 3), initial `scanning`, terminals `done` and `paused`. State path: `scanning → picking → gate → { done | paused | refactoring → (child god-file flow) → counting → scanning }`.

- [ ] **Step 1: Confirm the worst-path binding (record from Task 1 Step 4)**

Bind `worst_path` from the scan output at the JSON path Task 1 recorded — default `"$.output._action.call.params.path"` (StructureOS's "worst god file" handback), with `god_file_count` from `"$.output._summary.by_id.SOS001"`. If Task 1 found `get_diagnostics {id:SOS001}` more robust for worst-first ordering, use a `collecting` state calling that instead and bind `worst_path` to its first item's `node_id`. Record the choice.

- [ ] **Step 2: Write the orchestrator**

Add to `workflows:` in `frontrails-campaign.yaml`:
```yaml
  flow.findings.structural:
    lifecycle: experimental
    description: >
      Tier 2 — budgeted, stateless, per-target god-file decomposition. Scans via
      StructureOS, picks the worst god-file (SOS001), and dispatches it to the
      proven cognitive-max/flow.refactor.god-file (human-gated + agent fixup).
      Processes at most `budget` targets per run; residual god-files land in
      `paused` (re-run to continue). NOT agent-free — the referenced flow uses a
      coding agent.
    inputs:
      budget: { type: integer, required: false }
    initialState: scanning
    initialContext:
      act_scan: "scan_repo"
      empty_params: {}
      budget: 3
      targets_done: 0
      god_file_count: 0
      worst_path: ""
    states:

      scanning:
        goal: Refresh the structural model (StructureOS) so god-file selection is current.
        transitions:
          scan:
            target: picking
            actor: deterministic
            executor:
              kind: mcp
              connection: structureos
              tool: structureos
              map:
                action: "$.context.act_scan"
                params: "$.context.empty_params"
            output:
              # SOS001 count + the worst god-file StructureOS hands back.
              god_file_count: "$.output._summary.by_id.SOS001"
              worst_path: "$.output._action.call.params.path"

      picking:
        goal: Decide whether to refactor another god-file this run.
        transitions:
          none:
            target: reporting
            actor: deterministic
            guards: [ { kind: expr, expr: "$.context.god_file_count == 0" } ]
          budget_spent:
            target: reporting_paused
            actor: deterministic
            guards:
              - { kind: expr, expr: "$.context.god_file_count > 0" }
              - { kind: expr, expr: "$.context.targets_done >= $.context.budget" }
          refactor:
            target: refactoring
            actor: deterministic
            guards:
              - { kind: expr, expr: "$.context.god_file_count > 0" }
              - { kind: expr, expr: "$.context.targets_done < $.context.budget" }

      refactoring:
        goal: Dispatch the worst god-file to the god-file decomposition flow.
        transitions:
          decompose:
            target: counting
            actor: deterministic
            executor:
              kind: workflow
              definitionId: cognitive-max/flow.refactor.god-file
              use:
                inputs:
                  target_path: "$.context.worst_path"
            output:
              last_target: "$.context.worst_path"

      counting:
        goal: Count the processed target and re-scan for the next.
        transitions:
          next:
            target: scanning
            actor: deterministic
            executor: { kind: noop }
            output:
              targets_done: { add: ["$.context.targets_done", 1] }

      reporting:
        goal: Emit the per-run structural report (all god-files cleared or none present).
        transitions:
          report:
            target: done
            actor: deterministic
            executor:
              kind: script
              subject: run.campaign.structural-report
              workingDirectory: "$.run.repo_root"
              args:
                - "$.context.god_file_count"
                - "$.context.targets_done"
                - "$.context.budget"
                - "$.context.god_file_count"
            output:
              report_path: "$.output.json.report_path"

      reporting_paused:
        goal: Emit the report noting residual god-files (budget spent).
        transitions:
          report:
            target: paused
            actor: deterministic
            executor:
              kind: script
              subject: run.campaign.structural-report
              workingDirectory: "$.run.repo_root"
              args:
                - "$.context.god_file_count"
                - "$.context.targets_done"
                - "$.context.budget"
                - "$.context.god_file_count"
            output:
              report_path: "$.output.json.report_path"

      paused:
        terminal: true
      done:
        terminal: true
```

- [ ] **Step 3: Structural validation + coverage**

Run: `praxec check --config examples/campaign-check.yaml` → exit 0, `flow.findings.structural` resolved.
Then (if available) `praxec fuzz` over it → all edges covered, no orphans, both terminals (`done`, `paused`) reachable. Also `praxec query { subject: "flow.findings.structural" }` (describe) → confirm initial `scanning`, terminals `done`+`paused`, no dangling targets. If a guard/expr syntax error appears (e.g. comparing two `$.context.*` in one `expr`), record the exact praxec error and adjust to the supported guard form.

- [ ] **Step 4: Commit**

```bash
git add frontrails-campaign.yaml
git commit -m "feat(campaign): flow.findings.structural (Tier-2 budgeted god-file dispatcher)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 5: E2E go/no-go — pick + dispatch + budget

**Files:**
- Create: `tests/campaign-tier2-e2e.md`

**Interfaces:**
- Consumes everything above. Produces a documented run proving `flow.findings.structural` selects the correct worst god-file, dispatches it to the god-file flow with the right `target_path`, and honors `budget`. The full decomposition is the referenced flow's job — NOT re-proven here.

- [ ] **Step 1: Write the E2E procedure**

Create `tests/campaign-tier2-e2e.md`:
```markdown
# Tier-2 structural E2E (go/no-go)

Requires `praxec` + `structureos-mcp` on PATH, sibling `cognitive-architectures`
+ `cognitive-architectures-max` checkouts, and (to complete a real decomposition)
a coding agent wired — NOT needed for this go/no-go, which proves selection +
dispatch + budget only.

1. Copy the god-file fixture to a throwaway git repo:
   ```bash
   WORK=$(mktemp -d)
   cp -r tests/fixtures/rust-godfile/. "$WORK/"
   git -C "$WORK" init -q && git -C "$WORK" add -A \
     && git -C "$WORK" -c user.email=e2e@local -c user.name=e2e commit -qm init
   CFG="$PWD/examples/campaign-check.yaml"
   ```
2. Structure check: `praxec check --config "$CFG"` → exit 0.
3. Serve/drive with the throwaway as repo root: `cd "$WORK"` then start
   `flow.findings.structural` with `input: { budget: 1 }` via `praxec command`.
4. Observe the transition trace: `scanning` populates `worst_path` = the fixture's
   god-file (`src/lib.rs`) and `god_file_count >= 1`; `picking` takes the
   `refactor` edge (budget 1 > 0 targets); `refactoring` dispatches
   `cognitive-max/flow.refactor.god-file` with `target_path` = that path (the
   child flow starts and reaches its first HUMAN gate — this is the dispatch proof).

## Acceptance (selection + dispatch + budget)
- `scanning` bound `worst_path` to the fixture's god-file path (not empty).
- `picking` chose `refactor` (not `none`/`budget_spent`) on the first pass.
- the child `cognitive-max/flow.refactor.god-file` was started with `target_path`
  equal to `worst_path` (verify via the child workflow's input echo / audit event).
- With `budget: 0`, a fresh run instead takes `budget_spent` → `reporting_paused`
  → `paused`, and the report shows `tier2.residual >= 1` — proving the budget gate.
```

- [ ] **Step 2: Execute the `budget: 1` run**

Drive it; capture the transition trace and the child dispatch. Record the `worst_path` value and the child's received `target_path`.
Expected: `worst_path` non-empty (the fixture god-file), child started with matching `target_path`.

- [ ] **Step 3: Execute the `budget: 0` run (budget-gate proof)**

Fresh throwaway; start with `input: { budget: 0 }`.
Expected: `picking` → `budget_spent` → `reporting_paused` → terminal `paused`; the report JSON shows `tier2.residual >= 1` and `targets_attempted == 0`.

- [ ] **Step 4: Assert + clean up**

```bash
# from the budget:1 run trace
[ -n "$WORST_PATH" ] && echo OK_SELECT || echo FAIL_SELECT
[ "$CHILD_TARGET" = "$WORST_PATH" ] && echo OK_DISPATCH || echo FAIL_DISPATCH
# from the budget:0 run
echo "$PAUSED_REPORT" | jq -e '.tier2.residual >= 1 and .tier2.targets_attempted == 0' >/dev/null && echo OK_BUDGET || echo FAIL_BUDGET
rm -rf "$WORK"   # and any budget:0 throwaway
```
Expected: `OK_SELECT`, `OK_DISPATCH`, `OK_BUDGET`. Ensure no `campaign/*` branch or `.praxec/` dir left in the pack repo.

- [ ] **Step 5: Commit**

```bash
git add tests/campaign-tier2-e2e.md
git commit -m "test(campaign): Tier-2 structural selection/dispatch/budget E2E

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Self-Review

**Spec coverage:**
- §5 Tier-2 = structural refactor, god-files, per-target human gate via the referenced flow → Tasks 2, 4. ✓
- §5a StructureOS is the language-neutral structural source for Tier 2 (contrast Tier-1's seam) → Task 4 uses `kind: mcp` StructureOS. ✓
- §6/§7 `flow.findings.structural` budgeted/stateless/per-target, `paused` on budget-spent → Task 4 state machine. ✓
- §9 observability → Task 3 report. ✓
- §12 method/cycle deferred → out of scope, stated. ✓

**Placeholder scan:** No TBD/TODO in deliverables. Three explicit live-verification points (poka-yoke, not guesses): Task 1 Step 4 records the SOS001 worst-path JSON location; Task 4 Step 1 binds `worst_path` to it; Task 4 Step 3 confirms the two-`$.context` guard `expr` form (`targets_done >= budget`) is accepted, adjusting if praxec needs a different guard shape.

**Type/name consistency:** `god_file_count`, `targets_done`, `budget`, `worst_path` are defined in `initialContext` (Task 4) and passed positionally to `run.campaign.structural-report` (Task 3) as `$1..$4`. Script subject `run.campaign.structural-report` (Task 3) matches its `subject:` in Task 4. The referenced `cognitive-max/flow.refactor.god-file` input `target_path` matches its declared input (`{target_path, required}`).

**Known live-verification points:**
1. The scan JSON path for the worst god-file (`_action.call.params.path` vs `get_diagnostics {id:SOS001}` items) — Task 1 records, Task 4 binds.
2. Guard `expr` comparing two `$.context.*` values (`targets_done >= budget`) — confirm praxec supports it (Task 4 Step 3); if not, seed a `remaining` slot or split the guard.
3. `use: { inputs: { target_path } }` binding to a **flow** (not a cap) target — confirm the child receives it (Task 5 dispatch assertion). If praxec requires the child be a `cap.*` for `use:`, wrap the god-file flow in a thin `cap.coordinate.god-file` passthrough and reference that instead (record the finding).
