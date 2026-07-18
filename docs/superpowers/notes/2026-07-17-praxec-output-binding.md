# Praxec sub-workflow output binding — resolved (spike, Task 2)

Resolved empirically with a throwaway probe config run against the installed
`praxec` binary (`/home/mc/.cargo/bin/praxec`, v0.0.24), cross-checked against
the binary's actual source (`~/working/mcp-flowgate/crates/praxec-core` /
`praxec-executors` — the repo that binary is built from). Task 3: copy the
patterns below verbatim.

## 1. `use.outputs` DOES project a callee value to the caller — confirmed live

No fallback needed. The working syntax (verified against
`praxec-executors/src/workflow.rs` + `praxec-core/src/use_binding.rs`, both
unit-tested, and against a live two-hop `praxec command` run):

```yaml
executor:
  kind: workflow
  definitionId: <callee_definition_id>
  use:
    inputs:
      <callee_input_name>: "$.context.<host_field>"
    outputs:
      # LHS = HOST context path, MUST match ^\$\.context\.[a-z][a-z0-9_-]*$
      #       (V12 load-time check; only a top-level lowercase context key).
      # RHS = plain field name (NO "$." prefix) read off the CALLEE's
      #       terminal/final context — i.e. a top-level key the callee wrote
      #       via its own `output:` mapping before reaching a terminal state.
      "$.context.<host_field>": <callee_context_field_name>
```

Mechanics (from source, `expand_use_bindings`/`expand_one_transition` in
`praxec-core/src/config.rs`): at config-load time, praxec auto-**synthesizes**
a transition-level `output:` mapping from `use.outputs` —
`<host_field>: "$.output.<callee_context_field_name>"` — and merges it with
any operator-declared `output:` on that same transition (operator entries win
on key collisions). At runtime (`WorkflowExecutor::execute` in
`praxec-executors/src/workflow.rs`), when the child terminates `succeeded`,
`project_use_outputs` reads each declared callee field off the child's final
context and `rekey_by_cap_output_name` republishes it so the synthesized
`$.output.<name>` pointer resolves. **No manual `output:` block is required
on the caller's transition** — declaring `use.outputs` is sufficient; it
implies the mapping.

### Live proof (probe-outputs.yaml, throwaway, NOT committed)

Caller `probe.caller` (`use: { outputs: { "$.context.captured": "emitted" } }`)
called callee `probe.callee`, whose single `kind: script` transition wrote
`emitted` into its own context. Driving both hops via
`praxec command --config probe-outputs.yaml '{...}'` (sqlite store required —
`allow_ephemeral` alone does not persist a workflow across separate CLI
invocations; also needed `repos: [{ path: ".", writable: true,
definitions: false }]` or `command` fails fast with
`REPO_ROOT_REQUIRED`) produced this audit event on the PARENT the instant the
child terminated:

```
event_type: workflow.transition
payload.blackboardDelta: {"_subworkflow_wait": null, "captured": "tkt-probe-1"}
```

Final query on the caller (`praxec query --config probe-outputs.yaml
'{"workflowId":"..."}'`):

```json
{ "context": { "captured": "tkt-probe-1", "seed": "x" },
  "result": { "status": "succeeded" },
  "workflow": { "state": "done", "version": 3 } }
```

`captured` == the callee's emitted value. The projection is real and worked
on the first correctly-shaped attempt (once the CLI's actual `command`/store
requirements — see §4 — were satisfied).

## 2. Where a transition's output lands in context

- **`kind: script`**: stdout is auto-parsed as JSON and exposed at
  `$.output.json.<field>`. Confirmed both in `frontrails-campaign.yaml`
  (`flow.findings.structural`'s `extracting` state:
  `god_file_count: "$.output.json.god_file_count"`) and live in the probe
  (`emitted: "$.output.json.ticket_id"` populated `emitted` from
  `printf '{"ticket_id":"tkt-probe-1"}'`).
- **`kind: mcp`**: the executor has no `outputSchema` fallback parsing — its
  `$.output` is the RAW MCP envelope, plain text content at
  `$.output.content.0.text` (a JSON *string*, not a parsed object). A plain
  JSON-pointer read cannot parse that string further; hand it to a following
  `kind: script` transition, which does the actual `jq` parse (mirrors
  `frontrails-campaign.yaml`'s `scanning` → `extracting` two-step: `scan_text:
  "$.output.content.0.text"` then a script that `jq`s it). This was already
  live-verified in a prior task (structureos) and is architecturally identical
  for any `kind: mcp` tool call, including `intentos.propose`.

## 3. `ticket_id` path in a `pending_approval` propose envelope

Read `crates/intentos-apps/src/mcp/tools/handlers/hitl_async.rs`,
`build_pending_response()` (used by `propose`'s async-HITL park path,
`propose_hitl.rs`):

```rust
json!({
    "status": "pending_approval",
    "ticket_id": ticket.ticket_id,
    "expires_at": ticket.expires_at.to_rfc3339(),
    "parked_reason": ticket.parked_reason,
    "message": "...",
    "_llm_instruction": "...",
})
```

`ticket_id` is a **flat, top-level field** of the envelope — no nesting under
`result`/`data`/etc.

Composed with §2's `kind: mcp` finding, the full path from a
`propose_and_park`-style transition is:

1. `kind: mcp` transition output mapping: `mcp_text: "$.output.content.0.text"`
   (raw text, still a JSON string).
2. A following `kind: script` transition parses it:
   `jq -r '.ticket_id' <<<"$1"` (jq path: **`.ticket_id`**, top-level) —
   analogous to `run.campaign.extract-godfile-summary`'s
   `jq -r '._summary.by_id.SOS001 // 0'` pattern — then re-emits
   `{"ticket_id": "..."}` on its own stdout so the workflow's `output:`
   mapping captures it via `$.output.json.ticket_id` (§2).

## 4. Incidental CLI/config findings (bound this spike, not shipped)

- The installed binary's real flags are `praxec command --config <CONFIG>
  <ARGS_JSON>` and `praxec query --config <CONFIG> <ARGS_JSON>` — **not**
  `--definition`/`--input` (the brief's placeholder names). `ARGS_JSON` is a
  positional JSON blob, e.g. `'{"definitionId":"probe.caller"}'` to start, or
  `'{"workflowId":"...","expectedVersion":N,"transition":"go"}'` to submit.
- A non-deterministic (`actor: agent`, the default when `actor:` is omitted)
  transition does NOT auto-chain at workflow start — each hop (including the
  callee's own transitions) needs an explicit `command …transition` submit.
  Only `actor: deterministic` transitions auto-chain (as seen throughout
  `frontrails-campaign.yaml`).
- A `kind: workflow` sub-workflow call requires a durable `store: { kind:
  sqlite, path: ... }` to survive across separate CLI invocations —
  `gateway.allow_ephemeral: true` alone is in-memory-per-process and the
  child cannot be resumed by a second `command` call.
- Any run also requires a writable `repos:` entry (`path`, `writable: true`);
  omitting it fails fast with `REPO_ROOT_REQUIRED` before any transition runs.
- Scripts require an explicit `lifecycle:` field (`experimental` used here) —
  `check` fails with `MISSING_SCRIPT_LIFECYCLE` otherwise.

These four are load-bearing for Task 3's real flow (the caller invoking
`propose_and_park` will itself be a multi-hop `kind: mcp` → `kind: script`
→ `kind: workflow` chain and needs the same store/repos/lifecycle shape to
actually run, not just `check` green).
