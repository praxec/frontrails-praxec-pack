# Manual / CI-gated E2E

This E2E proves the pack drives FrontRails end-to-end **without modifying
FrontRails**. It is manual / CI-gated because it needs live binaries and a
model key — it is not part of `mcp-praxec check` (which is structure-only).

## Requirements

- `intent` (IntentOS) on `PATH`.
- `structureos-mcp` on `PATH` (for the StructureOS capabilities; optional for
  the intentos-only drive).
- A model API key for the `kind: llm` executor in
  `examples/autonomous_spec.yaml` (e.g. `ANTHROPIC_API_KEY`), and the
  `mcp-praxec serve` runtime.
- The fixture `tests/fixtures/spec-with-diagnostic/` populated with a real
  spec that yields exactly one diagnostic (see that directory's README — TODO).

## Procedure

1. Validate the config (must pass before anything else):

   ```bash
   FLOWGATE=/path/to/mcp-praxec
   "$FLOWGATE" check --config examples/autonomous_spec.yaml   # exit 0
   ```

2. Serve the gateway against the populated fixture as the workspace root, so
   IntentOS' `INTENTOS_SPEC_ROOT`/`INTENTOS_WORKSPACE_ROOT` resolve to the spec:

   ```bash
   cd tests/fixtures/spec-with-diagnostic
   "$FLOWGATE" serve --config ../../../examples/autonomous_spec.yaml
   ```

3. Start the `drive_intentos` workflow with the goal, e.g.:

   ```json
   { "goal": "Resolve all IntentOS diagnostics to a clean spec." }
   ```

   The `kind: llm` executor calls ONLY the exposed `intentos.*` capabilities,
   follows IntentOS' own `_required` / `_available` hints, and continues until
   `_spec_health.errors == 0`, then calls `intentos.finish`.

## Acceptance (BOTH must hold)

1. **Spec healthy:** the journey reaches `_spec_health.errors == 0`
   (the single fixture diagnostic is resolved by IntentOS via `propose`).

2. **Zero-FrontRails-diff poka-yoke:** FrontRails source is untouched:

   ```bash
   # Set FRONTRAILS_PRODUCT_DIR to your frontrails-product checkout root.
   git -C "${FRONTRAILS_PRODUCT_DIR:?set FRONTRAILS_PRODUCT_DIR}" diff --stat crates/
   ```

   This MUST be EMPTY. The whole point of the pack is that FrontRails is
   *leveraged* (its MCP servers are spawned as Praxec connections), never
   *modified*. Any diff under `crates/` fails the E2E.

> The pack reads FrontRails facts (`_spec_health`, `_required`, `_available`)
> and follows them — it never re-decides FrontRails' gates. IntentOS' gates
> stay authoritative.
