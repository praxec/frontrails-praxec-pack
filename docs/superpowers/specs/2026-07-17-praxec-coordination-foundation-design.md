# Praxec Coordination Foundation — Design

**Date:** 2026-07-17
**Repo:** `frontrails-praxec-pack` (pure Praxec configuration; FrontRails is never modified)
**Status:** Approved (design)

## 1. Purpose

Operationalizing the FrontRails `*os` capabilities as Praxec workflows breaks
into three sub-projects:

- **A** — StructureOS findings cleanup (Tiers 1 & 2 shipped; Tier 3 pending).
- **B** — Preveti → IntentOS strategic/compass population.
- **C** — uxos + IntentOS UX vet-and-fix.

B and C share one missing piece: a reusable way to take findings from a *source*
(a uxos audit, or a Preveti/Simuli strategy run) and drive them into IntentOS as
**grounded proposals**, respecting the async-HITL approval boundary. This spec
defines that shared enabling layer — the **coordination foundation** — plus the
one IntentOS action B needs exposed, and proves the seam end to end. B and C
themselves are out of scope here (their own later specs).

## 2. Ground truth (verified 2026-07-17)

- `simulate_and_project` is a **live IntentOS MCP action on `dev`** (present in
  `dispatch.rs`, `tool_loading.rs`, `params/`, and `handlers/`). B's engine —
  Simuli run → OpportunityCompass projection through the fail-closed
  `compass_write` gate — already exists inside FrontRails.
- **Async-HITL is shipped**: a mutating `intentos.propose` that cannot reach a
  human returns `{ status: "pending_approval", ticket_id }` and parks a durable
  approval ticket; `intentos.propose_status { ticket_id }` polls it (read-only).
- The existing pack workflows (`frontrails_spec`, `frontrails_burndown`,
  `frontrails_ux_conformance`) already exist and already handle a parked ticket
  correctly (record the id, keep going).
- **Hard boundary:** only IntentOS-desktop's gRPC handler can mint the
  `ApprovalToken`. Praxec — an orchestration agent — **structurally cannot
  approve** an IntentOS ticket. This is the "agent cannot self-approve"
  invariant, preserved by design. A pack-driven mutation can only *park and hand
  off* to a human; it can never *land* inside a headless run.

## 3. Scope decisions (locked)

- **Thin enabling layer** (not a formal coordination primitive, not an approval
  bridge): expose the missing action + one shared agent-driven mutation tail
  that B and C both call. Hand-off approval semantics.
- **Validated by one live E2E**: a minimal, fully-local uxos → propose slice
  whose success signal is a real parked ticket (no external services, no
  approval needed). Structural `check`/`fuzz` alone is insufficient — it does
  not validate `kind: workflow` resolution or runtime parking (the exact gap
  that let defects through in campaign Tiers 1–2 until the E2E ran).

## 4. Deliverables

All additive to the pack; **zero FrontRails diff**.

### 4.1 Expose `intentos.simulate_and_project`

A new capability in `frontrails.yaml` `capabilities:` (+ `proxy.expose` with
tags/aliases), mirroring the existing `intentos.*` capability shape:

```yaml
intentos.simulate_and_project:
  title: "IntentOS — run Preveti/Simuli and project into the compass"
  description: "Start a Simuli run and project the vetted result into the OpportunityCompass (evidence-first, via the fail-closed compass_write gate)."
  tags: [intentos, write, preveti]
  inputSchema:
    type: object
    required: [params]
    properties:
      params: { type: object }
  executor:
    kind: mcp
    connection: intentos
    tool: "intentos"
    map:
      action: "simulate_and_project"
      params: "$.arguments.params"
```

Exposing it now (even though B is deferred) is part of the enabling layer and
makes B trivial later. Its exact `params` schema is documented from the live
action at implementation time — not guessed.

### 4.2 `flow.intent.propose_and_park` — the reusable mutation tail

An agent-driven sub-workflow, referenced by callers via
`executor: { kind: workflow, definitionId: flow.intent.propose_and_park, use: {...} }`
(exactly as Tier 2 references `flow.refactor.god-file`). It is the memory's
aspirational `cap.coordinate.intentos`, realized as a **workflow** (not a
capability, because it is multi-step). Source-agnostic — that is what makes it
the foundation.

**Data contract (the OS boundary):**

```
use.inputs:
  source_findings : [ { summary, evidence, target_hint } ]   # opaque list from the caller
  rationale       : string                                    # why these changes are proposed
use.outputs:
  parked_tickets  : [ ticket_id ]        # what a human must approve in IntentOS-desktop
  applied         : [ change_summary ]   # any change that applied synchronously (attended)
  grounded_refs   : [ entity.id ]        # the real entity id(s) each proposal grounded on
```

**States (all `actor: agent` — the running LLM drives; IntentOS' gates and
grounding-verification stay authoritative):**

- `grounding` — for each finding, call `intentos.search_intent_harness` to
  resolve `target_hint` to real entity id(s). A proposal that cannot be grounded
  is skipped and reported, never fabricated.
- `proposing` — call `intentos.propose` with the change + `grounded_in` = the
  real ids + `rationale`. On `status: pending_approval`, record the `ticket_id`
  and continue; on `status: approved` (applied), record the change summary; never
  block, never retry a parked change, never self-approve.
- `reporting` — emit `parked_tickets` / `applied` / `grounded_refs` into the
  workflow context (the `use.outputs` projection) and terminate.

**Propose has three outcomes, not one (verified live 2026-07-17 — do not assume
"always parks"):** a propose resolves to `approved` (the change **applies
directly**), `pending_approval` (**parks** a ticket — the hand-off path), or
`needs_review` (a gate-crossing/attestation change opens a HITL review). Which
one occurs depends on config and the change's gate impact, **not** on the seam
being correct: a benign, non-gate-crossing propose against a headless intentos
with **no HITL mechanism wired auto-applies** (nothing is present to gate it) —
the "agent cannot self-approve" invariant only bites where a HITL mechanism is
active (the gateway + desktop) or the change crosses a gate. This is why
`use.outputs` carries **both** `parked_tickets` (park path) and `applied`
(auto-apply path); the flow records whichever occurs. `needs_review` (gate
crossing) can wedge headlessly on the not-yet-ticketized `review_strategy`
(SP-B) and is out of scope for the agent-free path.

`maxChainDepth` caps runaway/token burn. The transitions available are exactly
the exposed `intentos.*` capabilities (`search_intent_harness`, `propose`,
`propose_status`); no journey or gate logic is re-encoded.

**Output mechanism (implementation note):** how an `actor: agent` state writes
structured `use.outputs` (vs. a script/`kind: mcp` executor writing them) is
confirmed against the running Praxec engine during implementation; if the agent
cannot set outputs directly, a terminal `run.*` script reads the accumulated
ticket ids from context and emits the projection. This is a build-time
verify-item, not a design fork.

### 4.3 The E2E proof

**Fixtures** (copied to a temp dir + `git init` at run time, per the established
campaign E2E pattern — never mutating the pack or FrontRails):

- `tests/fixtures/ux-capture/` — a tiny HTML surface with **one** known,
  detectable UX defect (e.g. an icon-only button with no accessible name),
  yielding a real uxos a11y/tap-target finding. Feeds `uxos.extract` →
  `uxos.audit`.
- `tests/fixtures/intent-spec/` — a minimal `.frontrails/intent/` holding a few
  real entities (a journey + state + requirement) so `search_intent_harness`
  returns genuine ids to ground the proposal against. Without a real grounding
  target the proposal cannot be grounded and cannot park.

**Validating consumer** — `examples/ux_vet_fix_min.yaml`, a minimal composition
**clearly labeled a foundation proof, not sub-project C**: `uxos.audit`
(read-only) → hand one finding to `flow.intent.propose_and_park`.

**Drive & assert** (`tests/foundation-propose-park-e2e.md`): drive through the
`praxec` MCP tool following its HATEOAS links (the same guided-dogfood method
used for uxos/intentos). The drive is agent-guided; the **pass criteria are
deterministic on-disk artifacts**, each a single assertion:

1. `<temp-fixture>/.frontrails/**/approvals/<ticket_id>.yaml` exists after the run.
2. `intentos.propose_status { ticket_id }` reports the ticket as still pending
   (not yet applied/rejected) — the exact response field is read from the live
   action at implementation time.
3. the parked proposal's `grounded_in` names a real entity id from the fixture spec.

Because the run targets the temp fixture, the **zero-FrontRails-diff poka-yoke**
(`git -C <frontrails-product> diff --stat crates/` is empty) holds automatically.

## 5. Non-goals (YAGNI)

- **No approval bridge.** Praxec drives to the park point and hands off; it never
  approves (the `ApprovalToken` invariant forbids it).
- **No deterministic finding→proposal contract.** The tail is agent-driven; the
  only formal seam is the `use.inputs/outputs` shape.
- **Sub-projects B and C are not built here.** Only the enabling layer + the
  exposed action + the minimal C-flavored E2E slice.
- **No FrontRails-product changes.** Pure pack config.
- **No re-encoding of IntentOS gates/journey logic.** FrontRails stays
  authoritative.

## 6. Praxec authoring constraints (carried from campaign work)

- Blessed script roots (SPEC §22.4) for any `scripts:` key
  (build/test/deploy/format/lint/install/verify/run/inspect/audit/release/
  migrate/ci); a report/output script uses a `run.*` root.
- `subject:` does **not** resolve `$.context.*` at runtime — literal subjects
  only; pass runtime values via templated `args:` (jsonpath `$.context.x`, not
  Handlebars).
- `kind: mcp` output is the raw MCP text envelope at
  `$.output.content.0.text` (no `structuredContent`) — jq-parse in a script.
- `check`/`fuzz` do **not** validate `kind: workflow` `definitionId` resolution;
  keep the referenced id consistent with the mount mode (`include:` → unprefixed).

## 7. Acceptance

- `intentos.simulate_and_project` resolves at `praxec check` and its `params`
  schema matches the live action.
- `flow.intent.propose_and_park` resolves at `praxec check`; `praxec fuzz`
  reports no wedge/livelock on its mock-executor scenarios.
- The E2E produces a real parked ticket meeting all three assertions above.
- `git -C <frontrails-product> diff --stat crates/` is empty after the E2E.
