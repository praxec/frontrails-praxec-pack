# frontrails-flowgate-pack

A [Flowgate](https://github.com/) **pattern pack** that exposes the FrontRails
MCP servers — **IntentOS**, **StructureOS**, and **SecurityOS** — as Flowgate
capabilities.

The pack is pure Flowgate **configuration**. FrontRails itself is **never
modified**: its MCP servers are spawned as Flowgate `connections`, and each of
their `action`s is surfaced as an individually exposed capability. Flowgate
*reads* FrontRails' facts (`_spec_health`, `_required`, `_available`) and
follows them — it never *re-decides* FrontRails' gates. **FrontRails' gates
stay authoritative.**

## What's in the box

Each FrontRails server exposes ONE MCP tool (`intentos` / `structureos` /
`securityos`) that takes an `action` discriminator + `params`. So each action
becomes its own Flowgate capability: it calls the one tool with a fixed
`action` and templates the caller's `params` in via `$.arguments.params`.

- **Individual capabilities** (one per action), each exposed via `proxy.expose`
  with tags + aliases for discovery:
  - IntentOS: `intentos.status`, `intentos.search_intent_harness`,
    `intentos.propose`, `intentos.navigate`, `intentos.finish`
  - StructureOS: `structureos.scan_repo`, `structureos.get_diagnostics`,
    `structureos.move`
  - SecurityOS: `securityos.scan`
- **`frontrails_spec`** — a thin, hint-driven sub-workflow. Its agent-actor
  transitions *are* the exposed IntentOS capabilities; the running LLM picks
  each one and follows IntentOS' own HATEOAS hints. No journey logic is
  re-encoded — FrontRails' gates remain the source of truth.
- **`examples/autonomous_spec.yaml`** — an optional `kind: llm` driver that
  autonomously resolves IntentOS diagnostics toward a caller-supplied goal.

## Consuming the pack

### Verified remote include (recommended)

Pin a specific commit SHA (or signed tag) **and** the content hash:

```yaml
include:
  - uri: "https://raw.githubusercontent.com/<org>/frontrails-flowgate-pack/<sha>/frontrails.yaml"
    hash: "sha256:<64-hex>"
```

Non-`file://` includes **require** the `sha256:` hash — Flowgate rejects the
merged config with `INCLUDE_HASH_MISMATCH` if the fetched bytes don't match.

Compute the hash:

```bash
sha256sum frontrails.yaml
# prefix the hex digest with "sha256:" in the include entry
```

### Vendored file-path fallback

Vendor `frontrails.yaml` into your own repo and include it by relative path:

```yaml
include:
  - vendor/frontrails-flowgate-pack/frontrails.yaml   # hash optional for file:// paths
```

Includes deep-merge (maps merge, arrays concatenate, scalars: later wins), so
your own `connections` / `capabilities` / `proxy.expose` / `workflows` layer
cleanly on top.

## Versioning

- **Pin a SHA or signed tag + the `sha256:` hash.** Floating refs (`main`)
  defeat the hash guard and let upstream changes land silently.
- The pack's surface is the set of FrontRails **actions**. An exposure that
  names a removed action **fails fast at `mcp-flowgate check`** (capability
  reference resolution) — so a breaking FrontRails change surfaces as a config
  validation error in *your* CI, not a runtime surprise.
- Bump and re-pin when FrontRails action names change. Re-run `sha256sum` and
  update the `hash:` whenever `frontrails.yaml` changes.

## Validate

```bash
FLOWGATE=/path/to/mcp-flowgate ./scripts/check.sh        # exit 0 = valid
FLOWGATE=/path/to/mcp-flowgate "$FLOWGATE" check --config examples/autonomous_spec.yaml
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
