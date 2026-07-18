# Tier-3 triage-report E2E (go/no-go)

Requires `praxec` + `structureos-mcp` on `PATH`, sibling `cognitive-architectures`
+ `cognitive-architectures-max` checkouts (only so the shared driver overlay
resolves — Tier 3 itself uses NEITHER), and `jq`. Tier 3 is **read-only**: it
scans and reports, never mutates code, dismisses a finding, or commits a branch.
The full findings landscape is grouped by tier; reporting IS the deliverable
(spec §4). Every run happens in an ISOLATED throwaway copy so it never touches
the pack repo.

- `praxec` (this procedure was verified against `praxec 0.0.24`).
- `jq` on `PATH` (used by the campaign scripts and this procedure's assertions).
- `../cognitive-architectures` and `../cognitive-architectures-max` beside this
  repo (only to satisfy the shared driver-overlay `include:` list — no Tier-3
  transition dispatches into either).

## Why the driver overlay pins `STRUCTUREOS_WORKSPACE_ROOT`

`frontrails.yaml`'s `structureos` connection sets `STRUCTUREOS_WORKSPACE_ROOT: "."`.
praxec does NOT spawn the MCP connection process with cwd = `$.run.repo_root`, so
`"."` resolves to the gateway's own cwd — NOT the throwaway fixture (verified
live: a run with cwd set to `$WORK` still scanned the ambient repo, 3249 findings
/ 29 god-files, not the 1-file fixture). To make the scan deterministic against
the fixture, the throwaway driver overlay deep-merges a
`connections.structureos.env.STRUCTUREOS_WORKSPACE_ROOT: "$WORK"` override (maps
merge, scalars: later wins). This is a throwaway-config detail, exactly like the
Tier-1/Tier-2 E2E's `store:`/`repos:` overlay — the shipped pack config is
unchanged.

## Procedure

1. Copy a fixture to a throwaway git repo and give it a `praxec.repo.yaml`
   manifest (required to load a declared writable repo — same requirement
   documented in `tests/campaign-tier1-e2e.md`):

   ```bash
   PACK="$PWD"                                    # run from the pack repo root
   WORK=$(mktemp -d)
   cp -r tests/fixtures/rust-godfile/. "$WORK/"   # or rust-findings for the zero case
   rm -rf "$WORK/target" "$WORK/.frontrails/structure/cache"
   git -C "$WORK" init -q
   cat > "$WORK/praxec.repo.yaml" <<'EOF'
   schema: praxec.repo/v1
   name: rust-godfile-fixture
   namespace: e2e
   version: 0.0.0
   description: Throwaway E2E fixture repo; ships no capabilities/flows of its own.
   EOF
   git -C "$WORK" add -A
   git -C "$WORK" -c user.email=e2e@local -c user.name=e2e commit -qm init
   ```

2. Structure check the real shipped config first:

   ```bash
   praxec check --config "$PACK/examples/campaign-check.yaml"   # exit 0
   ```

3. **Build a driver config** outside the pack repo — `include`s the same files by
   absolute path, adds a `store:`, declares the throwaway `repos:` entry, AND
   pins the scan target (see the note above):

   ```yaml
   # e2e-gateway.yaml (throwaway; NOT part of this repo)
   include:
     - /abs/path/to/frontrails-praxec-pack/frontrails.yaml
     - /abs/path/to/frontrails-praxec-pack/frontrails-campaign.yaml
     - /abs/path/to/frontrails-praxec-pack/extensions/rust.yaml
     - /abs/path/to/cognitive-architectures/scripts-library/verify.cargo.cwd.yaml
     - /abs/path/to/cognitive-architectures-max/orchestrators/flow.refactor.god-file.yaml
   connections:
     structureos:
       kind: mcp
       command: "structureos-mcp"
       env:
         STRUCTUREOS_WORKSPACE_ROOT: "$WORK"     # absolute path to the throwaway
   store:
     kind: sqlite
     path: /abs/path/to/throwaway/e2e-store.db
   repos:
     - path: "$WORK"
       writable: true
   ```

   ```bash
   praxec check --config /abs/path/to/throwaway/e2e-gateway.yaml   # exit 0
   ```

4. Drive with `$WORK` as `cwd` (so `$.run.repo_root` resolves there for the
   report scripts' `workingDirectory`):

   ```bash
   cd "$WORK"
   praxec command --config "$DRIVER_CFG" \
     '{"definitionId":"flow.findings.triage-report","input":{}}'
   ```

   Every transition is `actor: deterministic`, so one `command` call auto-chains
   `scanning → extracting → reporting → done`. Read `.context.routing_json` and
   the written `.praxec/reports/triage-*.{json,md}` from the response.

5. Repeat step 1 + 4 against a **fresh** throwaway copy of `rust-findings` (no
   real findings) for the zero-count case.

## Acceptance

```bash
# Run 1 (rust-godfile) — from the `praxec command` response's context.routing_json:
RJ=$(jq -r '.context.routing_json' run1.json)
echo "$RJ" | jq -e '.total == 3'                 >/dev/null && echo OK_TOTAL       || echo FAIL_TOTAL
echo "$RJ" | jq -e '.tier2 == 1'                 >/dev/null && echo OK_TIER2       || echo FAIL_TIER2
echo "$RJ" | jq -e '.by_tier.tier2.SOS001 == 1'  >/dev/null && echo OK_TIER2_CODE  || echo FAIL_TIER2_CODE
echo "$RJ" | jq -e '.tier3 == 2'                 >/dev/null && echo OK_TIER3       || echo FAIL_TIER3
# SOS038 is a real drift signal: the taxonomy lists SOS037 for packaging, but
# StructureOS emits SOS038, so it is unrecognized -> Tier 3 default + unknown_codes.
echo "$RJ" | jq -e '.unknown_codes == ["SOS038"]' >/dev/null && echo OK_UNKNOWN    || echo FAIL_UNKNOWN
[ "$(jq -r '.workflow.state' run1.json)" = "done" ] && echo OK_DONE               || echo FAIL_DONE

# read-only poka-yoke: Tier 3 never mutates source or commits a branch
git -C "$WORK" branch --list 'campaign/*' | grep -q . && echo FAIL_BRANCH || echo OK_NO_BRANCH
git -C "$WORK" status --porcelain | grep -v '.praxec' | grep -q . && echo FAIL_MUTATED || echo OK_SRC_CLEAN

# Run 2 (rust-findings, no god-file / no real findings):
RJ2=$(jq -r '.context.routing_json' run2.json)
echo "$RJ2" | jq -e '.total == 0'          >/dev/null && echo OK_ZERO_TOTAL   || echo FAIL_ZERO_TOTAL
echo "$RJ2" | jq -e '.unknown_codes == []' >/dev/null && echo OK_ZERO_UNKNOWN || echo FAIL_ZERO_UNKNOWN
[ "$(jq -r '.workflow.state' run2.json)" = "done" ] && echo OK_ZERO_DONE      || echo FAIL_ZERO_DONE
```

Expected: `OK_TOTAL`, `OK_TIER2`, `OK_TIER2_CODE`, `OK_TIER3`, `OK_UNKNOWN`,
`OK_DONE`, `OK_NO_BRANCH`, `OK_SRC_CLEAN`, `OK_ZERO_TOTAL`, `OK_ZERO_UNKNOWN`,
`OK_ZERO_DONE`.

## Observed report (rust-godfile, verified live)

`context.routing_json`:

```json
{"total":3,"tier1":0,"tier2":1,"tier3":2,"unknown_codes":["SOS038"],
 "by_tier":{"tier1":{},"tier2":{"SOS001":1},"tier3":{"SOS038":1,"SOS103":1}}}
```

`.praxec/reports/triage-<stamp>.md`:

```
# Triage report <stamp> (Tier 3 — read-only, no mutation)

Total findings: 3

## Tier 1 — safe auto-fix (sweep-safe): 0
- (none)

## Tier 2 — structural refactor (god-file): 1
- SOS001: 1

## Tier 3 — advisory report (read-only): 2
- SOS038: 1
- SOS103: 1

Unknown codes (drift signal — routed to Tier 3 by default): SOS038
```

`.praxec/reports/triage-<stamp>.json` (§9 shape):

```json
{"tier":3,
 "findings_in":{"by_code":{"SOS001":1,"SOS038":1,"SOS103":1},"total":3},
 "tier_routing":{"tier1":0,"tier2":1,"tier3":2,"unknown_codes":["SOS038"]},
 "tier3":{"reported":2,"dismissed":0,"deferred":0}}
```

## Observed report (rust-findings, verified live)

```
# Triage report <stamp> (Tier 3 — read-only, no mutation)

Total findings: 0

## Tier 1 — safe auto-fix (sweep-safe): 0
- (none)
## Tier 2 — structural refactor (god-file): 0
- (none)
## Tier 3 — advisory report (read-only): 0
- (none)

Unknown codes (drift signal — routed to Tier 3 by default): (none)
```

The `SOS-RUSTC-OFF` sentinel in `_summary.by_id` is filtered by the extractor's
`^SOS[0-9]+$` guard, so it never counts as a finding — the report reads a clean
zero.

## Note — a full real-repo scan is an even richer proof

The very first drive (before pinning `STRUCTUREOS_WORKSPACE_ROOT`) scanned the
ambient `frontrails-product` (1397 files) and produced a genuine multi-tier
landscape — `total: 3249`, `tier1: 3` (SOS027), `tier2: 29` (SOS001), `tier3:
3217`, and 17 `unknown_codes` — grouped correctly by tier. That is the
production shape; the fixture runs above are the deterministic CI go/no-go.

## Cleanup

```bash
rm -rf "$WORK" "$WORK2" /abs/path/to/throwaway
git branch --list "campaign/*"    # must be empty
ls .praxec 2>&1                   # must not exist
```
