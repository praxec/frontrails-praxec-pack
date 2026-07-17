# B — Preveti → compass E2E (`flow.strategy.populate-compass`)

Proves sub-project B against a **live Preveti**, not a mock (owner decision):
`intentos.simulate_and_project` runs a real Preveti/Simuli simulation and
persists the projection into the IntentOS OpportunityCompass. Runs against an
isolated throwaway `$TMP` spec — never the pack repo or `frontrails-product`.

## Requirements

- `praxec`, `intent` (IntentOS), and a **running Preveti REST API**.
- The Preveti/Simuli sibling repo (`~/working/simuli`) built (`cargo build -p sim_host`).
- `jq` for the assertions.

## Stand up a local Preveti (in-memory, demo seed — no Postgres needed)

`sim_host` serves gRPC (:50051) **and** the REST API (:8080) from one port, and
`SEED_DEMO=1` seeds a known full-tier API key (`demo.preveti-demo-key`).
Deterministic runs complete without LLM provider keys.

```bash
cd ~/working/simuli
# Force the in-memory store (its .env sets DATABASE_URL -> Postgres; an empty
# DATABASE_URL wins because dotenvy never overrides an already-set var, and the
# empty value takes the in-memory branch). Launch via a ( … & ) subshell — a
# bare `&`/pkill -f on the binary path can self-signal the launching shell.
( DATABASE_URL='' SEED_DEMO=1 REST_PORT=8080 GRPC_PORT=50051 ./target/debug/sim_host > /tmp/simhost.log 2>&1 & )
# wait for it, then confirm the demo key mints a JWT:
curl -s --retry 20 --retry-delay 1 --retry-connrefused -X POST http://localhost:8080/v1/token \
  -H 'content-type: application/json' -d '{"api_key":"demo.preveti-demo-key"}' | jq -e .access_token
```

## Procedure

1. Isolated throwaway spec dir: `TMP=$(mktemp -d)`.
2. Launch `intent mcp serve` with the Preveti endpoint + key + spec root:

   ```bash
   INTENTOS_SPEC_ROOT=$TMP INTENTOS_WORKSPACE_ROOT=$TMP \
   PREVETI_BASE_URL=http://localhost:8080 PREVETI_API_KEY=demo.preveti-demo-key \
   intent mcp serve
   ```

   (Or drive `flow.strategy.populate-compass` through the `praxec` MCP tool with
   the `intentos` connection env overlaid to those three values — per Tier-3, the
   connection does NOT inherit `$.run.repo_root`.)
3. Drive `simulate` (`intentos.simulate_and_project { objective, audience_json }`).
   It runs Preveti (`start_run → await_run → project_run → persist`) — usually a
   few seconds deterministic, up to ~120s if research-grounded. Capture `run_id`
   / `status` / `summary`, then `record` + `advance` → `reporting`.
4. `reporting`: `status` confirms the compass/gate state; the projected counts are
   in the sim `summary`; `search` surfaces any projected `actor`. `record` +
   `finish`.

## Acceptance (atomic)

1. **Run completed:** the `simulate_and_project` response has a `run_id` and
   `status == "complete"`.
2. **Compass projected + persisted:** `$TMP/compass.yaml` exists and contains at
   least one projected frame with `source simuli` and the returned `run_id`:

   ```bash
   grep -q "source simuli" "$TMP/compass.yaml" && grep -q "$RUN_ID" "$TMP/compass.yaml"
   ```

3. **Zero FrontRails diff:** `git -C /home/mc/working/frontrails-product diff --stat crates/` empty.

## Live result (2026-07-17, real Preveti)

Driven headless against `sim_host` in-memory (demo seed) at `:8080`:

- `simulate_and_project { objective: "reduce CI/build cost friction for
  cost-conscious developers", audience_json: {...} }` → `run_id:
  run-ab45a4d2-abad-4e0a-a931-7e3187fca974`, `status: "complete"`, elapsed 2.7s,
  `summary: { segments: 0, arena_frames: 1, errc_moves: 0, actors: 0 }`. ✅
- `$TMP/compass.yaml` persisted with `arena.from-run`:
  `dominance_hypothesis: 'Preveti winning framework: Base thesis (run
  run-ab45a4d2-…, source simuli)'` — evidence-bound, `origin: simuli`, real run
  id. ✅ Assertions 1 & 2 pass.
- **Note:** compass frames live in `compass.yaml`, not the intent harness, so
  `search_intent_harness` does not list them (it returned empty) — the projected
  counts come from the sim `summary` / `status`; `search` is for projected actors.
- Isolation held (throwaway `$TMP`; zero frontrails-product diff). ✅
