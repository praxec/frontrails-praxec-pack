# Fixture: spec-with-diagnostic

This directory should hold a **minimal IntentOS spec** that, when loaded by
`intent`, yields **exactly one known diagnostic** (one error). The E2E
(`tests/README.md`) drives the `drive_intentos` workflow against this fixture
and asserts the journey reaches `_spec_health.errors == 0`.

## Status: NOT YET POPULATED

The spec internals are intentionally left empty. Fabricating `.frontrails/intent/`
YAML by hand would (a) likely fail IntentOS validation and (b) violate the
spec-first workflow (specs are mutated only via the `intentos` MCP `propose`
action, never hand-edited).

## TODO — populate it by recording a real diagnostic

1. In a throwaway directory, run `intent` to scaffold/seed a minimal spec.
2. Drive it (via the `intentos` tool / `propose`) into a state that produces
   exactly ONE diagnostic — e.g. an orphaned state (SEM-011) or a missing
   acceptance criterion. Confirm with `intentos(action: "status")`.
3. Copy the resulting `.frontrails/intent/` tree into this directory.
4. Record the expected diagnostic CODE here so the E2E can assert on it:

   - Expected diagnostic code: `TODO`
   - Expected count before drive: `1`
   - Expected count after drive: `0`

Do NOT hand-author the spec YAML — capture it from a real `intent` run so the
diagnostic is genuine and reproducible.
