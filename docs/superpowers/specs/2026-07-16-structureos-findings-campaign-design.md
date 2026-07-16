# Design — StructureOS Findings-Cleanup Campaign (praxec) — Revised v1

- **Date:** 2026-07-16
- **Status:** Approved design (revised after FMECA/poka-yoke/TRIZ vetting) → ready for implementation plan
- **Repo (home):** `frontrails-praxec-pack`
- **Program:** Operationalize FrontRails *os capabilities as praxec workflows. This spec is **Sub-project A** of three (A = structureos findings cleanup; B = Preveti→IntentOS strategic/compass population; C = uxos+intentos UX vet-and-fix). B and C are separate specs.

---

## 1. Context & goal

The `frontrails-praxec-pack` already exposes every *os action as a governed praxec proxy capability (`intentos.*`, `structureos.*`, `uxos.*`) via `map:` binding, keeping the *os MCP servers pure. `cognitive-architectures-max` already ships the proven orchestration patterns: `cap.coordinate.structureos` and a dogfooded `flow.refactor.god-file` (scan → propose → human gate → move → structural-verify → cargo build gate → bounded agent-fixup).

**Goal:** a praxec campaign that **safely reduces StructureOS findings**, with human governance proportional to each finding class's risk, running against a consumer repo (frontrails-product). "Reduce" — not "drive all to zero": see the DoD in §4.

**Non-goal for A:** anything touching IntentOS/UXOS specs (no async-HITL; those are Sub-projects B/C).

## 2. Vetting outcome — what changed and why

This design was adversarially vetted (FMECA + poka-yoke + TRIZ). The vetting removed real over-engineering; the decisions below are load-bearing, not cosmetic:

- **Reconcile-to-zero DoD was unsound → removed.** With ~2451 warnings + 590 info (mostly advisory), "every finding fixed *or dismissed-with-rationale*" made `dismiss_finding` the cheap path to "done" — a mass-dismissal machine. **Tier 3 is now a read-only report, never required to reach zero** (§4).
- **Agent removed from the safe path.** `dead_code` has false positives (`pub` cross-crate, `#[used]`, feature-gated, reflection). It **moves to Tier 3**; **Tier 1 is now agent-free** (unused imports via deterministic `cargo fix` only). `cap.fix.safe-class` is deleted.
- **Reference, don't copy.** god-file / method / cycle / `verify.cargo` are *generic* Rust+StructureOS refactors, not FrontRails-specific. They are **referenced cross-namespace from `cognitive-max`**, not mirrored into the pack (copying = guaranteed divergence).
- **No breaking migration.** The `praxec.repo.yaml` repo-manifest change is dropped; the campaign ships as an **additive `frontrails-campaign.yaml`** include (flat deep-merge), so existing single-file includers don't break.
- **Stateless re-runs, not durable resume.** Each run scans fresh, does up to `budget` structural targets, stops. "Resume" = run again. Removes a correctness dependency on praxec suspend/resume surviving repo drift.
- **No top sequencer, no `cap.coordinate.pr-open` in v1.** Operators run the tier-flows deliberately; flows commit a branch and a human/CI opens the PR.
- **Observability added** (§9) — the original design defined none.

## 3. Where it lives

Additive, non-breaking: a new `frontrails-campaign.yaml` deep-merges alongside `frontrails.yaml`. No repo-manifest migration; workflow ids are flat top-level keys.

```
frontrails-praxec-pack/
  frontrails.yaml               # (existing) *os connections + proxy exposure
  frontrails-campaign.yaml      # (new) campaign workflows + capabilities
  taxonomy.finding-tiers.yaml   # (new) data: SOS code → tier → treatment
  tests/                        # (existing) harness + fixtures (+ fixture repo)
  docs/superpowers/specs/       # (this file)
```

**Consumer wiring** (documented in README): include `frontrails.yaml` + `frontrails-campaign.yaml`, **and** load `cognitive-max` (for the referenced `flow.refactor.god-file`). The pack stays pure config; the campaign runs against the consumer repo via `$.run.repo_root` (praxec run-ambient).

## 4. Definition of done (revised)

- **Tier 1 (safe):** a run is *done* when every unused-import finding is fixed-and-build-verified or the run produced a clean no-op. Deterministic; converges in one run.
- **Tier 2 (structural):** a run is *done* when it has processed up to `budget` targets (default **3–5**), each ending in a human-gated, build-verified PR or an explicit `needs_human`. Full convergence happens across **stateless re-runs** — scan fresh each run.
- **Tier 3 (advisory):** **never required to reach zero.** The tier *reports* remaining findings; a human may opt-in to fix / `dismiss_finding`-with-rationale / defer. Reporting is the deliverable, not resolution.

There is no global "all findings resolved" gate. The campaign *reduces* actionable debt (Tiers 1–2) and *illuminates* the rest (Tier 3).

## 5. Routing table — data, not control flow

Shipped as `taxonomy.finding-tiers.yaml`, read by the flows. Re-tiering is a table edit. **Invariant: an unknown SOS code defaults to Tier 3 (report) — the flow never auto-mutates a finding it does not recognize**, and unknown-code counts are logged as a first-class signal (§9).

| Tier | v1 treatment | Finding classes | Mechanism |
|---|---|---|---|
| **1 — Deterministic auto-fix** | `cargo fix` → build-gate → batch PR | `SOS027` unused_imports | compiler-driven, **no LLM, no judgment** |
| **2 — Structural refactor** | per-target human gate → referenced sub-flow | `SOS001` god_files | `cognitive-max/flow.refactor.god-file` |
| **3 — Advisory report** | enumerate; human opt-in fix / dismiss-with-rationale / defer | `SOS025` dead_code, `SOS014` god_methods, `SOS020` cycles, `SOS103` large_public_surface, `SOS010` complexity, `SOS012` hygiene, `SOS016` test-concentration, `SOS028` composition, `SOS037` packaging, `SOS061` behavioral-risk, **+ every unrecognized SOS0xx** | report + optional `dismiss_finding` |

**Phase-2 promotions (post-dogfood):** `god_methods` → Tier 2 via new `flow.refactor.method`; `import_cycles` → Tier 2 via new `flow.refactor.cycle`. Until their lossy-edge profiles are characterized by manual dogfooding, they stay in the Tier 3 report — *not* auto-refactored.

## 6. Unit inventory (v1)

| Unit | Kind | Status |
|---|---|---|
| `flow.findings.sweep-safe` | orchestrator (Tier 1) | new |
| `flow.findings.structural` | orchestrator (Tier 2, budgeted, per-target) | new |
| `flow.findings.triage-report` | orchestrator (Tier 3, read-only + optional dismiss) | new |
| `cap.triage.dismiss` | capability (governed dismiss + rationale) | new (optional, opt-in) |
| `taxonomy.finding-tiers.yaml` | data | new |
| `cognitive-max/flow.refactor.god-file` | orchestrator (Tier-2 target) | **reference** (not copied) |
| `cap.coordinate.structureos`, `verify.cargo.cwd` | capability / script | **reference** from cognitive-max |

**Removed vs. the pre-vetting design:** top sequencer, `cap.fix.safe-class`, `cap.coordinate.pr-open`, `praxec.repo.yaml` migration, mirrored refactor flows, durable resume.
**Deferred to Phase 2+ (post-dogfood):** `flow.refactor.method`, `flow.refactor.cycle`, and (only if a real need emerges) a thin top sequencer.

## 7. Architecture

**Tier 1 — `flow.findings.sweep-safe`** (deterministic, agent-free, one batch PR):
```
collecting ─ get_diagnostics {SOS027} ─▶ gate_empty
  (none → done) │ (some → fixing)
fixing   ─ deterministic `cargo fix` (unused imports; NO agent) ─▶
building ─ verify.cargo.cwd (fmt + clippy -D warnings + test) ─▶ build_gate
  (green → reporting → done+branch) │ (red → needs_human)   ← cargo fix should never red; a red is a real signal
```
A red build here is not a fixup loop (there's no agent to fix) — it surfaces to `needs_human`, because a deterministic `cargo fix` breaking the build is itself a finding.

**Tier 2 — `flow.findings.structural`** (budgeted, stateless, per-target):
```
scanning ─ scan_repo ─▶ picking ─ pick the single worst god_file target ─▶ gate
  (no targets → done) │ (budget spent → done+report residual) │ (else → refactor)
refactor ─ kind:workflow → cognitive-max/flow.refactor.god-file(target_path)
  ↳ child carries its own human-gate + move + structural-verify + cargo-build-gate + bounded fixup
  ↳ on child done → done_count++ → back to scanning (fresh)
```
No durable suspend: when `budget` is spent, the run **ends** and reports residual target count; you run it again to continue.

**Tier 3 — `flow.findings.triage-report`** (read-only + opt-in):
```
scanning ─ scan_repo ─▶ enumerating ─ get_diagnostics (all non-Tier1/2 classes)
  + suggest_recommendations ─▶ reporting ─ emit the per-run report (§9) ─▶ gate
  (no opt-in → done) │ (human opts in → decide)
decide ─ HUMAN per class: dismiss (→ cap.triage.dismiss + required rationale) / defer / ticket ─▶ done
```
Dismissal is always human-approved with a captured rationale; never auto. The report — not resolution — is the deliverable.

## 8. Safety model

- **Read-only until a human gate** in every mutating path (Tier 1 is deterministic + build-gated; Tier 2 inherits the god-file gate; Tier 3 dismiss is human-approved).
- **No LLM in the "safe" tier** — Tier 1 is `cargo fix` only.
- **Gates proportional to risk** — Tier 1: one batch approval; Tier 2: per-target; Tier 3: per-class opt-in.
- **Verification** — `verify.cargo.cwd` (referenced) mirrors the consumer repo's *actual* CI gate (`cargo fmt --check`, `clippy --workspace -D warnings` — **no `--all-targets`**, matching frontrails CI to avoid false reds — `test`). Soft-fail JSON `{passed, issues, summary}`.
- **Stateless re-runs** — scan-fresh at run start; never carry a target list across runs; single-target-per-iteration; low default budget so a run is a reviewable session.

## 9. Observability (new)

Every run emits a **structured report** (JSON + a markdown render), committed with the run's branch / attached to the PR:

```
findings_in:        { by_code: {...}, total }
tier_routing:       { tier1, tier2, tier3, unknown_codes: [codes] }
tier1:              { fixed, gate_result }
tier2:              { targets_attempted, succeeded, needs_human, residual }
tier3:              { reported, dismissed, deferred }
approvals_count:    n
```

- **unknown_codes** is first-class: a spike signals a StructureOS version drift silently re-routing codes to the Tier-3 default.
- The praxec **`observe`** audit-event stream (`praxec.query observe`) is the live trace for every transition/gate.
- Detectable in production: bad fixes (build-gate red), premature "done" (residual counts in report), mass-dismissal (dismissed vs fixed ratio), policy drift (unknown_codes spike).

## 10. PR cadence

| Tier | PR unit | Rationale |
|---|---|---|
| 1 safe | one batch PR per run | small, uniform, compiler-trusted `cargo fix` diff |
| 2 structural | **one PR per target** | a single decomposition is already large + independently revertible |
| 3 advisory | triage-report artifact (+ dismissal-ledger commit if any) | reporting, not source change |

Flows **commit a branch and stop**; a human / CI opens the PR (no `cap.coordinate.pr-open` in v1 — avoids duplicating `praxec-meta`'s capability).

## 11. Testing

- **Structural validation** per unit via `praxec.query describe` — state reachability, no dangling transitions, stable `structural_fingerprint`.
- **Capability-harness** tests (pattern from `praxec-meta` `cap.verify.capability-harness` + the pack's `tests/`), driving human-gate transitions with canned `arguments`.
- **Fixture-repo dry-run** — a throwaway cargo crate seeded with one unused import, one dead fn, one small god-file, one 2-node cycle — so the flows execute end-to-end against real StructureOS output without touching frontrails-product.

## 12. Rollout — simplest viable v1, resequenced

Prove one thin vertical slice before expanding. Local-first, commit per phase:

1. **Slice** — `frontrails-campaign.yaml` (additive include) + `taxonomy.finding-tiers.yaml` + reference wiring for `verify.cargo.cwd` / `cap.coordinate.structureos`.
2. **Tier 1 vertical** — `flow.findings.sweep-safe` (unused imports → `cargo fix` → verify → branch). Prove the whole praxec plumbing against the fixture repo end-to-end. **This is the go/no-go slice.**
3. **Tier 2** — `flow.findings.structural` referencing `cognitive-max/flow.refactor.god-file` (god_files only, one budgeted target, human-gated).
4. **Tier 3** — `flow.findings.triage-report` + per-run report + optional `cap.triage.dismiss`.

One PR when v1 is coherent and harness-green (per-phase commits inside); Tier 1 optionally landable as its own earlier PR.

**Phase 2+ (post-dogfood, separate specs/PRs):** `flow.refactor.method`, `flow.refactor.cycle` (each after its lossy-edge profile is characterized manually), promoting god_methods/cycles from Tier-3-report to Tier-2. A top sequencer only if operators ask for one-shot chaining.

## 13. Scope boundaries (YAGNI)

- **Deferred to B/C:** `cap.coordinate.intentos`, `cap.coordinate.uxos`, async-HITL — A does not touch specs.
- **Not doing:** reconcile-to-zero DoD; agent-driven dead-code removal; auto-dismissal; batched structural refactors; durable cross-run suspend; a top sequencer; a pack-local `cap.coordinate.pr-open`; the `praxec.repo.yaml` migration; bespoke per-SOS-code treatments (unknown → Tier 3 report).
- The pack stays pure config; generic refactor primitives live in `cognitive-max` and are referenced, not copied.
