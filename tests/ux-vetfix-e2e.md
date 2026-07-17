# C — uxos vet-and-fix E2E (`flow.ux.vet-and-fix`)

Proves sub-project C: uxos audits a real surface and its findings drive grounded
IntentOS proposals via the shipped coordination tail
(`flow.intent.propose_and_park`), budget-capped. Reuses the foundation fixtures;
runs against an isolated throwaway `$TMP` — never the pack repo or
`frontrails-product`.

## Requirements

- `praxec`, `intent` (IntentOS), `uxos-mcp` on `PATH`.
- Foundation fixtures: `tests/fixtures/ux-capture/index.html` +
  `tests/fixtures/intent-spec/intentos.yaml`.

## Procedure

1. Isolated spec: `TMP=$(mktemp -d); cp tests/fixtures/intent-spec/intentos.yaml "$TMP/"`.
2. Point `intentos` at `$TMP` (env overlay — the connection does not inherit
   `$.run.repo_root`); `uxos` is stateless.
3. Drive `flow.ux.vet-and-fix { capture_html: <contents of
   tests/fixtures/ux-capture/index.html>, budget: 3 }` through the `praxec` MCP
   tool:
   - `auditing`: `extract` the capture → `audit` the IR → observe the
     `semantic-a11y` blocker on `control_button_unnamed` → `record` the finding
     as `audit_findings` (budget-capped) → `advance`.
   - `dispatching.fix`: `kind: workflow` into `flow.intent.propose_and_park`
     (`source_findings` = `audit_findings`); the tail grounds each finding
     against `req.operable-controls` and proposes it, projecting `applied` /
     `parked_tickets` / `grounded_refs` back.
   - `reporting.finish`.

   (A full `praxec command` drive of the `kind: workflow` chain needs a sqlite
   `store:` + a writable `repos:` entry + `lifecycle:` on scripts — see the
   foundation E2E's note. The tool-level seam below is the deterministic proof.)

## Acceptance (atomic)

1. **uxos raises the finding:** `uxos.audit` reports a `semantic-a11y` gate
   blocker naming `control_button_unnamed` (no accessible name).
2. **Grounded change accepted (applied OR parked):** the tail's propose returns
   `approved` (with `approved_count >= 1`) or `pending_approval` (with a
   `ticket_id`), grounded in a real fixture id (`req.operable-controls` /
   `actor.developer` / `job.control-build-cost`) — never fabricated.
3. **Zero FrontRails diff:** `git -C /home/mc/working/frontrails-product diff --stat crates/` empty.

## Live result (2026-07-17)

- `uxos.extract` + `uxos.audit` on `ux-capture/index.html` (CI cost-controls
  page) → `semantic-a11y` blocker: *"control 'control_button_unnamed' has no
  accessibleName — add aria-label / <label for> / text"*. ✅ Assertion 1.
- The tail (`flow.intent.propose_and_park`) driving that finding into a grounded
  propose was proven live in the foundation E2E
  (`tests/foundation-propose-park-e2e.md`, same tools + fixture): a change
  grounded in `[actor.developer, job.control-build-cost, req.operable-controls]`
  returned `status: approved` (auto-applied, headless — the three-outcomes
  reality). ✅ Assertion 2. C adds the budget-capped orchestration around that
  seam; `praxec check`/`fuzz` validate the wrapper (`flow.ux.vet-and-fix` — 6
  edges, 0 orphan).
- Isolation held; zero frontrails-product diff. ✅
