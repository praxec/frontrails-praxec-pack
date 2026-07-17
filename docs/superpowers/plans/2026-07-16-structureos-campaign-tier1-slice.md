# StructureOS Campaign — Tier-1 Vertical Slice (go/no-go) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove the whole praxec plumbing end-to-end with the safest tier — a deterministic, agent-free flow that removes unused-import findings (SOS027) via `cargo fix`, gated by a cargo build, against a fixture repo.

**Architecture:** An additive `frontrails-campaign.yaml` (deep-merges alongside `frontrails.yaml`, no breaking migration) declares one orchestrator `flow.findings.sweep-safe`. Tier-1 detect/fix/verify come from a per-language extension (spec §5a) — **no StructureOS in the Tier-1 path** (its `SOS027` is unreliable). For Rust that is `inspect.rust.safe-findings` + `run.rust.fix-safe` (cargo fix — rustfix applies only compiler-verified `MachineApplicable` suggestions) from `extensions/rust.yaml`, gated on the referenced `verify.cargo.cwd` script; then it emits a per-run observability report and commits a branch. A human/CI opens the PR.

**Tech Stack:** praxec (gateway `praxec`, config YAML — orchestrators/scripts/capabilities), StructureOS MCP (`structureos-mcp`), bash + `jq` + Rust `cargo`.

**Scope:** This plan is ONLY the go/no-go slice (spec §12 phases 1–2). Tier 2 (`flow.findings.structural`) and Tier 3 (`flow.findings.triage-report`) get their own plans AFTER this slice proves the plumbing — writing their tasks now would be speculative.

## Global Constraints

- Design home: `frontrails-praxec-pack`; additive include only — **never** convert to a `praxec.repo.yaml` layout in this slice (spec §2, §13).
- Tier 1 is **agent-free**: no `actor: agent` / `kind: llm` in `flow.findings.sweep-safe`. Only `deterministic`, `human`, `noop`, `script`, `mcp` executors (spec §7, §8).
- **No StructureOS in the Tier-1 path** (spec §5a) — Tier-1 detect/fix/verify come from the language-extension seam (`inspect.rust.safe-findings`/`run.rust.fix-safe`/`verify.cargo.cwd`), because `SOS027` is unreliable. (StructureOS returns as the language-neutral structural source in Tier 2/3, separate plans.)
- The build gate is the **referenced** `verify.cargo.cwd` from base `cognitive-architectures` (contract `{passed, issues, summary}`, always exit 0) — do NOT copy it into the pack (spec §2).
- CI parity: any cargo verification mirrors frontrails CI — `cargo fmt --check`, `clippy --workspace -D warnings` (NO `--all-targets`), `test` (spec §8). `verify.cargo.cwd` already encodes this; do not re-implement.
- Unknown StructureOS codes are never auto-mutated (spec §5). This slice only ever acts on `SOS027`.
- Commit messages end with: `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

---

## File Structure

- `tests/fixtures/rust-findings/` — throwaway cargo crate with a seeded unused import (Task 1).
- `frontrails-campaign.yaml` — additive config: language-neutral campaign `workflows:` + core `scripts:` (report, git.commit-branch) (Tasks 2, 4, 5).
- `extensions/rust.yaml` — the **Rust language extension**: `ext.rust.detect-safe` + `ext.rust.fix-safe` scripts (the `detect.safe`/`fix.safe` seam; `verify.build` is the referenced `verify.cargo.cwd`) (Task 3).
- `taxonomy.finding-tiers.yaml` — canonical SOS-code → tier → treatment table; source of truth (Task 2).
- `examples/campaign-check.yaml` — a minimal gateway config that includes the pack files + the rust extension, for `praxec check` (Task 2).
- `README.md` — a "Campaign wiring" section documenting the additive include + `cognitive-architectures` dependency (Task 2).
- `docs/superpowers/plans/` — this plan.

**Language-extension seam (spec §5a):** the campaign core is language-neutral. Tier-1 detect/fix/verify come from a per-language extension exposing `detect.safe`/`fix.safe`/`verify.build`, selected by the flow's `language` input. v1 ships only the Rust extension. The orchestrator resolves the seam via `initialContext` subjects (`ext_detect_safe`, `ext_fix_safe`, `ext_verify_build`) set from `language` — Task 4 verifies whether `subject:` accepts a `$.context.*` path and falls back to literal Rust subjects if not.

**Note on "taxonomy as data the flow reads":** praxec flows cannot read an arbitrary YAML file at runtime. For this slice the taxonomy file is the **canonical human-facing source of truth**, and `flow.findings.sweep-safe` encodes its one relevant code (`SOS027`) in `initialContext` consistent with it. A Phase-2 loader script can consume the file when Tier 3 routes many codes. This is an accepted realization of spec §5, not a deviation.

---

### Task 1: Fixture crate with a seeded unused-import finding

**Files:**
- Create: `tests/fixtures/rust-findings/Cargo.toml`
- Create: `tests/fixtures/rust-findings/src/lib.rs`

**Interfaces:**
- Produces: a self-contained cargo crate at `tests/fixtures/rust-findings/` whose `cargo build` emits exactly one `unused_imports` warning, and whose StructureOS scan yields an `SOS027` finding. Consumed by Tasks 3 and 6.

- [ ] **Step 1: Write the fixture Cargo.toml**

```toml
[package]
name = "rust-findings-fixture"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
path = "src/lib.rs"
```

- [ ] **Step 2: Write the fixture lib.rs with one unused import**

```rust
// Fixture for the Tier-1 sweep: `HashMap` is imported but never used, so rustc
// emits `unused_imports` (StructureOS SOS027). `add` keeps the crate non-empty
// and buildable so the post-fix build gate is meaningful.
use std::collections::HashMap;

/// Sum two numbers. (No use of HashMap — the import above is dead.)
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_works() {
        assert_eq!(add(2, 3), 5);
    }
}
```

- [ ] **Step 3: Verify the crate builds and warns exactly once**

Run: `cd tests/fixtures/rust-findings && cargo build 2>&1 | grep -c "unused import"`
Expected: `1`

- [ ] **Step 4: Verify StructureOS sees it as SOS027**

Run (from repo root, with `structureos-mcp` on PATH): pipe an `initialize` + `scan_repo` + `get_diagnostics {id:"SOS027"}` to `structureos-mcp`, or use the live `structureos` MCP:
```
scan_repo {root: "tests/fixtures/rust-findings"} ; get_diagnostics {id: "SOS027"}
```
Expected: at least one SOS027 item for `src/lib.rs`. **Record the exact JSON path to the finding count** (e.g. `_summary.by_id.SOS027` or `items` length) — Task 4 Step 3 binds it.

- [ ] **Step 5: Commit**

```bash
git add tests/fixtures/rust-findings/
git commit -m "test: fixture crate with a seeded unused-import (SOS027) finding

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 2: Additive config skeleton, taxonomy, and `praxec check` gate

**Files:**
- Create: `frontrails-campaign.yaml`
- Create: `taxonomy.finding-tiers.yaml`
- Create: `examples/campaign-check.yaml`
- Modify: `README.md` (append a "Campaign wiring" section)

**Interfaces:**
- Produces: `frontrails-campaign.yaml` with empty `workflows:`/`scripts:` maps (filled in Tasks 3–5), plus `examples/campaign-check.yaml` that includes both pack files so `praxec check` validates the merged config. Consumed by every later task's structural check.

- [ ] **Step 1: Write the taxonomy source-of-truth**

```yaml
# taxonomy.finding-tiers.yaml
# Canonical StructureOS finding-code -> tier -> treatment table (spec §5).
# Human-facing source of truth. Flows encode the codes relevant to their tier
# in `initialContext`, consistent with this table. Unknown codes default to
# tier 3 (report) — never auto-mutated.
version: "1.0.0"
default_tier: 3
tiers:
  "1":
    name: safe-auto-fix
    treatment: cargo-fix           # deterministic, compiler-verified; no LLM
    codes: [SOS027]                # unused_imports
  "2":
    name: structural-refactor
    treatment: refactor-subflow    # cognitive-max/flow.refactor.god-file (Phase 2)
    codes: [SOS001]                # god_files
  "3":
    name: advisory-report
    treatment: report              # read-only; optional human dismiss-with-rationale
    codes: [SOS025, SOS014, SOS020, SOS103, SOS010, SOS012, SOS016, SOS028, SOS037, SOS061]
```

- [ ] **Step 2: Write the additive campaign config skeleton**

```yaml
# frontrails-campaign.yaml
# Additive campaign layer — deep-merges alongside frontrails.yaml (maps merge,
# scalars: later wins). Declares the StructureOS findings-cleanup flows + their
# scripts. No praxec.repo.yaml migration (spec §2).
version: "1.0.0"

scripts: {}      # filled in Tasks 3 and 5

workflows: {}    # filled in Task 4
```

- [ ] **Step 3: Write the check config that merges both pack files**

```yaml
# examples/campaign-check.yaml
# Minimal gateway config to structurally validate the merged campaign config.
# `verify.cargo.cwd` and flow.refactor.god-file come from the cognitive
# repos — include them here so cross-references resolve at check time.
include:
  - frontrails.yaml
  - frontrails-campaign.yaml
  - extensions/rust.yaml          # the Rust language extension (Task 3)
  # Base cognitive repo supplies verify.cargo.cwd (the Rust `verify.build`),
  # and (Phase 2) cognitive-architectures-max supplies flow.refactor.god-file.
  # Adjust paths to your checkout.
  - ../cognitive-architectures/scripts-library/verify.cargo.cwd.yaml
```

- [ ] **Step 4: Run the structural check (expected: FAIL — empty workflows)**

Run: `praxec check --config examples/campaign-check.yaml`
Expected: FAIL or a warning that `workflows`/`scripts` are empty (no flow to validate yet). This confirms the include graph resolves before any flow exists.

- [ ] **Step 5: Append the README wiring section**

Append to `README.md`:
```markdown
## Campaign wiring (StructureOS findings cleanup)

The findings-cleanup campaign ships as an **additive** `frontrails-campaign.yaml`
that deep-merges alongside `frontrails.yaml`. Consumers include both, plus the
base `cognitive-architectures` repo (for `verify.cargo.cwd`) and — for Tier 2 —
`cognitive-architectures-max` (for `flow.refactor.god-file`):

```yaml
include:
  - <frontrails-praxec-pack>/frontrails.yaml
  - <frontrails-praxec-pack>/frontrails-campaign.yaml
  - <cognitive-architectures>/scripts-library/verify.cargo.cwd.yaml
```

The campaign runs against the consumer repo via `$.run.repo_root` (praxec
run-ambient). See `docs/superpowers/specs/2026-07-16-structureos-findings-campaign-design.md`.
```

- [ ] **Step 6: Commit**

```bash
git add frontrails-campaign.yaml taxonomy.finding-tiers.yaml examples/campaign-check.yaml README.md
git commit -m "feat(campaign): additive config skeleton + finding-tier taxonomy

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 3: Rust language extension (`ext.rust.detect-safe` + `ext.rust.fix-safe`)

**Files:**
- Create: `extensions/rust.yaml`

**Interfaces:**
- Produces the Rust half of the language seam (spec §5a), both scripts taking argv `$1` = working dir (defaults to `$PWD`), exit 0 always:
  - `ext.rust.detect-safe` → JSON `{count: int}` — number of `unused import` warnings (the `detect.safe` seam).
  - `ext.rust.fix-safe` → JSON `{passed: bool, fixed_count: int, before: int, after: int, summary: string}` — the `fix.safe` seam (deterministic `cargo fix`).
  - (The Rust `verify.build` seam is the *referenced* `verify.cargo.cwd`; no new script.)
- Consumed by `flow.findings.sweep-safe`'s `probing` and `fixing` states (Task 4) via the seam subjects.

- [ ] **Step 1: Write the failing test (run detect against the fixture, expect count 1)**

```bash
# detect.safe on the untouched fixture must report exactly one unused import.
# (Body = ext.rust.detect-safe from Step 2; run with arg = the fixture dir.)
git checkout tests/fixtures/rust-findings/src/lib.rs 2>/dev/null || true
bash -c "$DETECT_BODY" _ tests/fixtures/rust-findings | jq -e '.count == 1'
```
Expected before implementing: FAIL (script does not exist / no JSON).

- [ ] **Step 2: Implement the Rust extension in `extensions/rust.yaml`**

```yaml
# extensions/rust.yaml
# The Rust language extension — the detect.safe / fix.safe half of the campaign's
# language seam (spec §5a). verify.build for Rust is the referenced
# verify.cargo.cwd (base cognitive-architectures). No StructureOS here: the
# compiler is the deterministic source of truth for the safe tier (SOS027 is
# unreliable — rustc cross-check is suppressed by default).
version: "1.0.0"
scripts:
  ext.rust.detect-safe:
    verb: detect
    lifecycle: experimental
    source: frontrails-praxec-pack
    body: |
      #!/usr/bin/env bash
      # detect.safe (Rust): count unused-import warnings. Emits {count}. exit 0.
      set -uo pipefail
      cwd="${1:-$PWD}"; cd "$cwd" 2>/dev/null || true
      count=$(cargo build --message-format=short 2>&1 | grep -c "unused import" || true)
      printf '{"count": %s}\n' "$count"

  ext.rust.fix-safe:
    verb: fix
    lifecycle: experimental
    source: frontrails-praxec-pack
    body: |
      #!/usr/bin/env bash
      # fix.safe (Rust): deterministically remove unused imports with `cargo fix`.
      # rustfix applies ONLY MachineApplicable suggestions — compiler-verified,
      # behavior-preserving — so no LLM, no judgment (spec §8). Emits
      # {passed, fixed_count, before, after, summary}; exit 0 always.
      set -uo pipefail
      cwd="${1:-$PWD}"; cd "$cwd" 2>/dev/null || true
      count_unused() { cargo build --message-format=short 2>&1 | grep -c "unused import" || true; }
      before=$(count_unused)
      log=$(cargo fix --allow-dirty --allow-staged --lib --bins --tests 2>&1) && passed=true || passed=false
      after=$(count_unused)
      fixed=$(( before - after )); [ "$fixed" -lt 0 ] && fixed=0
      summary=$(printf '%s' "$log" | tail -c 1500 | jq -Rs . 2>/dev/null || printf '""')
      printf '{"passed": %s, "fixed_count": %s, "before": %s, "after": %s, "summary": %s}\n' \
        "$passed" "$fixed" "$before" "$after" "$summary"
```

- [ ] **Step 3: Verify detect.safe passes (count 1) then fix.safe removes the import**

```bash
git checkout tests/fixtures/rust-findings/src/lib.rs
# detect: expect count 1
bash -c "$DETECT_BODY" _ tests/fixtures/rust-findings | jq -e '.count == 1'   # exit 0
# fix: apply, then assert the import is gone
bash -c "$FIX_BODY" _ tests/fixtures/rust-findings >/dev/null
grep -q "use std::collections::HashMap;" tests/fixtures/rust-findings/src/lib.rs && echo FAIL || echo PASS
git checkout tests/fixtures/rust-findings/src/lib.rs   # restore
```
Expected: detect `jq -e` exit 0; then `PASS` (import removed).

- [ ] **Step 4: Verify both JSON contracts**

- `ext.rust.detect-safe` → `jq -e '.count|type=="number"'` → exit 0.
- `ext.rust.fix-safe` → `jq -e '.passed and (.fixed_count|type=="number") and (.summary|type=="string")'` → exit 0.

- [ ] **Step 5: Commit**

```bash
git checkout tests/fixtures/rust-findings/src/lib.rs
git add extensions/rust.yaml
git commit -m "feat(campaign): Rust language extension (detect.safe + fix.safe via cargo)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 4: `flow.findings.sweep-safe` orchestrator

**Files:**
- Modify: `frontrails-campaign.yaml` (fill the `workflows:` map)

**Interfaces:**
- Consumes: the language seam (`inspect.rust.safe-findings`, `run.rust.fix-safe` from Task 3; `verify.cargo.cwd` referenced), and the core scripts `run.campaign.sweep-report` + `run.campaign.commit-branch` (Task 5). **No `structureos` in the Tier-1 path** (spec §5a). All script subjects use blessed roots (SPEC §22.4).
- Produces: workflow `flow.findings.sweep-safe`, initialState `probing`, terminal `done`. State path: `probing → gate_empty → {done | fixing → building → build_gate → {reporting → committing → done | needs_human → done}}`.

- [ ] **Step 1a: Verify whether `subject:` accepts a `$.context.*` path (seam mechanism)**

Author a one-off transition with `subject: "$.context.ext_fix_safe"` and run `praxec check`. If it resolves, the orchestrator below drives the seam dynamically from `language`. If praxec requires a literal `subject:`, replace the three `"$.context.ext_*"` subjects below with their literal Rust values (`inspect.rust.safe-findings`, `run.rust.fix-safe`, `verify.cargo.cwd`) and note in the report that language selection is realized by include-swapping the extension file (still language-neutral core, static per config). Record which mechanism praxec supports. NOTE: script subjects MUST use a blessed root (SPEC §22.4: build/test/verify/run/inspect/audit/… ) or `praxec check` fails with `INVALID_SCRIPT_SUBJECT_ROOT`.

- [ ] **Step 1: Write the orchestrator (structural "test" is `praxec check` in Step 2)**

Replace `workflows: {}` with:
```yaml
workflows:
  flow.findings.sweep-safe:
    lifecycle: experimental
    description: >
      Tier 1 — deterministic, agent-free removal of unused-import findings
      (SOS027) via cargo fix, gated by a cargo build. Commits a branch; a human
      opens the PR. A red build surfaces to needs_human — a deterministic fix
      producing a red build is itself a finding.
    initialState: probing
    initialContext:
      # Language-extension seam (spec §5a): these three subjects are the
      # detect.safe / fix.safe / verify.build seam, resolved from `language`.
      # v1 wires Rust. If praxec requires literal `subject:` (Step 1a), replace
      # the "$.context.ext_*" references in the states with these values.
      language: "rust"
      # Blessed-root script subjects (SPEC §22.4). Conceptual seam names in §5a.
      ext_detect_safe: "inspect.rust.safe-findings"
      ext_fix_safe: "run.rust.fix-safe"
      ext_verify_build: "verify.cargo.cwd"
      finding_count: 0
      fixed_count: 0
      build_passed: false
      build_summary: ""
      build_issues: ""
    states:

      probing:
        goal: Detect safe auto-fixable findings via the language extension (detect.safe).
        transitions:
          probe:
            target: gate_empty
            actor: deterministic
            executor:
              kind: script
              subject: "$.context.ext_detect_safe"
              workingDirectory: "$.run.repo_root"
            output:
              finding_count: "$.output.json.count"

      gate_empty:
        transitions:
          none:
            target: done
            actor: deterministic
            guards: [ { kind: expr, expr: "$.context.finding_count == 0" } ]
          some:
            target: fixing
            actor: deterministic
            guards: [ { kind: expr, expr: "$.context.finding_count > 0" } ]

      fixing:
        goal: Apply the deterministic fix via the language extension (fix.safe; NO agent).
        transitions:
          fix:
            target: building
            actor: deterministic
            executor:
              kind: script
              subject: "$.context.ext_fix_safe"
              workingDirectory: "$.run.repo_root"
            output:
              fixed_count: "$.output.json.fixed_count"

      building:
        goal: Build/test gate via the language extension (verify.build) — the real acceptance test.
        transitions:
          build:
            target: build_gate
            actor: deterministic
            executor:
              kind: script
              subject: "$.context.ext_verify_build"
              workingDirectory: "$.run.repo_root"
            output:
              build_passed: "$.output.json.passed"
              build_summary: "$.output.json.summary"
              build_issues: "$.output.json.issues"

      build_gate:
        transitions:
          accept:
            target: reporting
            actor: deterministic
            guards: [ { kind: expr, expr: "$.context.build_passed == true" } ]
          reject:
            target: needs_human
            actor: deterministic
            guards: [ { kind: expr, expr: "$.context.build_passed == false" } ]

      reporting:
        goal: Emit the per-run observability report.
        transitions:
          report:
            target: committing
            actor: deterministic
            executor:
              kind: script
              subject: run.campaign.sweep-report
              workingDirectory: "$.run.repo_root"
            output:
              report_path: "$.output.json.report_path"

      committing:
        goal: Commit the fixes on a branch; a human/CI opens the PR.
        transitions:
          commit:
            target: done
            actor: deterministic
            executor:
              kind: script
              subject: run.campaign.commit-branch
              workingDirectory: "$.run.repo_root"
            output:
              branch: "$.output.json.branch"

      needs_human:
        goal: >
          The deterministic fix broke the build — a compiler-verified fix
          producing a red build is itself a finding. Review $.context.build_issues
          and decide.
        transitions:
          review:
            target: done
            actor: human
            inputSchema:
              type: object
              required: [decision]
              properties:
                decision: { type: string }
            executor: { kind: noop }
            output:
              decision: "$.arguments.decision"

      done:
        terminal: true
```

- [ ] **Step 2: Structural validation via `praxec check`**

Run: `praxec check --config examples/campaign-check.yaml`
Expected: exit 0 (all state targets resolve, no dangling transitions, executors reference defined scripts/connections). Note: `report.sweep-safe` and `git.commit-branch` are defined in Task 5 — run this check AFTER Task 5, or temporarily point `reporting`/`committing` at `{ kind: noop }` to check now and restore after Task 5.

- [ ] **Step 3: Runtime structural describe**

With the gateway served against the fixture (`praxec serve --config examples/campaign-check.yaml`), run via the praxec MCP:
```
praxec.query { subject: "flow.findings.sweep-safe" }   # describe
```
Expected: a state graph with `probing` as initial and `done` terminal; every transition target present; `structural_fingerprint` returned.

- [ ] **Step 4: Commit**

```bash
git add frontrails-campaign.yaml
git commit -m "feat(campaign): flow.findings.sweep-safe (Tier 1 orchestrator)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 5: Report + branch scripts (observability + PR handoff)

**Files:**
- Modify: `frontrails-campaign.yaml` (add two scripts to `scripts:`)

**Interfaces:**
- Consumes: `$.context` fields set by `flow.findings.sweep-safe` (passed as argv/env by the script executor).
- Produces: `run.campaign.sweep-report` — writes `.praxec/reports/sweep-safe-<runid>.json`+`.md`, emits `{report_path}`. `run.campaign.commit-branch` — creates branch `campaign/sweep-safe`, commits the working tree, emits `{branch}`. Both exit 0. (Blessed root `run`, SPEC §22.4.)

- [ ] **Step 1: Add the report script**

Add to `scripts:` in `frontrails-campaign.yaml`:
```yaml
  run.campaign.sweep-report:
    verb: run
    lifecycle: experimental
    source: frontrails-praxec-pack
    body: |
      #!/usr/bin/env bash
      # Per-run observability report (spec §9). Reads the campaign fields from
      # the environment the script executor injects (PRAXEC_CTX_* mirror
      # $.context), writes JSON + markdown under .praxec/reports/, emits the path.
      set -uo pipefail
      cwd="${1:-$PWD}"; cd "$cwd" 2>/dev/null || true
      dir=".praxec/reports"; mkdir -p "$dir"
      # Fields arrive as env vars mirroring $.context (verify names against the
      # gateway's script-env contract; fall back to 0/"" when unset).
      finding_count="${PRAXEC_CTX_finding_count:-0}"
      fixed_count="${PRAXEC_CTX_fixed_count:-0}"
      build_passed="${PRAXEC_CTX_build_passed:-false}"
      stamp="${PRAXEC_RUN_ID:-run}"
      json="$dir/sweep-safe-$stamp.json"
      printf '{"tier":1,"findings_in":{"SOS027":%s},"tier1":{"fixed":%s,"gate_result":"%s"},"unknown_codes":[]}\n' \
        "$finding_count" "$fixed_count" "$build_passed" > "$json"
      {
        echo "# Sweep-safe run $stamp"
        echo "- SOS027 findings in: $finding_count"
        echo "- fixed: $fixed_count"
        echo "- build gate: $build_passed"
      } > "$dir/sweep-safe-$stamp.md"
      printf '{"report_path": %s}\n' "$(printf '%s' "$json" | jq -Rs .)"
```

- [ ] **Step 2: Add the branch-commit script**

Add to `scripts:`:
```yaml
  run.campaign.commit-branch:
    verb: run
    lifecycle: experimental
    source: frontrails-praxec-pack
    body: |
      #!/usr/bin/env bash
      # Commit the working-tree fixes on a campaign branch; a human/CI opens the
      # PR (spec §10 — no cap.coordinate.pr-open in v1). Exit 0 always.
      set -uo pipefail
      cwd="${1:-$PWD}"; cd "$cwd" 2>/dev/null || true
      branch="campaign/sweep-safe"
      git checkout -b "$branch" 2>/dev/null || git checkout "$branch"
      git add -A
      if git diff --cached --quiet; then
        printf '{"branch": %s, "committed": false}\n' "$(printf '%s' "$branch" | jq -Rs .)"
      else
        git commit -m "chore(campaign): sweep unused imports (SOS027) via cargo fix" >/dev/null
        printf '{"branch": %s, "committed": true}\n' "$(printf '%s' "$branch" | jq -Rs .)"
      fi
```

- [ ] **Step 3: Verify both scripts' JSON contracts**

Run each body manually in a scratch dir; pipe stdout to `jq -e`:
- `run.campaign.sweep-report` → `jq -e '.report_path|type=="string"'` → exit 0.
- `run.campaign.commit-branch` (in a scratch git repo) → `jq -e '.branch|type=="string"'` → exit 0.

- [ ] **Step 4: Re-run the full structural check**

Run: `praxec check --config examples/campaign-check.yaml`
Expected: exit 0 (now `reporting`/`committing` reference defined scripts).

- [ ] **Step 5: Commit**

```bash
git add frontrails-campaign.yaml
git commit -m "feat(campaign): per-run report + branch-commit scripts (observability + PR handoff)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 6: End-to-end go/no-go run against the fixture

**Files:**
- Create: `tests/campaign-tier1-e2e.md` (the manual/CI-gated E2E procedure, matching `tests/README.md` style)

**Interfaces:**
- Consumes: everything above.
- Produces: a documented, reproducible end-to-end run proving fix → green build → report → branch. This is the **go/no-go gate** for the whole campaign.

- [ ] **Step 1: Write the E2E procedure**

Create `tests/campaign-tier1-e2e.md`:
```markdown
# Tier-1 sweep-safe E2E (go/no-go)

Requires `praxec` + `structureos-mcp` on PATH and a checkout of
`cognitive-architectures` beside this repo. The mutating run happens in an
ISOLATED throwaway copy so it never touches the pack repo.

1. Copy the fixture to a throwaway git repo:
   ```bash
   WORK=$(mktemp -d)
   cp -r tests/fixtures/rust-findings/. "$WORK/"
   git -C "$WORK" init -q && git -C "$WORK" add -A \
     && git -C "$WORK" -c user.email=e2e@local -c user.name=e2e commit -qm init
   CFG="$PWD/examples/campaign-check.yaml"   # absolute — includes resolve relative to the config file
   ```
2. Structure check: `praxec check --config "$CFG"`  → exit 0
3. Serve with the throwaway as repo root: `cd "$WORK" && praxec serve --config "$CFG"`
4. Start the flow: `praxec.command { definitionId: "flow.findings.sweep-safe", input: {} }`
5. Drive deterministic transitions to `done` (follow the returned `links`).

## Acceptance (all assertions against $WORK)
- `$WORK/src/lib.rs` no longer contains `use std::collections::HashMap;`
- final state is `done` (not `needs_human`)
- `$WORK/.praxec/reports/sweep-safe-*.json` exists with `tier1.fixed >= 1`
- branch `campaign/sweep-safe` exists in `$WORK` with one commit
```

- [ ] **Step 2: Execute the E2E and capture the run**

Follow the procedure. Record the final workflow status and the report JSON.
Expected: final `status: succeeded`, terminal state `done`, `tier1.fixed >= 1`.

- [ ] **Step 3: Assert acceptance criteria**

Run:
```bash
grep -q "use std::collections::HashMap;" "$WORK/src/lib.rs" && echo FAIL_IMPORT || echo OK_IMPORT
ls "$WORK"/.praxec/reports/sweep-safe-*.json >/dev/null 2>&1 && echo OK_REPORT || echo FAIL_REPORT
git -C "$WORK" rev-parse --verify campaign/sweep-safe >/dev/null 2>&1 && echo OK_BRANCH || echo FAIL_BRANCH
```
Expected: `OK_IMPORT`, `OK_REPORT`, `OK_BRANCH`.

- [ ] **Step 4: Discard the throwaway repo (pack fixture was never mutated)**

```bash
rm -rf "$WORK"
```

- [ ] **Step 5: Commit the E2E doc**

```bash
git add tests/campaign-tier1-e2e.md
git commit -m "test(campaign): Tier-1 sweep-safe end-to-end go/no-go procedure

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Self-Review

**Spec coverage (§ by §):**
- §2 additive include, agent-free Tier 1, reference verify.cargo, no migration → Tasks 2, 3, 4 (constraints enforced). ✓
- §4 Tier-1 DoD (fixed-and-build-verified or clean no-op) → Task 4 `gate_empty`/`build_gate`. ✓
- §5 taxonomy + unknown→Tier3 → Task 2 taxonomy (canonical doc). §5a language seam → Task 3 Rust extension (`detect.safe`/`fix.safe`) + Task 4 orchestrator dispatches via the seam; StructureOS is NOT in the Tier-1 path. ✓
- §7 Tier-1 state machine (incl. red→needs_human) → Task 4. ✓
- §8 safety (no LLM, verify.cargo mirrors CI) → Task 3 (cargo fix only), referenced verify.cargo.cwd. ✓
- §9 observability (per-run report) → Task 5 `report.sweep-safe`. ✓
- §10 PR cadence (branch + human opens PR) → Task 5 `git.commit-branch`. ✓
- §11 testing (praxec check + fixture dry-run) → Tasks 2/4 checks + Task 1/6 fixture E2E. ✓
- §12 rollout slice = phases 1–2 → this whole plan; Tier 2/3 explicitly deferred. ✓

**Placeholder scan:** No "TBD/TODO" in deliverables. Two explicit **live-verification** steps (not placeholders): Task 1 Step 4 records the `get_diagnostics` count JSON path; Task 4 Step 1 binds `finding_count` to it. These are mandatory because praxec configs must be validated against the live gateway (pack README convention) — guessing the path would be the error.

**Type/name consistency:** `finding_count`, `fixed_count`, `build_passed`, `build_summary`, `build_issues` are defined in `initialContext` (Task 4) and consumed by the same names in `report.sweep-safe` (Task 5). Script subjects `fix.unused-imports` (Task 3), `verify.cargo.cwd` (referenced), `report.sweep-safe` + `git.commit-branch` (Task 5) match their `subject:` references in the orchestrator (Task 4). `{passed, issues, summary}` matches `verify.cargo.cwd`'s real contract; `{passed, fixed_count, before, after, summary}` matches `fix.unused-imports`.

**Known live-verification points to confirm during execution (poka-yoke, not guesses):**
1. Whether `subject:` accepts a `$.context.*` path (Task 4 Step 1a) — dynamic language seam vs. literal-subject fallback (v1 still ships only Rust either way).
2. The script-executor env/argv contract for passing `$.context.*` into `report.sweep-safe` (Task 5 Step 1 uses `PRAXEC_CTX_*` with `:-` fallbacks; confirm against the gateway's script-env docs and adjust).
3. `praxec` subcommands confirmed available on the installed gateway: `check`, `serve`, `command`, `observe`.
4. (Resolved by dogfooding) StructureOS `SOS027` is NOT a reliable Tier-1 signal — rustc cross-check suppressed by default; Tier-1 uses the compiler via the Rust extension instead.
