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

The three `*os` servers each expose ONE MCP tool (`intentos` / `structureos` /
`securityos`) that takes an `action` discriminator + `params`, so each action
becomes its own Praxec capability (fixed `action` + templated `$.arguments.params`).
uxos (`ux-ir-mcp`) is the exception: it exposes one tool per verb, so each uxos
capability names its tool directly and templates the inline `ir` document.

- **Individual capabilities** (one per action), each exposed via `proxy.expose`
  with tags + aliases for discovery:
  - IntentOS: `intentos.status`, `intentos.search_intent_harness`,
    `intentos.propose`, `intentos.propose_status`, `intentos.navigate`,
    `intentos.worklist`, `intentos.certify`, `intentos.finish`
  - StructureOS: `structureos.scan_repo`, `structureos.get_diagnostics`,
    `structureos.move`
  - SecurityOS: `securityos.scan`
  - uxos: `uxos.extract`, `uxos.conform`, `uxos.scan_dark_patterns`,
    `uxos.audit`, `uxos.check_coverage`, `uxos.compare`. Unlike the `*os`
    servers, `ux-ir-mcp` exposes one tool PER VERB (no `action` discriminator),
    so each capability names its tool directly and templates the inline `ir`
    document (or raw `html` / `a11y_tree` for extract). Every uxos verb is
    **read-only** — it evaluates a UX model, never mutates a spec — so none is
    gated. These are CONFORMANCE verbs, not intentos clones: intentos authors
    *what* a step needs; uxos computes *how understandable/operable* the
    realization is (verdicts carry a deterministic-vs-advisory epistemic class).
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

## Tests

See `tests/README.md` for the manual / CI-gated E2E. Its acceptance includes
the **zero-FrontRails-diff poka-yoke**: after a successful drive,
`git -C <frontrails-product> diff --stat crates/` must be EMPTY.
