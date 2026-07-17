# Sub-projects B & C — Preveti→compass + uxos vet-and-fix — Design

**Date:** 2026-07-17
**Repo:** `frontrails-praxec-pack` (pure Praxec config; FrontRails never modified)
**Status:** Approved-in-progress (forks resolved; see §2)
**Builds on:** the coordination foundation (PR #5) — `flow.intent.propose_and_park`
+ exposed `intentos.simulate_and_project`.

## 1. Purpose

The last two sub-projects of "operationalize the `*os` capabilities as praxec
workflows." Both are thin consumers of the shipped foundation:

- **B — Preveti → IntentOS strategic/compass population.** Drive a Preveti/Simuli
  run and surface what it projected into the OpportunityCompass.
- **C — uxos + IntentOS UX vet-and-fix.** Audit a real surface with uxos and turn
  each finding into a grounded IntentOS proposal via the foundation tail.

## 2. Resolved forks

- **B is proven against a LIVE Preveti**, not a mock (owner decision — mocking
  risks diverging from real behavior; Preveti is the owner's sibling app,
  runnable locally). Stand-up: `sim_host` in-memory with `SEED_DEMO=1` serves the
  REST API on `:8080` and seeds a known full-tier demo API key
  (`demo.preveti-demo-key`); deterministic runs complete without LLM provider
  keys. So the E2E points intentos at `PREVETI_BASE_URL=http://localhost:8080`
  + `PREVETI_API_KEY=demo.preveti-demo-key`.
- **C proposes-and-records** each finding's outcome (`applied` **or**
  `pending_approval`), budget-capped — honoring the foundation's verified
  three-outcomes reality (a headless propose auto-applies; it parks only through
  a HITL-configured context or a gate-crossing change).

## 3. Ground truth (verified 2026-07-17)

- `intentos.simulate_and_project` (exposed on `frontrails.yaml`) is a **network**
  handler: `start_run(objective, audience_json) → await_run (≤120s) → project_run
  → persist_projected_intent` (writes compass + actors into the spec dir). Params
  `{ objective: string, audience_json: object, spec_path?, external_id? }`;
  returns `{ run_id, status, summary }`. It **persists directly** (no park).
- `flow.intent.propose_and_park` (`frontrails-coordination.yaml`): `use.inputs
  { source_findings, rationale }` → `use.outputs { parked_tickets, applied,
  grounded_refs }`. Records whichever propose outcome occurs.
- `uxos.audit` / `uxos.extract` are read-only and spec-root-independent.

## 4. Deliverables

Both additive; **zero FrontRails diff**.

### 4.1 B — `flow.strategy.populate-compass` (new `frontrails-strategy.yaml`)

An agent-driven workflow: `simulating → reporting → done`.

- `simulating` (`actor: agent`) — call `intentos.simulate_and_project` with the
  caller's `{ objective, audience_json }` (inputs, with sane defaults). It runs
  Preveti and persists the projection; capture `run_id` / `status` / `summary`.
- `reporting` (`actor: agent`) — surface what landed. The projected compass
  counts (segments / arena_frames / errc_moves / actors) are already in the sim
  `summary`. **Compass frames live in the OpportunityCompass (`compass.yaml`),
  NOT the intent harness — so `search_intent_harness` does not list
  arena/segment/errc frames** (verified live); call `intentos.status` to confirm
  the compass/gate state, and use `search_intent_harness` only for projected
  `actor` entities (those land in the harness). Emit a report: run id,
  projected-entity counts (from `summary`), compass gate state. Read-only.

`use.inputs { objective: string (default a dev-cost exemplar), audience_json:
object (default {}) }`; `use.outputs { run_id, projected_summary,
compass_entities: [entity.id] }`.

**Hand-off note:** `simulate_and_project` persists compass indicators through the
fail-closed `compass_write` gate (evidence-first, `origin: simuli`) — the
workflow does not re-decide that gate.

### 4.2 C — `flow.ux.vet-and-fix` (new `frontrails-ux.yaml`)

A budgeted, stateless vet-then-fix loop (mirrors Tier-2's budget discipline):

- `auditing` (`actor: agent`) — `uxos.extract` a capture → `uxos.audit` the IR →
  collect the findings (a11y/flow/dark-pattern blockers) into `audit_findings`.
- `dispatching` — for each finding **up to `budget`**, `kind: workflow` into
  `flow.intent.propose_and_park` (`use.inputs.source_findings` = the finding,
  `rationale` = the uxos verdict). Accumulate its `applied` / `parked_tickets` /
  `grounded_refs` outputs.
- `reporting` — emit a vet-and-fix report: findings vetted, changes applied,
  tickets parked, residual findings beyond budget.

`use.inputs { capture_html: string, budget: integer (default 3) }`; `use.outputs
{ applied, parked_tickets, grounded_refs, residual_findings }`. `budget` is a
declared input with a `default` (never seeded in `initialContext` — "initialContext
wins" constraint).

### 4.3 E2E

- **B** (`tests/preveti-compass-e2e.md`): stand up `sim_host` (in-memory,
  `SEED_DEMO=1`, `:8080`); point intentos at an isolated `$TMP` spec copy +
  `PREVETI_BASE_URL`/`PREVETI_API_KEY`; drive `flow.strategy.populate-compass`;
  assert (1) a `run_id` returned with `status` completed, (2) the projection
  persisted — `$TMP/compass.yaml` contains a frame with `source simuli` + the
  returned `run_id` (compass frames are in `compass.yaml`, not queryable via
  `search_intent_harness`), (3) zero FrontRails diff. Runs against the **real**
  Preveti REST API.
- **C** (`tests/ux-vetfix-e2e.md`): reuse the foundation fixtures
  (`ux-capture/index.html` + `intent-spec/intentos.yaml`) in an isolated `$TMP`;
  drive `flow.ux.vet-and-fix`; assert the uxos finding produced a grounded change
  accepted (applied or parked) + zero FrontRails diff.

## 5. Non-goals

- No changes to `simulate_and_project` / the compass gate / IntentOS internals.
- No re-encoding of IntentOS gates or the Preveti projection logic (both stay
  authoritative in FrontRails / Simuli).
- C does not force a park (that needs a HITL context); it records the real outcome.
- B does not mint/manage Preveti tenants or keys — it consumes a configured
  endpoint + key.

## 6. Praxec authoring constraints (carried)

Blessed script roots for any `scripts:`; literal subjects (no `$.context.*` in
`subject:`); `kind: mcp` output is the raw envelope at `$.output.content.0.text`;
overridable inputs declared under `inputs: {default}` not `initialContext`;
`kind: workflow` `definitionId` referenced unprefixed (`include:` convention),
resolution unchecked by `check`/`fuzz`; the `use.outputs` projection is
host-`$.context.*` LHS ← callee-field RHS.

## 7. Acceptance

- B and C resolve at `praxec check` + no wedge at `praxec fuzz`.
- B live-drives against the real Preveti REST API and lands compass entities.
- C live-drives on the foundation fixtures and produces a grounded accepted change.
- `git -C <frontrails-product> diff --stat crates/` empty after both.
