# Design — StructureOS Findings-Cleanup Campaign (praxec)

- **Date:** 2026-07-16
- **Status:** Approved design → ready for implementation plan
- **Repo (home):** `frontrails-praxec-pack`
- **Program:** Operationalize FrontRails *os capabilities as praxec workflows. This spec is **Sub-project A** of three (A = structureos findings cleanup; B = Preveti→IntentOS strategic/compass population; C = uxos+intentos UX vet-and-fix). B and C are separate specs.

---

## 1. Context & goal

The `frontrails-praxec-pack` already exposes every *os action as a governed praxec proxy capability (`intentos.*`, `structureos.*`, `uxos.*`) via `map:` binding, keeping the *os MCP servers pure. `cognitive-architectures-max` already proves the orchestration patterns (`cap.coordinate.structureos`, and a working `flow.refactor.god-file`: scan → propose → human gate → move → structural-verify → cargo build gate → bounded agent-fixup).

**Goal:** a praxec campaign that **safely drives all StructureOS findings to a resolved state** — every finding is either *fixed* or *explicitly dismissed with rationale* — running against a consumer repo (frontrails-product) with human governance proportional to each finding class's risk.

**Non-goal for A:** anything touching IntentOS/UXOS specs (no async-HITL; those are Sub-projects B/C).

## 2. Where it lives

All new definitions go in `frontrails-praxec-pack`, which adopts the `praxec.repo.yaml` repo-manifest layout (mirroring `cognitive-max`) so units live in their own files and carry the `frontrails/` namespace:

```
frontrails-praxec-pack/
  praxec.repo.yaml            # (new) repo manifest — makes this a namespaced repo
  frontrails.yaml             # (existing) *os connections + proxy exposure
  capabilities/               # (new) cap.*
  orchestrators/              # (new) flow.*
  scripts-library/            # (new) verify.cargo.cwd, etc.
  taxonomy.finding-tiers.yaml # (new) data: SOS code → tier → treatment
  tests/                      # (existing) harness + fixtures
  docs/superpowers/specs/     # (this file)
```

**Include-migration note (Phase-1 task):** the pack is currently consumed as a single-file include (`frontrails.yaml`). Adopting the repo-manifest layout shifts consumers to including the repo/manifest. Document the new include form in the README and keep `frontrails.yaml` as the exposure fragment so existing `connections`/`capabilities` keep deep-merging as before.

## 3. Definition of done (computed, not assumed)

The campaign is a **class-routed burn-down**. One run is a single linear pass — `scan_repo` → `get_diagnostics` (build the tiered worklist) → run each tier-flow once (each tier-flow internally iterates its own classes/targets) → `reconciling` — not an unbounded outer loop. `reconciling` reaches `done` only when a re-scan shows every finding is **gone (fixed)** or present **in `list_dismissals` (dismissed-with-rationale)**; otherwise it lands in `paused`. Nothing is silently skipped — anything unrouted falls to Tier 3. Convergence to zero residual happens **across runs** (resume the paused workflow, or start a fresh run), which is what makes "all findings" tractable given the volume.

Because the volume is large and irreducibly human-gated (e.g. 287 god-methods), the campaign is **budgeted and resumable**: it takes a `budget` (max structural targets per run), and praxec `kind: workflow` sub-flows durably suspend/resume, so it **pauses** when the budget is spent with residual findings and is resumed later. "All findings" is thus an invariant the flow drives toward across runs, not one heroic execution.

## 4. Routing table — data, not control flow

Shipped as `taxonomy.finding-tiers.yaml` and read by the orchestrator. Adding/re-tiering a class is a table edit, never a state-machine change. **Invariant: an unknown SOS code defaults to Tier 3 (advisory triage) — the flow never auto-mutates a finding it does not recognize.**

| Tier | Treatment | Finding classes (from live scan) | StructureOS actions |
|---|---|---|---|
| **1 — Safe auto-fix** | Batch-fix → build-gate → **one** human approval → PR | `SOS027` unused_imports, `SOS025` dead_code | `unused_imports` via deterministic `cargo fix`; `dead_code` agent-proposed |
| **2 — Structural refactor** | **Per-target** human gate → move → structural-verify → cargo build gate → bounded agent-fixup | `SOS001` god_files, `SOS014` god_methods, `SOS020` import_cycles | `propose_decomposition`/`move`, `decompose_method`/`extract_methods`, `break_cycle` |
| **3 — Advisory triage** | **Per-class** human decision: fix-by-hand-ticket / `dismiss_finding`-with-rationale / defer | `SOS010` complexity, `SOS012` hygiene, `SOS016` test-concentration, `SOS028` composition, `SOS037` packaging, `SOS061` behavioral-risk, `SOS103` large_public_surface, **+ every unrecognized SOS0xx** | `suggest_recommendations`, `dismiss_finding` |

Notes:
- Exact per-code tiering is finalized at implementation against real `get_diagnostics` output; the table above seeds the recognized classes.
- `large_public_surface` (SOS103) starts in Tier 3 triage; promote to a Tier-2 sub-flow later only if a mechanical treatment proves out.
- `dead_code` (SOS025) is *not* as safe as unused imports (false positives: `pub` cross-crate, `#[used]`, feature-gated, reflection). It is **agent-proposed and always shown in the batch diff for human approval** — the approval gate is what makes it safe.

## 5. Unit inventory

| Unit | Kind | Status |
|---|---|---|
| `frontrails/flow.campaign.structureos-findings` | orchestrator (top sequencer) | new |
| `frontrails/flow.findings.sweep-safe` | orchestrator (Tier 1) | new |
| `frontrails/flow.findings.refactor-structural` | orchestrator (Tier 2, budgeted) | new |
| `frontrails/flow.findings.triage-advisory` | orchestrator (Tier 3) | new |
| `frontrails/flow.refactor.god-file` | orchestrator (Tier-2 target) | reuse — mirror from cognitive-max |
| `frontrails/flow.refactor.method` | orchestrator (Tier-2 target) | new (god-file sibling) |
| `frontrails/flow.refactor.cycle` | orchestrator (Tier-2 target) | new (god-file sibling) |
| `frontrails/cap.fix.safe-class` | capability (agent-constrained removal for one class) | new |
| `frontrails/cap.triage.dismiss` | capability (governed dismiss + rationale) | new |
| `frontrails/cap.coordinate.pr-open` | capability (thin, git-based) | new (pack-local, self-contained) |
| `frontrails/cap.coordinate.structureos` | capability | reuse pattern |
| `verify.cargo.cwd` | script | new (pack-local copy) |

## 6. Architecture

**Top sequencer** — `flow.campaign.structureos-findings` (~5-state router; all real work in the tier-flows):

```
scanning ─ scan_repo ─▶ triaging ─ get_diagnostics, build tiered worklist ─▶
  sweep_safe      ─ kind:workflow → flow.findings.sweep-safe ─────────────────┐
  refactor_struct ─ kind:workflow → flow.findings.refactor-structural(budget) ─┤
  triage_advisory ─ kind:workflow → flow.findings.triage-advisory ────────────┤
                                                                               ▼
                        reconciling ─ re-scan; assert every finding fixed-or-dismissed
                          (residual == 0 → done) │ (residual > 0 & budget spent → paused)
```

**Tier 1 — `flow.findings.sweep-safe`** (batch, one human approval):

```
collecting ─ get_diagnostics {SOS027, SOS025} ─▶ gate_empty
  (none → done) │ (some → fixing)
fixing   ─ unused_imports: deterministic `cargo fix`; dead_code: agent-proposed, constrained
           to the exact finding list ─▶
building ─ verify.cargo.cwd (fmt + clippy -D warnings + test) ─▶ build_gate
  (green → approving) │ (red<cap → fixing) │ (red@cap → needs_human)
approving ─ HUMAN reviews the whole batch diff, approves ─▶ opening_pr ─▶ done
```

**Tier 2 — `flow.findings.refactor-structural`** (budgeted, resumable, per-target):

```
scanning ─ scan_repo ─▶ picking ─ pick the single worst structural target ─▶ gate
  (no targets → done) │ (budget spent → paused) │ (else → route)
route ─ by target class ─▶
   god_file   → kind:workflow flow.refactor.god-file(target_path)
   god_method → kind:workflow flow.refactor.method(target)
   cycle      → kind:workflow flow.refactor.cycle(target)
   ↳ on child done → done_count++ → back to scanning
```

Each child sub-flow carries its own human-gate + move + structural-verify + cargo-build-gate + bounded agent-fixup (the god-file pattern). Tier 2 only picks targets and counts against `budget`.

**Tier 3 — `flow.findings.triage-advisory`** (per-class human decision): loop advisory classes; for each, present the class + `suggest_recommendations`; human chooses `dismiss` (→ `cap.triage.dismiss` with a required rationale string, a governed structureos mutation) / defer / fix-by-hand-ticket. Collect the run's decisions into one triage-ledger commit.

## 7. Safety model

- **Read-only until a human gate** in every mutating path (mirrors god-file flow).
- **Gates proportional to risk** — Tier 1: one batch approval; Tier 2: per-target approval; Tier 3: per-class decision. Dismissals are always human-approved with captured rationale; never auto-dismissed.
- **Verification** — `verify.cargo.cwd` mirrors the consumer repo's *actual* CI gate (`cargo fmt --check`, `clippy --workspace -D warnings` — **no `--all-targets`**, matching frontrails CI to avoid false reds — `test`). Soft-fail: always exits 0, emits `{passed, issues, summary}` JSON so a red build routes to fixup, never a hard abort.
- **Budgeted + resumable** — no silent spin; `paused` surfaces residual counts; `reconciling` gates `done` on residual == 0.

## 8. PR cadence

| Tier | PR unit | Rationale |
|---|---|---|
| 1 safe | one batch PR per run | small, uniform, reviewable together |
| 2 structural | **one PR per target** | a single decomposition is already a large, independently-reviewable + revertible diff; batching is unreviewable |
| 3 advisory | one triage-ledger PR per run | dismissals mutate the dismissal ledger + rationale (not source); collect a run's decisions |

PR-open is the pack-local thin `cap.coordinate.pr-open` (git-based) — self-contained, so the pack does not depend on `praxec-meta` being loaded in the consumer's gateway.

## 9. Testing

- **Structural validation** per unit via `praxec.query describe` — state reachability, no dangling transitions, stable `structural_fingerprint`.
- **Capability-harness** tests (pattern from `praxec-meta` `cap.verify.capability-harness` + the pack's `tests/`), driving human-gate transitions with canned `arguments`.
- **Fixture-repo dry-run** — a throwaway cargo crate with seeded findings (one unused import, one dead fn, one small god-file) so the flows execute end-to-end against real structureos output without touching frontrails-product.

## 10. Rollout (local-first, commit per phase)

1. **Plumbing** — `praxec.repo.yaml` + include-migration, `verify.cargo.cwd`, `cap.coordinate.pr-open`, `taxonomy.finding-tiers.yaml`, mirror `flow.refactor.god-file`.
2. **Tier 1** — `flow.findings.sweep-safe` + `cap.fix.safe-class` (ships value immediately).
3. **Tier 2** — `flow.refactor.method` + `flow.refactor.cycle` + `flow.findings.refactor-structural`.
4. **Tier 3** — `flow.findings.triage-advisory` + `cap.triage.dismiss`.
5. **Sequencer** — `flow.campaign.structureos-findings` + reconcile + resumability.

One PR when Sub-project A is coherent and harness-green (per-phase commits inside), with Tier 1 optionally landable as its own earlier PR.

## 11. Scope boundaries (YAGNI)

- **Deferred to B/C:** `cap.coordinate.intentos`, `cap.coordinate.uxos`, and the async-HITL burn-down pattern — A does not touch specs.
- **Not doing:** auto-dismissal; batched structural refactors; bespoke per-SOS-code treatments (unknown → Tier 3 default).
- The pack stays pure config; the campaign runs against the consumer repo via `$.run.repo_root` (praxec 0.0.22 run-ambient).
