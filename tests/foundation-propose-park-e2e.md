# Foundation propose→park E2E (uxos → propose_and_park)

Requires `praxec`, `intent` (IntentOS CLI/MCP), and `uxos-mcp` on `PATH`. Proves
the coordination-foundation seam: a uxos audit finding on a fixture capture
flows through `flow.intent.propose_and_park` (`frontrails-coordination.yaml`,
Task 3) and lands as a real, grounded, parked IntentOS approval ticket — never
applied, never self-approved (the hand-off boundary documented at the top of
that file). Every mutating/driving run happens against an ISOLATED throwaway
copy of `tests/fixtures/intent-spec/`, never against the pack repo or
`frontrails-product` itself.

## Requirements

- `praxec` (this procedure was authored/checked against `praxec 0.0.24`).
- `intent`/IntentOS on `PATH` (spawned by the `intentos` connection as
  `intent mcp serve`).
- `uxos-mcp` on `PATH` (spawned by the `uxos` connection).
- `jq` (used by assertion 2 below).
- The two fixtures from Task 4: `tests/fixtures/ux-capture/index.html` and
  `tests/fixtures/intent-spec/intentos.yaml`.

## Fixture shape note: `intentos.yaml`, not `.frontrails/intent/…`

The plan anticipated a `.frontrails/intent/…` on-disk shape for the fixture.
Live-testing the installed `intent` binary during Task 4 showed that is not
how it actually lays out a spec: `intent init` / a fresh `propose` write a
single-file spec (`intentos.yaml`) at the spec root, and `.frontrails/intent/`
under a repo is where the tool writes its OWN regenerable cache/diagnostics
(`llm/`, `vm/`, `milestones.json`, `.write.lock`, `.gitignore`) — never
committed source. This pack's own root `.gitignore` already ignores any
`.frontrails/` path at any depth, which would have silently swallowed a
`git add` of that shape anyway. So the committed fixture is the single file
`tests/fixtures/intent-spec/intentos.yaml`; `.frontrails/intent/` regenerates
on demand under a copy and does not need to be committed or copied.

## Real entities in the fixture (recorded here — do not fabricate)

Confirmed live via a raw JSON-RPC probe against `intent mcp serve`
(`search_intent_harness`, filter `{}`) during Task 4. All three are
developer/cost-themed; there is no shopper/checkout/cart entity anywhere in
this pack:

- `actor.developer` (actor, "Developer" — cost-conscious developer managing
  CI/build spend)
- `job.control-build-cost` (product_job, "Control Build Cost" — controls
  CI/build spend)
- `req.operable-controls` (requirement, MUST — "Every interactive control in
  the CI cost-controls view has an accessible name.")

**Expected `grounded_in` target for this E2E: `req.operable-controls`.** It is
the direct spec-side counterpart of the uxos finding. Confirmed live (Task 4):
`uxos.extract` on the fixture capture, then `uxos.audit` on the resulting IR,
reports a `semantic-a11y` gate blocker —
`gates.dimensions[semantic-a11y].blockers: ["control 'control_button_unnamed'
has no accessibleName — add aria-label / <label for> / text; name-dependent
a11y rules cannot evaluate it"]` — the exact defect `req.operable-controls`
exists to cover.

## Known gate: HMAC attestation on a fresh/edited spec

A spec file's first `propose` after any edit made outside the tool chain
(including authoring this fixture) fails closed with `ATTEST-001`
(`"invalid or missing HMAC attestation... all mutations blocked until
re-attestation via HITL"`), pointing at
`intentos(action: "request_full_review")`. Live-tested during Task 4:
`request_full_review` against an EMPTY spec (no entities yet) resolved
**instantly** (`attestation_status: "cleared"`, `hmac_regeneration.files_signed:
1`) — no browser/human needed when there is nothing yet to review, and it
rewrites the file with a fresh `_file_hmac` matching current content. The
committed `tests/fixtures/intent-spec/intentos.yaml` was left in exactly that
cleared/attested state, so a `propose` against an UNMODIFIED copy of it should
reach the async-HITL PARK path directly, without hitting `ATTEST-001` first.

However, the very next `propose` tried during Task 4 (adding the fixture's
first `product_job`, which crosses the spec's gate boundary
`baseline → market_strategy → product_strategy`) opened a `review_strategy`
HITL session that did **not** return within 20s in this headless environment
— the same synchronous-HITL gap already tracked in project memory
(async-HITL Phase 3 / SP-B: `review_strategy`/`request_full_review` are not
yet ticketized like `propose` is).

**Implication for Task 5:** drive `propose` with a change that does **not**
cross a gate boundary against this fixture — e.g. add a new `acceptance`
criterion `verifies: ["req.operable-controls"]`, or another `requirement`,
grounded in the existing `req.operable-controls` / `actor.developer` /
`job.control-build-cost` (gate stays at `product_strategy`, already cleared).
Do **not** add a new `product_job` or `actor` here, or the run may wedge on
`review_strategy` exactly like the gap above.

## Procedure

1. Copy the fixture to an isolated throwaway repo:

   ```bash
   TMP=$(mktemp -d)
   cp -r tests/fixtures/intent-spec/. "$TMP/"
   git -C "$TMP" init -q
   git -C "$TMP" -c user.email=e2e@local -c user.name=e2e add -A
   git -C "$TMP" -c user.email=e2e@local -c user.name=e2e commit -qm init
   ```

2. Point the `intentos` connection at `$TMP`, not the pack repo's cwd. Per
   Tier-3's finding, the `intentos` connection does NOT inherit
   `$.run.repo_root` — `frontrails.yaml`'s `connections.intentos.env` is a
   static `{INTENTOS_SPEC_ROOT: ".", INTENTOS_WORKSPACE_ROOT: "."}` resolved
   against wherever `praxec` itself is launched from. Two ways to make it
   resolve to `$TMP` (either works; the second is the explicit env overlay
   this task's brief asked for):

   - **(cwd override)** `cd "$TMP"` before starting `praxec serve` / driving
     via the `praxec` MCP tool, so `.` really is `$TMP`.
   - **(explicit env overlay)** a throwaway driver config that re-declares the
     `intentos` connection with absolute paths (a later `connections.intentos`
     block deep-merges over `frontrails.yaml`'s — same last-wins rule used
     everywhere else in this pack):

     ```yaml
     # e2e-gateway.yaml (throwaway; NOT part of this repo)
     include:
       - /abs/path/to/frontrails-praxec-pack/examples/ux_vet_fix_min.yaml
     connections:
       intentos:
         kind: mcp
         command: "intent"
         args: ["mcp", "serve"]
         env:
           INTENTOS_SPEC_ROOT: "$TMP"
           INTENTOS_WORKSPACE_ROOT: "$TMP"
     store:
       kind: sqlite
       path: /abs/path/to/throwaway/e2e-store.db
     repos:
       - path: "$TMP"
         writable: true
     ```

3. A durable `store:` + a writable `repos:` entry for `$TMP` are required
   regardless of which option is used in step 2 — the `advance` transition's
   `kind: workflow` dispatch into `flow.intent.propose_and_park` needs both to
   survive across separate `praxec command` invocations (Task 2's note;
   `examples/ux_vet_fix_min.yaml` itself has neither, which is fine for
   `check` but not for a live multi-hop `command` drive).

4. Validate the (overlaid) config first: `praxec check --config <overlay>.yaml`
   → exit 0.

5. Drive via the `praxec` MCP tool, following HATEOAS links. Every transition
   here is `actor: agent`, so each hop needs an explicit submit — nothing
   auto-chains:

   - `praxec.command { definitionId: "ux_vet_fix_min" }` → start.
   - `extract` — `arguments: { html: <contents of
     tests/fixtures/ux-capture/index.html> }`.
   - `audit` — `arguments: { ir: <extract's returned ir> }` — observe the
     `semantic-a11y` gate blocker on `control_button_unnamed`.
   - `record` — `arguments: { audit_findings: [{ target_hint: "operable
     control has no accessible name", detail: "control_button_unnamed has no
     accessibleName" }] }` (the shape is opaque to `propose_and_park`;
     `target_hint` is what its `grounding.search` transition feeds to
     `search_intent_harness`).
   - `advance` — dispatches the `kind: workflow` child
     (`flow.intent.propose_and_park`); expect the response to park with
     `result.status: "waiting"` naming the child workflow id (mirrors
     Tier-2's `refactoring`/`decompose` dispatch pattern) — this hop does not
     resolve the child itself.
   - Against the CHILD workflow id: `search` —
     `intentos.search_intent_harness { params: { filter: { entity_type:
     ["requirement"] } } }` → confirms `req.operable-controls` is real;
     `record` (persist `grounded_refs`); `advance` → `proposing`.
   - `propose` — e.g.
     ```json
     {
       "changes": [
         { "op": "add", "entity_type": "acceptance",
           "entity_id": "ac.build-cost-controls-operable",
           "fields": { "statement": "The icon-only button gains an accessible name (aria-label or visible text).",
                        "verifies": ["req.operable-controls"] } }
       ],
       "grounded_in": ["req.operable-controls"],
       "rationale": "uxos flagged an operable-name defect"
     }
     ```
     (an `acceptance` linked to the existing `req.operable-controls` does not
     cross a gate boundary — see the attestation note above). Capture
     `ticket_id` from the `pending_approval` envelope (`kind: mcp` raw text at
     `$.output.content.0.text` → jq `.ticket_id`, per Task 2's note).
   - `record_parked`; `advance` → `reporting`; `finish` → `done`. Back on the
     PARENT: `finish` → `done`; confirm `context.parked_tickets` contains the
     `ticket_id`.

## Acceptance (three atomic assertions)

1. **Parked ticket file exists:**

   ```bash
   test -n "$(find "$TMP" -path '*/approvals/*.yaml' -name "${TICKET_ID}.yaml")"
   ```

   Expected: exactly one match.

2. **Ticket still pending:**

   ```bash
   # PROPOSE_STATUS_JSON = the text envelope from
   # intentos.propose_status { params: { ticket_id: "$TICKET_ID" } }
   jq -e '.status == "pending"' <<<"$PROPOSE_STATUS_JSON"
   ```

   Expected: `true`.

3. **Grounded in the real entity id:**

   ```bash
   grep -rq 'req\.operable-controls' "$TMP"/.frontrails/*/approvals/"${TICKET_ID}.yaml" \
     "$TMP"/*/approvals/"${TICKET_ID}.yaml" 2>/dev/null
   ```

   Expected: match — the parked proposal's `grounded_in` contains
   `req.operable-controls`, the id recorded above (not fabricated).

Plus the poka-yoke every task in this plan repeats:

```bash
git -C /home/mc/working/frontrails-product diff --stat crates/
```

Expected: empty.

## Cleanup

```bash
rm -rf "$TMP"
```
