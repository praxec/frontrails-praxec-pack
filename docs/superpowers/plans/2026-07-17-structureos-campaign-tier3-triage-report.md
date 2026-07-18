# StructureOS Campaign — Tier-3 Triage-Report Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `flow.findings.triage-report` — a **read-only** orchestrator that scans the target repo via StructureOS, buckets every finding code into its tier per the canonical taxonomy, and emits an observability report grouping the full findings landscape by tier. It NEVER mutates code, dismisses findings, or commits a branch — reporting IS the deliverable (spec §4 Tier-3 DoD: "never required to reach zero").

**Architecture:** The additive `frontrails-campaign.yaml` gains one orchestrator plus two scripts. The flow scans via the `structureos` MCP connection (`scan_repo`), relays the raw MCP text envelope to a new `run.campaign.triage-extract` script (which jq-parses `_summary.by_id` and buckets each `SOS<digits>` code into tier1/tier2/tier3 per the taxonomy, defaulting unknown codes to Tier 3 and flagging them in `unknown_codes`), then emits the landscape through a new `run.campaign.triage-report` script (§9-shaped JSON + a per-tier markdown render). State machine is minimal: `scanning → extracting → reporting → done`. Deliberately avoids the mass-dismissal trap the spec's FMECA vetting removed.

**Tech Stack:** praxec (`praxec check`/`command`), StructureOS MCP (`structureos-mcp`), bash + `jq`. No cargo, no coding agent — Tier 3 touches no source.

**Scope:** ONLY the read-only triage report. The optional `cap.triage.dismiss` capability (spec §6, "new (optional, opt-in)") is explicitly OUT of scope for this slice — dismissal is a separate governed capability and the report is the mandated Tier-3 deliverable; adding it now would be speculative. No burn-down, no mutation, no branch commit.

## Global Constraints

- Home: `frontrails-praxec-pack`; additive `frontrails-campaign.yaml` only (spec §2). No `praxec.repo.yaml` migration.
- **Tier 3 is read-only** — no `actor: agent`, no `kind: llm`, no mutation, no `dismiss_finding`, no branch commit (spec §4, §7). Only `deterministic`/`mcp`/`script`/`noop` executors.
- StructureOS is the language-neutral structural source (spec §5a). The scan uses `kind: mcp, connection: structureos, tool: structureos, map: {action, params}` — the same pattern as `flow.findings.structural`. A `kind: mcp` executor's `$.output` is the RAW MCP text envelope (structureos-mcp declares no `outputSchema`), so bind `scan_text: "$.output.content.0.text"` and jq-parse it in a script (the same T2.5 finding the Tier-2 slice recorded).
- Taxonomy is read as DATA (spec §5). praxec flows cannot read `taxonomy.finding-tiers.yaml` at runtime, so the flow's `initialContext` mirrors the code→tier lists consistent with the table (same realization the Tier-1 plan documents for SOS027) and passes them to the extract script as space-separated argv.
- **Invariant (spec §5):** an unknown SOS code defaults to Tier 3 (report), and unknown-code counts are a first-class signal (`unknown_codes`, §9). Non-finding sentinel keys (e.g. `SOS-RUSTC-OFF`) are filtered by a `^SOS[0-9]+$` guard.
- All script subjects use blessed roots (SPEC §22.4) — `run.campaign.*`. `subject:` must be LITERAL (a `$.context.*` subject passes `praxec check` but fails at runtime — Tier-1 finding).
- Templating into scripts uses jsonpath `$.context.x` (NOT `{{ }}`) via `args:`; `workingDirectory:` sets cwd. Commit messages end with `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

---

## File Structure

- `frontrails-campaign.yaml` — gains `flow.findings.triage-report` (Task 3) + `run.campaign.triage-extract` and `run.campaign.triage-report` scripts (Tasks 1, 2).
- `examples/campaign-check.yaml` — NO change needed (the flow references only the `structureos` connection + the two new same-file scripts, all already resolved by the existing include set). Confirmed by `praxec check`.
- `README.md` — extend "Campaign wiring" with a Tier-3 subsection (Task 4).
- `tests/campaign-tier3-e2e.md` — the manual/CI-gated E2E procedure (Task 5).

---

### Task 1: `run.campaign.triage-extract` script

**Files:**
- Modify: `frontrails-campaign.yaml` (add one script to `scripts:`)

**Interfaces:**
- Produces `run.campaign.triage-extract` — argv1 = `scan_text` (raw StructureOS scan JSON string from the MCP envelope), argv2..4 = the tier1/tier2/tier3 code-lists (space-separated). jq-parses `_summary.by_id`, buckets each real `SOS<digits>` code by tier (unknown → Tier 3 + `unknown_codes`), and emits `{routing_json, total, tier1, tier2, tier3}` on stdout (auto-parsed into `$.output.json.*`). `routing_json` is the full payload (by_code + by_tier + unknown_codes) JSON-encoded as a string, relayed to the report script. exit 0 always. Mirrors `run.campaign.extract-godfile-summary`'s raw-text-relay pattern.

- [ ] **Step 1: Add the script** (see the shipped body in `frontrails-campaign.yaml`).
- [ ] **Step 2: Verify the bucketing** with a crafted `scan_text` (`{"_summary":{"by_id":{"SOS001":1,"SOS038":1,"SOS103":1}}}`): assert `total==3`, `tier2==1`, `tier3==2`, `unknown_codes==["SOS038"]` (SOS038 is unknown — the taxonomy lists SOS037 for packaging, but StructureOS emits SOS038: a real drift signal). Also assert a `{"SOS-RUSTC-OFF":1}` input yields `total==0` (sentinel filtered).
- [ ] **Step 3: `praxec check`** → exit 0 (`run.` is blessed).

---

### Task 2: `run.campaign.triage-report` script

**Files:**
- Modify: `frontrails-campaign.yaml` (add one script to `scripts:`)

**Interfaces:**
- Produces `run.campaign.triage-report` — argv1 = `routing_json` (the full payload string from Task 1). Writes `.praxec/reports/triage-<stamp>.json` (§9-shaped: `findings_in`, `tier_routing`, `tier3`) + `.md` (per-tier grouped render), emits `{report_path}`. exit 0. Read-only — no mutation, no branch. Mirrors the Tier-1/Tier-2 report scripts' cwd/arg contract.

- [ ] **Step 1: Add the script** (see the shipped body).
- [ ] **Step 2: Verify the JSON + markdown contract** — pipe the Task-1 `routing_json` in; assert `report_path` is a string and the JSON has `tier_routing.tier2 == 1` and `tier3.reported == 2`; assert the markdown groups codes under `## Tier 1/2/3` headings and lists the `unknown_codes`.
- [ ] **Step 3: `praxec check`** → exit 0.

---

### Task 3: `flow.findings.triage-report` orchestrator

**Files:**
- Modify: `frontrails-campaign.yaml` (add to `workflows:`)

**Interfaces:**
- Consumes: StructureOS (`scan_repo`), `run.campaign.triage-extract` (Task 1), `run.campaign.triage-report` (Task 2).
- Produces: workflow `flow.findings.triage-report`, initial `scanning`, terminal `done`. State path: `scanning → extracting → reporting → done`. All transitions `actor: deterministic` (auto-chains to `done` in one `command` call). `initialContext` carries the tier code-lists (consistent with the taxonomy) + `routing_json`/`total`/`tier*_count` slots.

- [ ] **Step 1: Write the orchestrator** (see the shipped definition). Bind `scan_text: "$.output.content.0.text"` (raw MCP envelope), pass it + the three code-lists to `extracting`'s script, then relay `routing_json` to `reporting`'s script. Bind the scalar tier totals into context too so the audit trace (§9) shows the routing outcome.
- [ ] **Step 2: Structural validation** — `praxec check --config examples/campaign-check.yaml` → exit 0, `flow.findings.triage-report` in the workflow list. No `examples/campaign-check.yaml` edit is needed (the flow adds no new external references).
- [ ] **Step 3: Commit** the three additive units.

---

### Task 4: README Tier-3 wiring subsection

**Files:**
- Modify: `README.md`

- [ ] **Step 1** Append a "Tier 3 (advisory report)" subsection under "Campaign wiring" noting: read-only (no coding agent, no cargo, no mutation), scans via `structureos`, groups the full landscape by tier, `unknown_codes` is the drift signal, and no `examples/campaign-check.yaml` change is required.
- [ ] **Step 2: Commit.**

---

### Task 5: E2E go/no-go — scan + tier grouping (read-only)

**Files:**
- Create: `tests/campaign-tier3-e2e.md`

**Interfaces:**
- Consumes everything above. Produces a documented run proving `flow.findings.triage-report` scans a fixture, groups counts by tier with real counts (Tier 2 SOS001, Tier 3 SOS103 + unknown SOS038), surfaces the drift signal, and mutates nothing.

- [ ] **Step 1: Write the E2E procedure** mirroring `tests/campaign-tier2-e2e.md` (throwaway `mktemp -d` + `git init` + `praxec.repo.yaml`; a driver overlay with `store:`/`repos:`). **Key Tier-3-specific detail:** the driver overlay must ALSO deep-merge a `connections.structureos.env.STRUCTUREOS_WORKSPACE_ROOT: "$WORK"` override — praxec does NOT spawn the MCP connection with cwd=`$.run.repo_root`, so the shipped `"."` resolves to the gateway's own cwd, not the fixture. Pinning it makes the scan deterministic against the fixture.
- [ ] **Step 2: Execute** against `rust-godfile` (`input: {}`) — assert `scanning → extracting → reporting → done`, `total==3`, `tier2==1` (SOS001), `tier3==2` (SOS103 + SOS038), `unknown_codes==["SOS038"]`, no `campaign/*` branch, source clean.
- [ ] **Step 3: Execute the zero-findings case** against `rust-findings` — assert `total==0`, all tiers 0, `unknown_codes==[]` (SOS-RUSTC-OFF filtered), terminal `done`.
- [ ] **Step 4: Assert + clean up** — remove throwaways; confirm no `.praxec/` or `campaign/*` residue in the pack repo.
- [ ] **Step 5: Commit** the E2E doc.

---

## Self-Review

**Spec coverage:**
- §4 Tier-3 DoD (report, never required to reach zero; no mass-dismissal) → Task 3 read-only state machine, no `dismiss_finding`. ✓
- §5 taxonomy as data + unknown→Tier 3 + `unknown_codes` first-class → Task 1 bucketing. ✓
- §5a StructureOS is the language-neutral structural source → Task 3 `kind: mcp` scan. ✓
- §6/§7 `flow.findings.triage-report` = read-only orchestrator, minimal state machine → Task 3. ✓
- §9 observability (per-run report: `findings_in`, `tier_routing`, `tier3`, `unknown_codes`) → Task 2 report. ✓
- §6 `cap.triage.dismiss` marked optional/opt-in → explicitly OUT of scope, stated. ✓

**Placeholder scan:** No TBD/TODO in deliverables. One explicit live-verification point (poka-yoke, not a guess): the `structureos` connection's `STRUCTUREOS_WORKSPACE_ROOT` does NOT track `$.run.repo_root` at runtime — the E2E pins it to the fixture in the throwaway driver overlay (Task 5 Step 1). Recorded, not assumed.

**Type/name consistency:** `routing_json`, `total`, `tier1_count`/`tier2_count`/`tier3_count`, `scan_text`, `tier1_codes`/`tier2_codes`/`tier3_codes` are defined in `initialContext` (Task 3) and threaded through `run.campaign.triage-extract` (`$.output.json.*`) and `run.campaign.triage-report` (argv1). Script subjects `run.campaign.triage-extract` / `run.campaign.triage-report` (Tasks 1/2) match their `subject:` in Task 3.

**Known live-verification points confirmed during execution:**
1. `$.output.content.0.text` is the correct raw-envelope binding for a `structureos` `kind: mcp` scan (same as Tier 2). ✓
2. `praxec check` resolves `flow.findings.triage-report` with the existing `examples/campaign-check.yaml` include set — no config edit needed. ✓
3. `STRUCTUREOS_WORKSPACE_ROOT` must be pinned to the fixture in the E2E driver overlay (does not follow `$.run.repo_root`). ✓
