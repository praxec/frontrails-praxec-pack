# frontrails-praxec-pack

A Praxec **pattern pack** that exposes the FrontRails
MCP servers — **IntentOS**, **StructureOS**, **SecurityOS**, and **uxos** — as
Praxec capabilities.

The pack is pure Praxec **configuration**. FrontRails itself is **never
modified**: its MCP servers are spawned as Praxec `connections`, and each of
their `action`s is surfaced as an individually exposed capability. Praxec
*reads* FrontRails' facts (`_spec_health`, `_required`, `_available`) and
follows them — it never *re-decides* FrontRails' gates. **FrontRails' gates
stay authoritative.**

## What's in the box

All four `*os` servers each expose ONE MCP tool (`intentos` / `structureos` /
`securityos` / `uxos`) that takes an `action` discriminator + `params`, so each
action becomes its own Praxec capability (fixed `action` + templated
`$.arguments.params`).

Bindings use `map:` — **not** `arguments:`. The `mcp` executor reads only
`connection` + `tool` + `map`; an `arguments:` block is silently ignored, and
without a `map:` the caller's raw args pass straight through, so the pinned
`action` never reaches the tool. `map:` resolves recursively, which is what lets
a capability assemble a nested `params: { … }` from literals and `$.` paths.

- **Individual capabilities** (one per action), each exposed via `proxy.expose`
  with tags + aliases for discovery:
  - IntentOS: `intentos.status`, `intentos.search_intent_harness`,
    `intentos.propose`, `intentos.propose_status`, `intentos.navigate`,
    `intentos.worklist`, `intentos.certify`, `intentos.finish`
  - StructureOS: `structureos.scan_repo`, `structureos.get_diagnostics`,
    `structureos.move`
  - SecurityOS: `securityos.scan`
  - uxos: `uxos.extract`, `uxos.conform`, `uxos.scan_dark_patterns`,
    `uxos.audit`, `uxos.check_coverage`, `uxos.compare` — the `uxos` tool's
    actions, each templating the inline `ir` document (or raw `html` /
    `a11y_tree` for extract). Every uxos verb is **read-only** — it evaluates a
    UX model, never mutates a spec — so none is gated. These are CONFORMANCE
    verbs, not intentos clones: intentos authors *what* a step needs; uxos
    computes *how understandable/operable* the realization is (verdicts carry a
    deterministic-vs-advisory epistemic class).
- **`frontrails_ux_conformance`** — assess a real surface with uxos: from a raw
  capture (`extract`) to the one-shot `audit` (conformance + the modality-neutral
  completeness ladder + structural dark patterns), drilling into `dark_patterns`
  / `conform` / `ux_coverage` as needed. Read-only: it computes and reports,
  never mutates.
- **`frontrails_spec`** — a thin, hint-driven sub-workflow. Its agent-actor
  transitions *are* the exposed IntentOS capabilities; the running LLM picks
  each one and follows IntentOS' own HATEOAS hints. No journey logic is
  re-encoded — FrontRails' gates remain the source of truth.
- **`frontrails_burndown`** — the worklist **burn-down loop**. After one user
  kickoff, the running LLM repeatedly reads the prioritized frontier
  (`intentos.worklist`), resolves the top-ranked item via its `suggested_action`
  (usually `intentos.propose`), and re-reads — stopping when the worklist is empty
  or `intentos.certify` reports `passed: true`. The worklist's ranking and
  IntentOS' write-protection/gates stay authoritative (attested intent is diverted
  to candidates, never overwritten); Praxec provides the loop + audit + the
  `maxChainDepth` runaway cap. This is the autonomous self-correction loop on top
  of the M4 worklist + M6 certify gate.
- **`examples/autonomous_spec.yaml`** — an optional `kind: llm` driver that
  autonomously resolves IntentOS diagnostics toward a caller-supplied goal.

Soundness of all three workflows is checked with `mcp-praxec fuzz`
(mock-executor scenarios for wedges/livelocks/engine errors) — see
`scripts/check.sh` and the acceptance notes.

### Async-HITL (FrontRails 0.0.17)

Mutating IntentOS actions are human-gated. When a mutation can't reach a human —
as in this headless/agent context — `intentos.propose` (and the
`review_strategy` / `request_full_review` review actions) no longer wedge or
fail closed: they **park the change as a durable approval ticket** and return
`status: "pending_approval"` + a `ticket_id`. The change is not applied until a
human approves it through IntentOS' own dialog channel — an agent structurally
cannot self-approve, so that gate stays authoritative here. Poll the ticket with
`intentos.propose_status { ticket_id }` (read-only: `pending` → `applied` /
`rejected` / `failed` / `expired`). The `frontrails_burndown` loop treats a
parked ticket as "handed off, keep going" — it records the id and burns down
other worklist items rather than blocking; `certify` will not report
`passed: true` until the parked change is approved and applied.

## Consuming the pack

### Verified remote include (recommended)

Pin a specific commit SHA (or signed tag) **and** the content hash:

```yaml
include:
  - uri: "https://raw.githubusercontent.com/<org>/frontrails-praxec-pack/<sha>/frontrails.yaml"
    hash: "sha256:<64-hex>"
```

Non-`file://` includes **require** the `sha256:` hash — Praxec rejects the
merged config with `INCLUDE_HASH_MISMATCH` if the fetched bytes don't match.

Compute the hash:

```bash
sha256sum frontrails.yaml | awk '{print "sha256:" $1}'
```

### Vendored file-path fallback

Vendor `frontrails.yaml` into your own repo and include it by relative path:

```yaml
include:
  - vendor/frontrails-praxec-pack/frontrails.yaml   # hash optional for file:// paths
```

Includes deep-merge (maps merge, arrays concatenate, scalars: later wins), so
your own `connections` / `capabilities` / `proxy.expose` / `workflows` layer
cleanly on top.

## Versioning

- **Pin a SHA or signed tag + the `sha256:` hash.** Floating refs (`main`)
  defeat the hash guard and let upstream changes land silently.
- The pack's surface is the set of FrontRails **actions**. An exposure that
  names a removed action **fails fast at `mcp-praxec check`** (capability
  reference resolution) — so a breaking FrontRails change surfaces as a config
  validation error in *your* CI, not a runtime surprise.
- Bump and re-pin when FrontRails action names change. Re-run `sha256sum` and
  update the `hash:` whenever `frontrails.yaml` changes.

## Validate

```bash
export FLOWGATE=/path/to/mcp-praxec          # the built binary
./scripts/check.sh                              # validates frontrails.yaml (exit 0 = valid)
"$FLOWGATE" check --config examples/autonomous_spec.yaml
```

`check` validates **structure only** — it does not spawn the MCP servers, so
missing server binaries are fine at validation time.

## Open items

- **SecurityOS binary name unconfirmed.** The `securityos` connection uses
  `command: "securityos"` with `args: ["mcp", "serve"]` — verify against the
  real binary before runtime use.
- **StructureOS / SecurityOS action names are best-effort.**
  `scan_repo` / `get_diagnostics` / `move` / `scan` are taken from the
  workflow docs; confirm against each server's live `action` discriminator.
- **E2E fixture not yet populated** — see `tests/fixtures/spec-with-diagnostic/`.

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

### Tier 2 (structural) additional wiring

`flow.findings.structural` references the god-file decomposition flow by the id
`flow.refactor.god-file`, so the consumer gateway must also load that flow and
have a **coding agent** wired — the god-file flow's `fixing` state is
`actor: agent` (Tier 2 is therefore not agent-free; Tier 1 is).

**Note on the referenced id.** The id you write depends on how the flow is
mounted:
- **`include:` (flat merge, this pack's convention)** — add the flow file to
  your `include:` list; it merges as a top-level workflow, so reference it
  **unprefixed** as `flow.refactor.god-file` (see `examples/campaign-check.yaml`).
- **`repos:` mount (namespaced)** — if you instead mount
  `cognitive-architectures-max` as a repo, reference it **prefixed** as
  `cognitive-max/flow.refactor.god-file`.

The shipped flow uses the unprefixed form to match this pack's `include:`-based
config. **Caveat:** `praxec check`/`fuzz` do NOT validate `kind: workflow`
`definitionId` resolution, so a wrong id fails only at dispatch time — keep the
id form consistent with your mount mode.

### Tier 3 (advisory report) — read-only, no extra wiring

`flow.findings.triage-report` is **read-only**: it scans the target repo via the
`structureos` connection (`scan_repo`), buckets every finding code into its tier
per `taxonomy.finding-tiers.yaml`, and emits a per-tier report under
`.praxec/reports/triage-*.{json,md}`. It NEVER mutates code, dismisses a finding,
or commits a branch — reporting IS the deliverable (spec §4: Tier 3 is never
required to reach zero, deliberately avoiding the mass-dismissal trap). So Tier 3
needs **no coding agent and no cargo** (unlike Tier 2) — only `structureos-mcp` +
`jq`. It adds no new cross-references, so `examples/campaign-check.yaml` resolves
it with the existing include set (no edit needed).

- **Tier routing is data.** Unknown SOS codes default to Tier 3 and are listed in
  `unknown_codes` — the first-class drift signal (spec §9): a spike means a
  StructureOS version bump silently re-routed codes to the Tier-3 default.
  Non-finding sentinel keys (e.g. `SOS-RUSTC-OFF`) are filtered out.
- **Scan target.** praxec does not spawn the `structureos` connection with cwd =
  `$.run.repo_root`, so the shipped `STRUCTUREOS_WORKSPACE_ROOT: "."` resolves to
  the gateway's own cwd. Point the scan at your repo by running the gateway from
  the repo root, or pin `connections.structureos.env.STRUCTUREOS_WORKSPACE_ROOT`
  to an absolute path (see `tests/campaign-tier3-e2e.md`).
## Coordination foundation (source findings → IntentOS)

The thin enabling layer for driving a *source* (a uxos audit finding, or a
Preveti/Simuli strategy run) into IntentOS as a **grounded** proposal. Ships as
an additive `frontrails-coordination.yaml` that deep-merges alongside
`frontrails.yaml`:

- **`flow.intent.propose_and_park`** — a reusable, agent-driven mutation tail
  (`grounding → proposing → reporting`). Callers hand it `source_findings` +
  `rationale` via `kind: workflow` + `use:` (exactly like Tier 2 references the
  god-file flow) and read back `parked_tickets` / `applied` / `grounded_refs`.
  It grounds each finding against the live spec (`search_intent_harness` → real
  entity ids), proposes it grounded, and records the outcome — never
  self-approving.
- **`intentos.simulate_and_project`** — exposed on `frontrails.yaml` so
  sub-project B (Preveti → compass) can drive it directly.
- **`examples/ux_vet_fix_min.yaml`** — the validating consumer: `uxos.audit` on a
  fixture capture → `flow.intent.propose_and_park`.

**Propose has three outcomes, not one** (verified live — see
`tests/foundation-propose-park-e2e.md`): `approved` (applies directly),
`pending_approval` (parks a ticket — the hand-off path), or `needs_review` (a
gate-crossing change opens a HITL review). A benign propose against a headless
intentos with no HITL mechanism **auto-applies**; the "agent cannot self-approve"
hand-off bites only where a HITL mechanism is active (the gateway + desktop) or
the change crosses a gate. The flow records `applied` and `parked_tickets` alike.

The flow reference is unprefixed (`include:` convention); `check`/`fuzz` do not
validate `kind: workflow` resolution — a live `praxec command` drive does. See
`docs/superpowers/specs/2026-07-17-praxec-coordination-foundation-design.md`.

## Sub-projects B & C (built on the foundation)

Two thin consumers of the coordination foundation — additive files that
deep-merge alongside `frontrails.yaml`. Design:
`docs/superpowers/specs/2026-07-17-operationalize-b-c-design.md`.

- **B — `flow.strategy.populate-compass`** (`frontrails-strategy.yaml`): drives
  a Preveti/Simuli run via the exposed `intentos.simulate_and_project` and reports
  what it projected into the IntentOS OpportunityCompass. **Requires a live
  Preveti** — set `PREVETI_BASE_URL` + `PREVETI_API_KEY` on the `intentos`
  connection. For local dev, `sim_host` (in `~/working/simuli`) in-memory with
  `SEED_DEMO=1` serves the REST API on `:8080` and seeds the key
  `demo.preveti-demo-key` (see `tests/preveti-compass-e2e.md`). The
  `compass_write` gate and the Simuli projection stay authoritative — B only
  drives + reports.
- **C — `flow.ux.vet-and-fix`** (`frontrails-ux.yaml`): audits a UX capture with
  `uxos` and hands the budget-capped findings to `flow.intent.propose_and_park`
  in one dispatch (the tail iterates the list). Records whichever outcome each
  proposal reaches — `applied` or `parked_tickets` — honoring the three-outcomes
  reality; never self-approves. E2E: `tests/ux-vetfix-e2e.md`.

C dispatches the foundation tail, so its consumer includes
`frontrails-coordination.yaml` too; the tail now declares its `outputs:`
(`parked_tickets`/`applied`/`grounded_refs`) so an included consumer's
`use.outputs` projection types under SPEC §7.2. Reference ids unprefixed
(`include:` convention); `check`/`fuzz` don't validate `kind: workflow`
resolution — a live drive does.

## Tests

See `tests/README.md` for the manual / CI-gated E2E. Its acceptance includes
the **zero-FrontRails-diff poka-yoke**: after a successful drive,
`git -C <frontrails-product> diff --stat crates/` must be EMPTY.
