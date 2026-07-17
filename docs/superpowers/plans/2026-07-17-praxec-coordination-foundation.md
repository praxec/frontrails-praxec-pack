# Praxec Coordination Foundation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the thin enabling layer that lets sub-projects B & C drive source findings into IntentOS as grounded, parked proposals — an exposed `simulate_and_project` action plus one reusable agent-driven mutation tail — proven end to end by a local uxos→propose slice.

**Architecture:** Pure Praxec configuration additive to `frontrails-praxec-pack`; FrontRails is never modified. A new `frontrails-coordination.yaml` (deep-merged like `frontrails-campaign.yaml`) holds the reusable `flow.intent.propose_and_park` sub-workflow; `frontrails.yaml` gains one capability. A minimal example + fixtures prove the seam by parking a real approval ticket.

**Tech Stack:** Praxec workflow YAML (`praxec` binary: `check` / `fuzz` / `command` + the `praxec` MCP tool); the `intent` and `uxos-mcp` MCP servers; bash + jq for report/probe scripts.

## Global Constraints

- **Zero FrontRails diff.** No change under `crates/` of `frontrails-product`. Poka-yoke: `git -C /home/mc/working/frontrails-product diff --stat crates/` is EMPTY after every task.
- **Additive only.** New config lives in `frontrails-coordination.yaml` (new) + additive blocks in `frontrails.yaml`. Never migrate `praxec.repo.yaml`.
- **Blessed script roots (SPEC §22.4):** any `scripts:` key starts with build/test/deploy/format/lint/install/verify/run/inspect/audit/release/migrate/ci. Report/emit scripts use a `run.*` root.
- **`subject:` does NOT resolve `$.context.*` at runtime** — literal subjects only; pass runtime values via templated `args:` (jsonpath `$.context.x`, NOT Handlebars `{{ }}`).
- **`kind: mcp` output is the raw MCP text envelope** at `$.output.content.0.text` (no `structuredContent`) — jq-parse in a script.
- **`initialContext` WINS over caller `input.X`** — any overridable input must be declared under `inputs: { X: { default: … } }` and NOT seeded in `initialContext` (praxec-core runtime.rs "initialContext wins").
- **`check`/`fuzz` do NOT validate `kind: workflow` `definitionId` resolution** — a wrong id fails only at dispatch. Ids are referenced unprefixed (this pack's `include:` convention).
- **Live action facts (verified 2026-07-17):** `intentos.simulate_and_project` params = `{ objective: string(req), audience_json: json(req), spec_path?: string, external_id?: string }`. `intentos.propose` on a parked change returns `{ status: "pending_approval", ticket_id }`. `intentos.propose_status { ticket_id }` returns `{ status: "pending"|"applied"|…, ticket_id, detail, … }`. A parked ticket persists at `<state_root>/approvals/<ticket_id>.yaml`.
- **Approval is hand-off.** Praxec never approves (it cannot mint the `ApprovalToken`). The tail parks and reports; a human approves in IntentOS-desktop out of band.
- **Test hygiene:** E2E assertions are atomic (one assertion each); format only the files you touch.
- **The `praxec` binary** is at `/home/mc/.cargo/bin/praxec`; `scripts/check.sh` wraps `check`/`fuzz` (`FLOWGATE` env or `mcp-praxec` on PATH — export `FLOWGATE=/home/mc/.cargo/bin/praxec`).

---

### Task 1: Expose `intentos.simulate_and_project`

Additive, independent of everything else — the one action B needs, exposed now so B becomes trivial later. Nothing consumes it in this plan; it must simply resolve at `check`.

**Files:**
- Modify: `frontrails.yaml` — add the capability to `capabilities:` and an entry to `proxy.expose`.

**Interfaces:**
- Produces: capability `intentos.simulate_and_project` (aliases: `simulate`, `preveti`, `compass_populate`).

- [ ] **Step 1: Add the capability block.** In `frontrails.yaml` `capabilities:`, after `intentos.finish`, add (mirrors the existing `intentos.propose` shape — `map:` pins the action + templates `params`):

```yaml
  intentos.simulate_and_project:
    title: "IntentOS — run Preveti/Simuli and project into the compass"
    description: >
      Start a Preveti/Simuli simulation run, wait for it, and project the vetted
      result into the OpportunityCompass (evidence-first, via the fail-closed
      compass_write gate). params: { objective: string, audience_json: object,
      spec_path?: string, external_id?: string }. Network + write.
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

- [ ] **Step 2: Expose it.** In `frontrails.yaml` `proxy.expose:`, after the `intentos.finish` line, add:

```yaml
    - { capability: intentos.simulate_and_project, aliases: [simulate, preveti, compass_populate] }
```

- [ ] **Step 3: Validate.** Run:

```bash
export FLOWGATE=/home/mc/.cargo/bin/praxec
"$FLOWGATE" check --config frontrails.yaml
```

Expected: exit 0, `validation: ok`; `intentos.simulate_and_project` appears in the capabilities list.

- [ ] **Step 4: Confirm zero FrontRails diff.** Run: `git -C /home/mc/working/frontrails-product diff --stat crates/` → Expected: empty.

- [ ] **Step 5: Commit.**

```bash
git add frontrails.yaml
git commit -m "feat(coordination): expose intentos.simulate_and_project (enabling layer for sub-project B)"
```

---

### Task 2: Resolve the sub-workflow output mechanism (spike)

The spec flagged one build-time unknown: **how an agent-driven callee returns accumulated results (`parked_tickets`) to its caller via `use.outputs`.** Praxec has repeatedly passed `check` but failed at runtime (literal subjects, arg templating), so resolve this empirically on a throwaway probe BEFORE authoring the real flow. This task produces a notes file, not shipped config.

**Files:**
- Create (throwaway, git-ignored scratch): `/tmp/claude-1000/.../scratchpad/probe-outputs.yaml`
- Create: `docs/superpowers/notes/2026-07-17-praxec-output-binding.md` (records the decision)

**Interfaces:**
- Produces: a confirmed pattern for (a) capturing a `kind: mcp` transition's response field into workflow context, and (b) projecting a callee context value to the caller via `use.outputs`. Task 3 consumes this pattern verbatim.

- [ ] **Step 1: Read the reference patterns.** Read `frontrails-campaign.yaml` `flow.findings.structural` — note how a `kind: mcp` transition binds its output into context (`scan_text: "$.output.content.0.text"`) and how the `refactoring` transition passes `use: { inputs: { target_path: "$.context.worst_path" } }` to the god-file flow. Read the god-file flow's `inputs:` declaration (callee side). Confirm no example projects `use.outputs` back — that is the gap this spike closes.

- [ ] **Step 2: Write the probe.** Create `probe-outputs.yaml` in the scratchpad: a caller workflow with one `kind: workflow` transition into a tiny callee. The callee has a single `kind: script` transition (subject `run.probe.emit`, body `printf '{"ticket_id":"tkt-probe-1"}'`) that binds `emitted: "$.output.content.0.text"` (or the script's stdout path) into context, then a terminal state. The caller's transition declares `use: { outputs: { captured: "$.<callee-ctx>.emitted" } }`. Include a second variant where the callee sets context via an `{add}`/assignment expression, in case output-binding differs for script vs agent transitions.

```yaml
# probe-outputs.yaml (scratch — determines the real binding for Task 3)
workflows:
  probe.caller:
    initialState: calling
    states:
      calling:
        transitions:
          go:
            target: done
            executor:
              kind: workflow
              definitionId: probe.callee
              use:
                inputs: { seed: "$.context.seed" }
                outputs: { captured: "$.context.emitted" }   # <-- the thing under test
      done: { terminal: true }
    initialContext: { seed: "x", captured: "" }
  probe.callee:
    inputs: { seed: { type: string, required: false, default: "x" } }
    initialState: emitting
    states:
      emitting:
        transitions:
          emit:
            target: fin
            executor: { kind: script, subject: "run.probe.emit", args: [] }
            # bind the script stdout into callee context under `emitted`
      fin: { terminal: true }
    initialContext: { emitted: "" }
scripts:
  run.probe.emit:
    verb: run
    body: |
      #!/usr/bin/env bash
      printf '{"ticket_id":"tkt-probe-1"}\n'
```

- [ ] **Step 3: Run the probe and observe.** Run:

```bash
export FLOWGATE=/home/mc/.cargo/bin/praxec
"$FLOWGATE" check --config probe-outputs.yaml && \
"$FLOWGATE" command --config probe-outputs.yaml --definition probe.caller --input '{}'
```

(If the CLI flag names differ, discover them with `"$FLOWGATE" command --help`.) Observe whether the caller's final `context.captured` == the callee's emitted value. Record which binding path actually populates it (`$.context.emitted`, `$.output…`, or an `{add}` accumulation), and how a script transition's stdout lands in context.

- [ ] **Step 4: Resolve the propose-response capture.** Confirm from `frontrails-product/crates/intentos-apps/src/mcp/tools/handlers/propose.rs` (or a live `intentos.propose` call) the exact JSON path to `ticket_id` in a `pending_approval` response envelope (the `kind: mcp` raw text at `$.output.content.0.text` → jq `.ticket_id`). Note it.

- [ ] **Step 5: Record the decision.** Write `docs/superpowers/notes/2026-07-17-praxec-output-binding.md` with: the confirmed `use.outputs` projection syntax, how to capture a `kind: mcp`/`kind: script` transition output into context, and the propose `ticket_id` jq path. Task 3 uses these verbatim.

- [ ] **Step 6: Commit the note.**

```bash
git add docs/superpowers/notes/2026-07-17-praxec-output-binding.md
git commit -m "docs(coordination): resolve praxec agent/callee output-binding for propose_and_park"
```

---

### Task 3: Author `flow.intent.propose_and_park`

The reusable mutation tail. Uses the binding pattern from Task 2's note.

**Files:**
- Create: `frontrails-coordination.yaml` (new additive config file)
- Modify: `examples/campaign-check.yaml` → copy to / create `examples/coordination-check.yaml` that includes `frontrails.yaml` + `frontrails-coordination.yaml` (for `check`/`fuzz`)

**Interfaces:**
- Consumes: `intentos.search_intent_harness`, `intentos.propose`, `intentos.propose_status` (already exposed in `frontrails.yaml`); the output-binding pattern from Task 2.
- Produces: workflow `flow.intent.propose_and_park` with `use.inputs { source_findings: array, rationale?: string }` and `use.outputs { parked_tickets: array, applied: array, grounded_refs: array }`.

- [ ] **Step 1: Author the sub-workflow.** Create `frontrails-coordination.yaml`. Declare `inputs:` (source_findings required; rationale optional with a default — NOT in `initialContext`, per the Global Constraint). States `grounding → proposing → reporting → done`, all `actor: agent`, transitions bound to the exposed `intentos.*` capabilities. Seed `parked_tickets: []`, `applied: []`, `grounded_refs: []` in `initialContext`. A terminal `reporting` step emits the outputs using the Task-2 pattern. Include header comments citing the design spec + the hand-off boundary. State goals (verbatim intent):
  - `grounding`: "For each finding in `source_findings`, call `search_intent_harness` to resolve `target_hint` to a real entity id. Never fabricate an id; a finding that cannot be grounded is recorded in `grounded_refs` as unresolved and skipped."
  - `proposing`: "Call `propose` with the change grounded in the real id(s) + `rationale`. On `status=pending_approval`, append `ticket_id` to `parked_tickets` and continue. Never block, never retry a parked change, never self-approve. If a change applied synchronously, append its summary to `applied`."
  - `reporting`: "Emit `parked_tickets` / `applied` / `grounded_refs` and terminate."
- [ ] **Step 2: Author the check harness.** Create `examples/coordination-check.yaml`:

```yaml
include:
  - frontrails.yaml
  - frontrails-coordination.yaml
```

- [ ] **Step 3: Validate resolution.** Run:

```bash
export FLOWGATE=/home/mc/.cargo/bin/praxec
"$FLOWGATE" check --config examples/coordination-check.yaml
```

Expected: exit 0, `validation: ok`, `flow.intent.propose_and_park` listed in workflows, and the three `intentos.*` transitions resolve.

- [ ] **Step 4: Fuzz for wedges.** Run:

```bash
"$FLOWGATE" fuzz --config examples/coordination-check.yaml
```

Expected: no wedge/livelock/engine-error reported for `flow.intent.propose_and_park` (mock executors).

- [ ] **Step 5: Confirm zero FrontRails diff.** `git -C /home/mc/working/frontrails-product diff --stat crates/` → empty.

- [ ] **Step 6: Commit.**

```bash
git add frontrails-coordination.yaml examples/coordination-check.yaml
git commit -m "feat(coordination): flow.intent.propose_and_park — reusable grounded-propose+park tail"
```

---

### Task 4: E2E fixtures + validating consumer

Build the two fixtures and the minimal composition that proves the seam. No live run yet (that is Task 5) — this task ends when the example resolves at `check`.

**Files:**
- Create: `tests/fixtures/ux-capture/index.html` (one detectable a11y defect)
- Create: `tests/fixtures/intent-spec/.frontrails/intent/…` (a minimal real spec — a journey + state + requirement)
- Create: `examples/ux_vet_fix_min.yaml` (the validating consumer)
- Create: `tests/foundation-propose-park-e2e.md` (the guided procedure + 3 atomic assertions)

**Interfaces:**
- Consumes: `uxos.audit` (exposed), `flow.intent.propose_and_park` (Task 3).
- Produces: a runnable, self-contained E2E.

**Fixture theme (do not deviate):** the user for this fixture is a **cost-conscious developer** — the actual persona of a developer-tooling product — NOT a separate "Shopper"/consumer persona. Do not model a shopping/checkout/cart flow and do not introduce an `actor.shopper`-style consumer. Both fixtures are about a developer managing build/CI cost.

- [ ] **Step 1: UX capture fixture.** Create `tests/fixtures/ux-capture/index.html`: a minimal **developer-tooling** page (a CI/build cost-controls view) with exactly one clear defect uxos will flag — an icon-only button with no accessible name:

```html
<!doctype html>
<title>Build settings</title>
<main>
  <h1>CI cost controls</h1>
  <p>Monthly build spend: $412</p>
  <button><svg width="16" height="16" aria-hidden="true"></svg></button>
</main>
```

- [ ] **Step 2: Intent-spec fixture.** Create a minimal `tests/fixtures/intent-spec/.frontrails/intent/` modeling the **cost-conscious developer** as the actor — a real actor (`Developer`), a product job / journey about controlling build cost, a state, and a requirement — so `search_intent_harness` returns genuine ids. Scaffold it with the tooling rather than hand-writing schema:

```bash
cd tests/fixtures/intent-spec
INTENTOS_SPEC_ROOT=. INTENTOS_WORKSPACE_ROOT=. intent init   # or the pack's documented scaffold command
```

Then confirm it holds real entities: `INTENTOS_SPEC_ROOT=. intent status` (or drive `intentos.search_intent_harness` via the `praxec` MCP tool) returns ≥1 actor/journey/state/requirement id. Record one real entity id in the E2E doc as the expected `grounded_in` target. If `intent init` is unavailable, copy the smallest known-good spec under `frontrails-product/.frontrails/intent/` that yields a searchable entity, trimmed to a few entities, and rename entities to the developer/cost theme. Do NOT create a shopper/checkout spec.

- [ ] **Step 3: Validating consumer.** Create `examples/ux_vet_fix_min.yaml` — clearly commented "FOUNDATION E2E PROOF, not sub-project C". It includes `frontrails.yaml` + `frontrails-coordination.yaml`, and defines a tiny workflow: `assessing` (agent: `uxos.audit` on the fixture capture → produce one finding into `context.audit_findings`) → a `kind: workflow` transition into `flow.intent.propose_and_park` with `use: { inputs: { source_findings: "$.context.audit_findings", rationale: "uxos flagged an operable-name defect" }, outputs: { parked_tickets: "$.context.parked_tickets" } }` → terminal.

```yaml
include:
  - frontrails.yaml
  - frontrails-coordination.yaml
# workflow: ux_vet_fix_min — audit → propose_and_park (parks a real ticket)
```

- [ ] **Step 4: E2E procedure doc.** Create `tests/foundation-propose-park-e2e.md` mirroring `tests/campaign-tier2-e2e.md`: copy `tests/fixtures/intent-spec/` to a temp dir + `git init`; point the `intentos` connection's `INTENTOS_SPEC_ROOT`/`INTENTOS_WORKSPACE_ROOT` at that temp dir via an env overlay (per Tier-3's finding that the connection does NOT inherit `$.run.repo_root`); drive `examples/ux_vet_fix_min.yaml` through the `praxec` MCP tool following HATEOAS links. Record the three **atomic** assertions:
  1. `test -f "$TMP"/.frontrails/**/approvals/<ticket_id>.yaml` — the parked ticket file exists.
  2. `intentos.propose_status { ticket_id }` → `.status == "pending"`.
  3. the parked proposal's `grounded_in` contains the real entity id recorded in Step 2.
  Plus the poka-yoke: `git -C /home/mc/working/frontrails-product diff --stat crates/` is empty.

- [ ] **Step 5: Validate the example resolves.** Run:

```bash
export FLOWGATE=/home/mc/.cargo/bin/praxec
"$FLOWGATE" check --config examples/ux_vet_fix_min.yaml
```

Expected: exit 0, `validation: ok`; `ux_vet_fix_min` and `flow.intent.propose_and_park` both resolve.

- [ ] **Step 6: Commit.**

```bash
git add tests/fixtures/ux-capture tests/fixtures/intent-spec examples/ux_vet_fix_min.yaml tests/foundation-propose-park-e2e.md
git commit -m "test(coordination): uxos→propose E2E fixtures + validating consumer + procedure"
```

---

### Task 5: Run the E2E, assert the parked ticket, document

Execute the guided E2E and prove the seam with the on-disk artifact; then wire the README.

**Files:**
- Modify: `README.md` — add a "Coordination foundation" subsection.

**Interfaces:**
- Consumes: everything from Tasks 1, 3, 4.

- [ ] **Step 1: Prepare the temp fixture.** Per `tests/foundation-propose-park-e2e.md`, copy `tests/fixtures/intent-spec/` to a temp dir and `git init` it. Export the env overlay pointing `intentos` at the temp dir.

- [ ] **Step 2: Drive the seam.** Through the `praxec` MCP tool: `start` `ux_vet_fix_min`; follow links — `uxos.audit` on the capture fixture → hand the finding into `flow.intent.propose_and_park` → `search_intent_harness` (ground) → `propose`. Capture the `ticket_id` from the `pending_approval` response.

- [ ] **Step 3: Assertion 1 — ticket artifact exists.** Run: `find "$TMP" -path '*/approvals/*.yaml'` → Expected: one file whose stem is the captured `ticket_id`.

- [ ] **Step 4: Assertion 2 — status pending.** Drive `intentos.propose_status { ticket_id }` → Expected: `.status == "pending"`.

- [ ] **Step 5: Assertion 3 — grounded in a real id.** Inspect the parked ticket / the propose call's `grounded_in` → Expected: it contains the real entity id from Task 4 Step 2 (not fabricated).

- [ ] **Step 6: Poka-yoke.** Run: `git -C /home/mc/working/frontrails-product diff --stat crates/` → Expected: empty.

- [ ] **Step 7: Document.** Add a "Coordination foundation" subsection to `README.md`: what `flow.intent.propose_and_park` is (the reusable grounded-propose+park tail B & C call via `kind: workflow` + `use:`), the exposed `intentos.simulate_and_project`, the hand-off approval boundary (praxec parks, a human approves in IntentOS-desktop), and a pointer to the E2E doc + design spec. Note the `include:`-unprefixed id convention and that `check`/`fuzz` don't validate `kind: workflow` resolution.

- [ ] **Step 8: Commit.**

```bash
git add README.md
git commit -m "docs(coordination): README foundation subsection + E2E acceptance recorded"
```

---

## Self-Review

**Spec coverage:**
- §4.1 expose `simulate_and_project` → Task 1. ✓
- §4.2 `flow.intent.propose_and_park` (contract + agent states + output mechanism) → Task 2 (mechanism) + Task 3 (flow). ✓
- §4.3 E2E (both fixtures, validating consumer, drive & 3 assertions, poka-yoke) → Task 4 (build) + Task 5 (run/assert). ✓
- §5 non-goals — no task builds B/C, no approval bridge, no FrontRails change (enforced by the zero-diff poka-yoke in every task). ✓
- §7 acceptance (check resolves, fuzz clean, real parked ticket, zero diff) → Tasks 1/3/5 steps. ✓

**Placeholder scan:** Task 2 is a spike but its procedure is concrete (exact probe YAML + commands + decision rule) — it resolves the one genuinely engine-dependent unknown rather than hiding it. Task 3's state goals are given as verbatim intent (agent-driven flow — the "code" is the goal text + transition bindings, which is what praxec authoring is). Task 4 Step 2's spec scaffold names a fallback (copy a known-good minimal spec) so it can't dead-end.

**Type consistency:** `parked_tickets`/`applied`/`grounded_refs` and `source_findings`/`rationale` names are identical across the spec §4.2, Task 3 interfaces, and Task 4's `use:` binding. The `intentos.propose_status` field is `status` (value `"pending"`) consistently in the Global Constraints and Tasks 4–5. `ticket_id` capture path is resolved in Task 2 and consumed in Tasks 3–5.
